//! End-to-end DatabaseNode integration tests

use shardforge::config::{ShardForgeConfig, StorageEngineType};
use shardforge::server::DatabaseNode;
use tempfile::TempDir;

#[tokio::test]
async fn test_database_node_ddl_dml() {
    let temp = TempDir::new().unwrap();
    let mut config = ShardForgeConfig::default();
    config.cluster.data_directory = temp.path().join("data");
    config.storage.engine = StorageEngineType::Memory;
    config.node.bind_address = "127.0.0.1:0".to_string();

    let node = DatabaseNode::from_config(config).await.expect("open node");

    node.execute_sql("CREATE TABLE items (id INTEGER NOT NULL, name VARCHAR(64))")
        .await
        .expect("create table");

    node.execute_sql("INSERT INTO items (id, name) VALUES (1, 'widget')")
        .await
        .expect("insert");

    let result = node
        .execute_sql("SELECT id, name FROM items")
        .await
        .expect("select");

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns.len(), 2);

    let status = node.status().await;
    assert_eq!(status.table_count, 1);
    assert_eq!(status.storage_engine, "memory");
}

#[tokio::test]
async fn test_catalog_survives_reopen_with_memory_is_empty() {
    // Memory engine does not persist across process reopen; this asserts clean open.
    let temp = TempDir::new().unwrap();
    let mut config = ShardForgeConfig::default();
    config.cluster.data_directory = temp.path().join("data");
    config.storage.engine = StorageEngineType::Memory;

    let node = DatabaseNode::from_config(config.clone()).await.unwrap();
    node.execute_sql("CREATE TABLE t (id INTEGER)")
        .await
        .unwrap();
    assert_eq!(node.status().await.table_count, 1);

    let node2 = DatabaseNode::from_config(config).await.unwrap();
    assert_eq!(node2.status().await.table_count, 0);
}
