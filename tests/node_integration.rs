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

#[tokio::test]
async fn test_session_transactions() {
    let temp = TempDir::new().unwrap();
    let mut config = ShardForgeConfig::default();
    config.cluster.data_directory = temp.path().join("data");
    config.storage.engine = StorageEngineType::Memory;

    let node = DatabaseNode::from_config(config).await.unwrap();
    node.execute_sql("CREATE TABLE acct (id INTEGER, bal INTEGER)")
        .await
        .unwrap();

    node.execute_sql("BEGIN").await.unwrap();
    assert!(node.status().await.in_transaction);
    node.execute_sql("INSERT INTO acct (id, bal) VALUES (1, 100)")
        .await
        .unwrap();
    node.execute_sql("COMMIT").await.unwrap();
    assert!(!node.status().await.in_transaction);

    let rows = node.execute_sql("SELECT id FROM acct").await.unwrap();
    assert_eq!(rows.rows.len(), 1);
}

#[cfg(feature = "sled")]
#[tokio::test]
async fn test_sled_catalog_and_data_survive_restart() {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().join("sled-data");

    let mut config = ShardForgeConfig::default();
    config.cluster.data_directory = data_dir.clone();
    config.storage.engine = StorageEngineType::Sled;
    config.node.bind_address = "127.0.0.1:0".to_string();

    {
        let node = DatabaseNode::from_config(config.clone())
            .await
            .expect("open sled node");
        assert_eq!(node.status().await.storage_engine, "sled");

        node.execute_sql("CREATE TABLE users (id INTEGER NOT NULL, name VARCHAR(64))")
            .await
            .expect("create");
        node.execute_sql("INSERT INTO users (id, name) VALUES (1, 'alice')")
            .await
            .expect("insert");
        node.execute_sql("INSERT INTO users (id, name) VALUES (2, 'bob')")
            .await
            .expect("insert2");

        // Flush by dropping the node (closes storage)
        assert_eq!(node.status().await.table_count, 1);
        drop(node);
    }

    // Re-open same data directory
    let node2 = DatabaseNode::from_config(config)
        .await
        .expect("reopen sled node");
    assert_eq!(node2.status().await.table_count, 1);

    let result = node2
        .execute_sql("SELECT id, name FROM users")
        .await
        .expect("select after restart");
    assert_eq!(result.rows.len(), 2);
}
