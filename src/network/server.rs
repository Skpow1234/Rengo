//! gRPC / network server implementation

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use rengo_core::{Result, RengoError};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::network::protocol::proto;
use crate::network::NetworkConfig;
use crate::server::DatabaseNode;

/// gRPC server for Rengo database
pub struct GrpcServer {
    config: NetworkConfig,
    service: Arc<DatabaseServiceImpl>,
    shutdown_signal: Arc<RwLock<Option<tokio::sync::oneshot::Receiver<()>>>>,
}

impl GrpcServer {
    /// Create a new gRPC server
    pub fn new(config: NetworkConfig, service: DatabaseServiceImpl) -> Self {
        Self {
            config,
            service: Arc::new(service),
            shutdown_signal: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the gRPC server
    ///
    /// Full tonic serving requires the `grpc-enabled` feature and protoc.
    /// Until then, this validates the bind address and keeps the service handle
    /// ready; prefer `DatabaseNode::start` for the TCP SQL protocol.
    pub async fn start(&self) -> Result<()> {
        let addr: SocketAddr =
            self.config.bind_address.parse().map_err(|e| RengoError::Config {
                message: format!("Invalid bind address: {}", e),
            })?;

        println!("gRPC service configured for {}", addr);
        if self.service.node.is_some() {
            println!("DatabaseNode attached — SQL RPCs will execute against the live node");
        } else {
            println!("No DatabaseNode attached — RPCs return placeholders");
        }
        println!(
            "Note: enable `--features grpc-enabled` with protoc for full tonic serve; \
             use `rengo start` for the TCP SQL listener"
        );

        if let Some(_shutdown_rx) = self.shutdown_signal.read().await.as_ref() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Ok(())
    }

    /// Set shutdown signal
    pub async fn set_shutdown_signal(&self, receiver: tokio::sync::oneshot::Receiver<()>) {
        *self.shutdown_signal.write().await = Some(receiver);
    }
}

/// Implementation of the DatabaseService gRPC service
pub struct DatabaseServiceImpl {
    /// Optional live database node for query execution
    pub node: Option<Arc<DatabaseNode>>,
}

impl DatabaseServiceImpl {
    /// Create an empty service (placeholder responses)
    pub fn new() -> Self {
        Self { node: None }
    }

    /// Create a service backed by a live [`DatabaseNode`]
    pub fn with_node(node: Arc<DatabaseNode>) -> Self {
        Self { node: Some(node) }
    }
}

impl Default for DatabaseServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl proto::DatabaseService for DatabaseServiceImpl {
    async fn execute_query(
        &self,
        request: Request<proto::QueryRequest>,
    ) -> std::result::Result<Response<proto::QueryResponse>, Status> {
        let req = request.into_inner();

        if let Some(node) = &self.node {
            match node.execute_sql(&req.sql).await {
                Ok(_result) => {
                    return Ok(Response::new(proto::QueryResponse {
                        has_more: false,
                        cursor: "".to_string(),
                    }));
                }
                Err(e) => {
                    return Err(Status::internal(e.to_string()));
                }
            }
        }

        Ok(Response::new(proto::QueryResponse {
            has_more: false,
            cursor: "".to_string(),
        }))
    }

    async fn begin_transaction(
        &self,
        request: Request<proto::TransactionRequest>,
    ) -> std::result::Result<Response<proto::TransactionResponse>, Status> {
        let _req = request.into_inner();

        if let Some(node) = &self.node {
            match node.execute_sql("BEGIN").await {
                Ok(_) => {
                    return Ok(Response::new(proto::TransactionResponse {
                        transaction_id: format!(
                            "tx_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or(0)
                        ),
                        start_timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0),
                    }));
                }
                Err(e) => return Err(Status::internal(e.to_string())),
            }
        }

        Ok(Response::new(proto::TransactionResponse {
            transaction_id: "tx_unattached".to_string(),
            start_timestamp: 0,
        }))
    }

    async fn get_cluster_status(
        &self,
        request: Request<proto::StatusRequest>,
    ) -> std::result::Result<Response<proto::StatusResponse>, Status> {
        let _req = request.into_inner();

        let (healthy_nodes, leader) = if let Some(node) = &self.node {
            let status = node.status().await;
            (1u32, status.bind_address)
        } else {
            (0u32, "none".to_string())
        };

        Ok(Response::new(proto::StatusResponse {
            status: Some(proto::ClusterStatus {
                health: if healthy_nodes > 0 { 0 } else { 2 },
                total_nodes: 1,
                healthy_nodes,
                leader_node_id: leader,
            }),
        }))
    }
}
