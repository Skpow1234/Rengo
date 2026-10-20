//! Database node orchestration — composition root for single-node operation.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rengo_config::{RengoConfig, StorageEngineType};
use rengo_core::{RengoError, Result, TransactionId};
use rengo_storage::{MVCCStorage, StorageConfig, StorageEngine, StorageEngineFactory};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};

use crate::sql::{
    ExecutionContext, QueryExecutor, QueryResult, SchemaCatalog, SqlParser, Statement,
};
use crate::transaction::{TransactionConfig, TransactionManager};

const CATALOG_LIST_KEY: &str = "__catalog__tables";
const CATALOG_TABLE_PREFIX: &str = "__catalog__table__";

/// Runtime status of a database node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Whether the node is currently serving
    pub running: bool,
    /// Configured bind address
    pub bind_address: String,
    /// Storage engine name
    pub storage_engine: String,
    /// Number of tables in the catalog
    pub table_count: usize,
    /// Whether an explicit SQL transaction session is open
    pub in_transaction: bool,
    /// Version string
    pub version: String,
}

/// Composition root: storage + SQL + transactions + TCP SQL server
pub struct DatabaseNode {
    config: RengoConfig,
    executor: Arc<Mutex<QueryExecutor>>,
    tx_manager: Arc<TransactionManager>,
    /// Explicit session transaction started by BEGIN (shared with connection handle)
    active_tx: Arc<Mutex<Option<TransactionId>>>,
    bind_address: String,
    storage_engine_name: String,
    running: Arc<AtomicBool>,
    table_count: Arc<RwLock<usize>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlRequest {
    sql: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlResponse {
    ok: bool,
    columns: Vec<String>,
    /// Row values encoded as UTF-8 lossy strings for the simple wire protocol
    rows: Vec<Vec<String>>,
    affected_rows: u64,
    error: Option<String>,
}

impl DatabaseNode {
    /// Open storage and build a node from configuration
    pub async fn from_config(config: RengoConfig) -> Result<Self> {
        let data_dir = config.cluster.data_directory.clone();
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            RengoError::internal(format!(
                "Failed to create data directory {}: {}",
                data_dir.display(),
                e
            ))
        })?;

        let storage_config = StorageConfig {
            block_cache_size_mb: config.storage.block_cache_size_mb,
            write_buffer_size_mb: config.storage.write_buffer_size_mb,
            max_write_buffer_number: config.storage.max_write_buffer_number,
            enable_statistics: config.storage.enable_statistics,
            compression: config.storage.compression,
        };

        let storage =
            StorageEngineFactory::create(config.storage.engine, &storage_config, &data_dir).await?;

        let storage_engine_name = match config.storage.engine {
            StorageEngineType::Memory => "memory",
            StorageEngineType::Sled => "sled",
            StorageEngineType::RocksDB => "rocksdb",
        }
        .to_string();

        let mvcc = Arc::new(MVCCStorage::new());
        let tx_manager = Arc::new(TransactionManager::new(Arc::clone(&mvcc)));

        // Executor owns a separate MVCC instance for storage-layer versioning today;
        // TransactionManager tracks SQL-level transaction lifecycle.
        let mut catalog = SchemaCatalog::new();
        Self::load_catalog(storage.as_ref(), &mut catalog).await?;
        let table_count = catalog.list_tables().len();

        let context = ExecutionContext {
            storage,
            mvcc: MVCCStorage::new(),
            catalog,
        };
        let executor = Arc::new(Mutex::new(QueryExecutor::new(context)));

        let bind_address = config.node.bind_address.clone();

        Ok(Self {
            config,
            executor,
            tx_manager,
            active_tx: Arc::new(Mutex::new(None)),
            bind_address,
            storage_engine_name,
            running: Arc::new(AtomicBool::new(false)),
            table_count: Arc::new(RwLock::new(table_count)),
        })
    }

    /// Load configuration from a TOML file and open the node
    pub async fn from_config_file(path: impl AsRef<Path>) -> Result<Self> {
        let loader = rengo_config::ConfigLoader::new();
        let config = loader.load_from_file(path).await?;
        Self::from_config(config).await
    }

    /// Execute a single SQL statement (parse → session/auto txn → execute)
    pub async fn execute_sql(&self, sql: &str) -> Result<QueryResult> {
        execute_sql_inner(&self.executor, &self.tx_manager, &self.active_tx, &self.table_count, sql)
            .await
    }

    /// Current node status snapshot
    pub async fn status(&self) -> NodeStatus {
        NodeStatus {
            running: self.running.load(Ordering::SeqCst),
            bind_address: self.bind_address.clone(),
            storage_engine: self.storage_engine_name.clone(),
            table_count: *self.table_count.read().await,
            in_transaction: self.active_tx.lock().await.is_some(),
            version: crate::VERSION.to_string(),
        }
    }

    /// Bind TCP listener and serve newline-delimited JSON SQL requests until shutdown
    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr = self.bind_address.parse().map_err(|e| RengoError::Config {
            message: format!("Invalid bind address '{}': {}", self.bind_address, e),
        })?;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| RengoError::internal(format!("Failed to bind {}: {}", addr, e)))?;

        let local = listener
            .local_addr()
            .map_err(|e| RengoError::internal(format!("Failed to read local address: {}", e)))?;

        self.running.store(true, Ordering::SeqCst);
        println!("Rengo node listening on {}", local);
        println!("Storage engine: {}", self.storage_engine_name);
        println!("Cluster: {}", self.config.cluster.name);
        println!("Send newline-delimited JSON: {{\"sql\":\"SELECT 1\"}}");

        let running = Arc::clone(&self.running);
        let node = self.clone_handle();

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            tokio::select! {
                accept = listener.accept() => {
                    match accept {
                        Ok((stream, peer)) => {
                            let node = node.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, node).await {
                                    tracing::debug!("Client {} closed: {}", peer, e);
                                }
                            });
                        }
                        Err(e) => {
                            if running.load(Ordering::SeqCst) {
                                tracing::error!("Accept error: {}", e);
                            }
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("Shutdown signal received");
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }

        self.running.store(false, Ordering::SeqCst);
        println!("Rengo node stopped");
        Ok(())
    }

    /// Request the server loop to stop (cooperative; used by tests)
    pub fn request_stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn clone_handle(&self) -> Arc<DatabaseNodeHandle> {
        Arc::new(DatabaseNodeHandle {
            executor: Arc::clone(&self.executor),
            tx_manager: Arc::clone(&self.tx_manager),
            active_tx: Arc::clone(&self.active_tx),
            table_count: Arc::clone(&self.table_count),
        })
    }

    async fn load_catalog(storage: &dyn StorageEngine, catalog: &mut SchemaCatalog) -> Result<()> {
        let list_key = rengo_core::Key::from_string(CATALOG_LIST_KEY);
        let Some(list_value) = storage.get(&list_key).await? else {
            return Ok(());
        };

        let names: Vec<String> = serde_json::from_slice(list_value.as_slice())
            .map_err(|e| RengoError::internal(format!("Corrupt catalog list: {}", e)))?;

        for name in names {
            let key = rengo_core::Key::from_string(&format!("{}{}", CATALOG_TABLE_PREFIX, name));
            if let Some(raw) = storage.get(&key).await? {
                let schema: crate::sql::TableSchema = serde_json::from_slice(raw.as_slice())
                    .map_err(|e| {
                        RengoError::internal(format!("Corrupt catalog entry for '{}': {}", name, e))
                    })?;
                catalog.add_table(schema);
            }
        }

        Ok(())
    }
}

/// Lightweight cloneable handle for connection tasks
struct DatabaseNodeHandle {
    executor: Arc<Mutex<QueryExecutor>>,
    tx_manager: Arc<TransactionManager>,
    active_tx: Arc<Mutex<Option<TransactionId>>>,
    table_count: Arc<RwLock<usize>>,
}

impl DatabaseNodeHandle {
    async fn execute_sql(&self, sql: &str) -> Result<QueryResult> {
        execute_sql_inner(&self.executor, &self.tx_manager, &self.active_tx, &self.table_count, sql)
            .await
    }
}

async fn execute_sql_inner(
    executor: &Arc<Mutex<QueryExecutor>>,
    tx_manager: &Arc<TransactionManager>,
    active_tx: &Arc<Mutex<Option<TransactionId>>>,
    table_count: &Arc<RwLock<usize>>,
    sql: &str,
) -> Result<QueryResult> {
    let mut parser = SqlParser::new();
    let statement = parser.parse(sql)?;

    match &statement {
        Statement::Begin(_) => {
            let mut guard = active_tx.lock().await;
            if guard.is_some() {
                return Err(RengoError::Transaction {
                    message: "Transaction already in progress".to_string(),
                });
            }
            let tx_id = tx_manager.begin_transaction(TransactionConfig::default()).await?;
            *guard = Some(tx_id);
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            });
        }
        Statement::Commit(_) => {
            let mut guard = active_tx.lock().await;
            let Some(tx_id) = guard.take() else {
                return Err(RengoError::Transaction {
                    message: "No active transaction to COMMIT".to_string(),
                });
            };
            tx_manager.commit_transaction(tx_id).await?;
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            });
        }
        Statement::Rollback(_) => {
            let mut guard = active_tx.lock().await;
            let Some(tx_id) = guard.take() else {
                return Err(RengoError::Transaction {
                    message: "No active transaction to ROLLBACK".to_string(),
                });
            };
            tx_manager.rollback_transaction(tx_id).await?;
            return Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
            });
        }
        _ => {}
    }

    let session_tx = { active_tx.lock().await.clone() };
    let (tx_id, auto_commit) = if let Some(tx_id) = session_tx {
        (tx_id, false)
    } else {
        (tx_manager.begin_transaction(TransactionConfig::default()).await?, true)
    };

    let result = {
        let mut exec = executor.lock().await;
        exec.execute(statement).await
    };

    match result {
        Ok(query_result) => {
            if auto_commit {
                if let Err(e) = tx_manager.commit_transaction(tx_id).await {
                    tracing::warn!("Auto-commit failed after successful execute: {}", e);
                }
            }
            let count = {
                let exec = executor.lock().await;
                exec.context.catalog.list_tables().len()
            };
            *table_count.write().await = count;
            Ok(query_result)
        }
        Err(e) => {
            if auto_commit {
                let _ = tx_manager.rollback_transaction(tx_id).await;
            }
            Err(e)
        }
    }
}

async fn handle_client(stream: TcpStream, node: Arc<DatabaseNodeHandle>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| RengoError::internal(format!("Read error: {}", e)))?
    {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<SqlRequest>(&line) {
            Ok(req) => match node.execute_sql(&req.sql).await {
                Ok(result) => SqlResponse {
                    ok: true,
                    columns: result.columns,
                    rows: result
                        .rows
                        .into_iter()
                        .map(|row| {
                            row.into_iter()
                                .map(|v| String::from_utf8_lossy(v.as_slice()).into_owned())
                                .collect()
                        })
                        .collect(),
                    affected_rows: result.affected_rows,
                    error: None,
                },
                Err(e) => SqlResponse {
                    ok: false,
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                    error: Some(e.to_string()),
                },
            },
            Err(e) => SqlResponse {
                ok: false,
                columns: vec![],
                rows: vec![],
                affected_rows: 0,
                error: Some(format!("Invalid request JSON: {}", e)),
            },
        };

        let mut out = serde_json::to_string(&response)
            .map_err(|e| RengoError::internal(format!("Serialize response: {}", e)))?;
        out.push('\n');
        writer
            .write_all(out.as_bytes())
            .await
            .map_err(|e| RengoError::internal(format!("Write error: {}", e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_node_execute_sql_roundtrip() {
        let temp = TempDir::new().unwrap();
        let mut config = RengoConfig::default();
        config.cluster.data_directory = temp.path().to_path_buf();
        config.storage.engine = StorageEngineType::Memory;
        config.node.bind_address = "127.0.0.1:0".to_string();

        let node = DatabaseNode::from_config(config).await.unwrap();

        let create = node
            .execute_sql("CREATE TABLE users (id INTEGER NOT NULL, name VARCHAR(255))")
            .await
            .unwrap();
        assert_eq!(create.affected_rows, 1);

        let insert = node
            .execute_sql("INSERT INTO users (id, name) VALUES (1, 'alice')")
            .await
            .unwrap();
        assert_eq!(insert.affected_rows, 1);

        let select = node.execute_sql("SELECT id, name FROM users").await.unwrap();
        assert_eq!(select.rows.len(), 1);

        let status = node.status().await;
        assert_eq!(status.table_count, 1);
        assert!(!status.running);
        assert!(!status.in_transaction);
    }

    #[tokio::test]
    async fn test_session_transaction_begin_commit() {
        let temp = TempDir::new().unwrap();
        let mut config = RengoConfig::default();
        config.cluster.data_directory = temp.path().to_path_buf();
        config.storage.engine = StorageEngineType::Memory;

        let node = DatabaseNode::from_config(config).await.unwrap();
        node.execute_sql("CREATE TABLE t (id INTEGER)").await.unwrap();

        node.execute_sql("BEGIN").await.unwrap();
        assert!(node.status().await.in_transaction);

        node.execute_sql("INSERT INTO t (id) VALUES (1)").await.unwrap();
        node.execute_sql("COMMIT").await.unwrap();
        assert!(!node.status().await.in_transaction);

        let rows = node.execute_sql("SELECT id FROM t").await.unwrap();
        assert_eq!(rows.rows.len(), 1);

        // Nested BEGIN should fail
        node.execute_sql("BEGIN").await.unwrap();
        assert!(node.execute_sql("BEGIN").await.is_err());
        node.execute_sql("ROLLBACK").await.unwrap();
    }

    #[tokio::test]
    async fn test_session_transaction_rollback_clears_flag() {
        let temp = TempDir::new().unwrap();
        let mut config = RengoConfig::default();
        config.cluster.data_directory = temp.path().to_path_buf();
        config.storage.engine = StorageEngineType::Memory;

        let node = DatabaseNode::from_config(config).await.unwrap();
        node.execute_sql("BEGIN").await.unwrap();
        node.execute_sql("ROLLBACK").await.unwrap();
        assert!(!node.status().await.in_transaction);
        assert!(node.execute_sql("COMMIT").await.is_err());
    }
}
