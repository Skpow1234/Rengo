//! Database node orchestration — composition root for single-node operation.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use shardforge_config::{ShardForgeConfig, StorageEngineType};
use shardforge_core::{Result, ShardForgeError};
use shardforge_storage::{MVCCStorage, StorageConfig, StorageEngine, StorageEngineFactory};
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
    /// Version string
    pub version: String,
}

/// Composition root: storage + SQL + transactions + TCP SQL server
pub struct DatabaseNode {
    config: ShardForgeConfig,
    executor: Arc<Mutex<QueryExecutor>>,
    tx_manager: Arc<TransactionManager>,
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
    pub async fn from_config(config: ShardForgeConfig) -> Result<Self> {
        let data_dir = config.cluster.data_directory.clone();
        std::fs::create_dir_all(&data_dir).map_err(|e| ShardForgeError::internal(format!(
            "Failed to create data directory {}: {}",
            data_dir.display(),
            e
        )))?;

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
            bind_address,
            storage_engine_name,
            running: Arc::new(AtomicBool::new(false)),
            table_count: Arc::new(RwLock::new(table_count)),
        })
    }

    /// Load configuration from a TOML file and open the node
    pub async fn from_config_file(path: impl AsRef<Path>) -> Result<Self> {
        let loader = shardforge_config::ConfigLoader::new();
        let config = loader.load_from_file(path).await?;
        Self::from_config(config).await
    }

    /// Execute a single SQL statement (parse → optional txn → execute)
    pub async fn execute_sql(&self, sql: &str) -> Result<QueryResult> {
        let mut parser = SqlParser::new();
        let statement = parser.parse(sql)?;

        match statement {
            Statement::Begin(_) => {
                let _ = self
                    .tx_manager
                    .begin_transaction(TransactionConfig::default())
                    .await?;
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                });
            }
            Statement::Commit(_) | Statement::Rollback(_) => {
                // Explicit multi-statement sessions are tracked separately later;
                // for now these are accepted as no-ops after auto-commit paths.
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                });
            }
            _ => {}
        }

        let tx_id = self
            .tx_manager
            .begin_transaction(TransactionConfig::default())
            .await?;

        let result = {
            let mut executor = self.executor.lock().await;
            executor.execute(statement).await
        };

        match result {
            Ok(query_result) => {
                if let Err(e) = self.tx_manager.commit_transaction(tx_id).await {
                    tracing::warn!("Transaction commit failed after successful execute: {}", e);
                }
                let count = {
                    let executor = self.executor.lock().await;
                    executor.context.catalog.list_tables().len()
                };
                *self.table_count.write().await = count;
                Ok(query_result)
            }
            Err(e) => {
                let _ = self.tx_manager.rollback_transaction(tx_id).await;
                Err(e)
            }
        }
    }

    /// Current node status snapshot
    pub async fn status(&self) -> NodeStatus {
        NodeStatus {
            running: self.running.load(Ordering::SeqCst),
            bind_address: self.bind_address.clone(),
            storage_engine: self.storage_engine_name.clone(),
            table_count: *self.table_count.read().await,
            version: crate::VERSION.to_string(),
        }
    }

    /// Bind TCP listener and serve newline-delimited JSON SQL requests until shutdown
    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr =
            self.bind_address.parse().map_err(|e| ShardForgeError::Config {
                message: format!("Invalid bind address '{}': {}", self.bind_address, e),
            })?;

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            ShardForgeError::internal(format!("Failed to bind {}: {}", addr, e))
        })?;

        let local = listener.local_addr().map_err(|e| {
            ShardForgeError::internal(format!("Failed to read local address: {}", e))
        })?;

        self.running.store(true, Ordering::SeqCst);
        println!("ShardForge node listening on {}", local);
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
        println!("ShardForge node stopped");
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
            table_count: Arc::clone(&self.table_count),
        })
    }

    async fn load_catalog(storage: &dyn StorageEngine, catalog: &mut SchemaCatalog) -> Result<()> {
        let list_key = shardforge_core::Key::from_string(CATALOG_LIST_KEY);
        let Some(list_value) = storage.get(&list_key).await? else {
            return Ok(());
        };

        let names: Vec<String> = serde_json::from_slice(list_value.as_slice()).map_err(|e| {
            ShardForgeError::internal(format!("Corrupt catalog list: {}", e))
        })?;

        for name in names {
            let key = shardforge_core::Key::from_string(&format!("{}{}", CATALOG_TABLE_PREFIX, name));
            if let Some(raw) = storage.get(&key).await? {
                let schema: crate::sql::TableSchema = serde_json::from_slice(raw.as_slice())
                    .map_err(|e| {
                        ShardForgeError::internal(format!(
                            "Corrupt catalog entry for '{}': {}",
                            name, e
                        ))
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
    table_count: Arc<RwLock<usize>>,
}

impl DatabaseNodeHandle {
    async fn execute_sql(&self, sql: &str) -> Result<QueryResult> {
        let mut parser = SqlParser::new();
        let statement = parser.parse(sql)?;

        let tx_id = self
            .tx_manager
            .begin_transaction(TransactionConfig::default())
            .await?;

        let result = {
            let mut executor = self.executor.lock().await;
            executor.execute(statement).await
        };

        match result {
            Ok(query_result) => {
                let _ = self.tx_manager.commit_transaction(tx_id).await;
                let count = {
                    let executor = self.executor.lock().await;
                    executor.context.catalog.list_tables().len()
                };
                *self.table_count.write().await = count;
                Ok(query_result)
            }
            Err(e) => {
                let _ = self.tx_manager.rollback_transaction(tx_id).await;
                Err(e)
            }
        }
    }
}

async fn handle_client(stream: TcpStream, node: Arc<DatabaseNodeHandle>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await.map_err(|e| {
        ShardForgeError::internal(format!("Read error: {}", e))
    })? {
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

        let mut out = serde_json::to_string(&response).map_err(|e| {
            ShardForgeError::internal(format!("Serialize response: {}", e))
        })?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await.map_err(|e| {
            ShardForgeError::internal(format!("Write error: {}", e))
        })?;
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
        let mut config = ShardForgeConfig::default();
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
    }
}
