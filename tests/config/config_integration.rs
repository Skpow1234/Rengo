//! Integration tests for configuration system

use rengo_config::*;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use tempfile::TempDir;
use tokio::fs;

/// Serialize tests that mutate process environment / cwd.
fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
}

fn clear_rengo_env_overrides() {
    for key in [
        "RENGO_CLUSTER__NAME",
        "RENGO_NODE__BIND_ADDRESS",
        "RENGO_STORAGE__BLOCK_CACHE_SIZE_MB",
    ] {
        env::remove_var(key);
    }
}

fn set_home(path: &Path) -> (Option<std::ffi::OsString>, Option<std::ffi::OsString>) {
    let prev_home = env::var_os("HOME");
    let prev_profile = env::var_os("USERPROFILE");
    env::set_var("HOME", path);
    env::set_var("USERPROFILE", path);
    (prev_home, prev_profile)
}

fn restore_home(prev_home: Option<std::ffi::OsString>, prev_profile: Option<std::ffi::OsString>) {
    match prev_home {
        Some(v) => env::set_var("HOME", v),
        None => env::remove_var("HOME"),
    }
    match prev_profile {
        Some(v) => env::set_var("USERPROFILE", v),
        None => env::remove_var("USERPROFILE"),
    }
}

/// Restores the previous working directory on drop (avoids leaking cwd across tests).
struct CwdGuard(PathBuf);

impl CwdGuard {
    fn enter(path: &Path) -> std::io::Result<Self> {
        let previous = env::current_dir()?;
        env::set_current_dir(path)?;
        Ok(Self(previous))
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.0);
    }
}

#[tokio::test]
async fn test_config_loader_hierarchical_loading() {
    let _guard = env_lock();
    clear_rengo_env_overrides();

    let temp_dir = TempDir::new().unwrap();
    let system_dir = temp_dir.path().join("etc");
    let home_dir = temp_dir.path().join("home");
    let local_dir = temp_dir.path().join("local");
    fs::create_dir_all(&system_dir).await.unwrap();
    fs::create_dir_all(home_dir.join(".rengo")).await.unwrap();
    fs::create_dir_all(&local_dir).await.unwrap();

    // System config: <search_path>/config.toml (lowest file precedence)
    let system_content = r#"
        [cluster]
        name = "system-cluster"

        [storage]
        engine = "rocksdb"
    "#;
    fs::write(system_dir.join("config.toml"), system_content).await.unwrap();

    // User config: $HOME/.rengo/config.toml
    let user_content = r#"
        [cluster]
        name = "user-cluster"

        [node]
        bind_address = "127.0.0.1:8080"
    "#;
    fs::write(home_dir.join(".rengo").join("config.toml"), user_content)
        .await
        .unwrap();

    // Local config: <search_path>/rengo.toml (highest file precedence)
    let local_content = r#"
        [cluster]
        name = "local-cluster"

        [monitoring]
        enabled = false
    "#;
    fs::write(local_dir.join("rengo.toml"), local_content).await.unwrap();

    let (prev_home, prev_profile) = set_home(&home_dir);

    // Enter local_dir so cwd `rengo.toml` is the local layer (not the repo root file)
    let _cwd = CwdGuard::enter(&local_dir).unwrap();
    let loader =
        ConfigLoader::new().with_search_paths(vec![system_dir.to_str().unwrap().to_string()]);

    let config = loader.load().await.unwrap();

    // Local file wins for overlapping keys; lower layers still contribute unique keys
    assert_eq!(config.cluster.name, "local-cluster");
    assert_eq!(config.node.bind_address, "127.0.0.1:8080"); // From user config
    assert_eq!(config.storage.engine, StorageEngineType::RocksDB); // From system config
    assert!(!config.monitoring.enabled); // From local config

    restore_home(prev_home, prev_profile);
}

#[tokio::test]
async fn test_config_loader_environment_variables() {
    let _guard = env_lock();
    let temp_dir = TempDir::new().unwrap();
    let _cwd = CwdGuard::enter(temp_dir.path()).unwrap();

    env::set_var("RENGO_CLUSTER__NAME", "env-cluster");
    env::set_var("RENGO_NODE__BIND_ADDRESS", "0.0.0.0:9999");
    env::set_var("RENGO_STORAGE__BLOCK_CACHE_SIZE_MB", "512");

    let loader = ConfigLoader::new().with_search_paths(vec![]);
    let config = loader.load().await.unwrap();

    assert_eq!(config.cluster.name, "env-cluster");
    assert_eq!(config.node.bind_address, "0.0.0.0:9999");
    assert_eq!(config.storage.block_cache_size_mb, 512);

    clear_rengo_env_overrides();
}

#[tokio::test]
async fn test_config_loader_tilde_expansion() {
    let _guard = env_lock();
    clear_rengo_env_overrides();

    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().join("home");
    fs::create_dir_all(&home_dir).await.unwrap();
    let (prev_home, prev_profile) = set_home(&home_dir);
    let home_str = home_dir.to_string_lossy();

    let config_content = format!(
        r#"
            [network]
            certificate_path = "~/certs/cert.pem"
            private_key_path = "~/certs/key.pem"
            ca_certificate_path = "~/certs/ca.pem"

            [logging]
            file_path = "~/logs/rengo.log"
        "#
    );

    let config_path = temp_dir.path().join("tilde.toml");
    fs::write(&config_path, config_content).await.unwrap();

    let loader = ConfigLoader::new().with_search_paths(vec![]);
    let config = loader.load_from_file(&config_path).await.unwrap();

    let expected_cert_path = format!("{}/certs/cert.pem", home_str);
    let expected_key_path = format!("{}/certs/key.pem", home_str);
    let expected_ca_path = format!("{}/certs/ca.pem", home_str);
    let expected_log_path = format!("{}/logs/rengo.log", home_str);

    // Compare via PathBuf so Windows separators still match
    assert_eq!(
        config.network.certificate_path.unwrap(),
        std::path::PathBuf::from(&expected_cert_path)
    );
    assert_eq!(
        config.network.private_key_path.unwrap(),
        std::path::PathBuf::from(&expected_key_path)
    );
    assert_eq!(
        config.network.ca_certificate_path.unwrap(),
        std::path::PathBuf::from(&expected_ca_path)
    );
    assert_eq!(config.logging.file_path.unwrap(), std::path::PathBuf::from(&expected_log_path));

    restore_home(prev_home, prev_profile);
}

#[tokio::test]
async fn test_config_validation_comprehensive() {
    let mut config = RengoConfig::default();

    // Valid configuration should pass
    assert!(ConfigValidator::validate(&config).is_ok());

    // Invalid bind address
    config.node.bind_address = "invalid-address".to_string();
    assert!(ConfigValidator::validate(&config).is_err());

    // Reset and test TLS without certificates
    config = RengoConfig::default();
    config.network.tls_enabled = true;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[tokio::test]
async fn test_config_round_trip() {
    let _guard = env_lock();
    clear_rengo_env_overrides();

    let mut original_config = RengoConfig::default();
    original_config.cluster.name = "round-trip-test".to_string();
    original_config.node.bind_address = "127.0.0.1:9999".to_string();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("round_trip.toml");

    let loader = ConfigLoader::new().with_search_paths(vec![]);
    loader.save_to_file(&original_config, &config_path).await.unwrap();

    let loaded_config = loader.load_from_file(&config_path).await.unwrap();

    assert_eq!(loaded_config.cluster.name, original_config.cluster.name);
    assert_eq!(loaded_config.node.bind_address, original_config.node.bind_address);
}

#[tokio::test]
async fn test_config_loader_nonexistent_files() {
    let _guard = env_lock();
    clear_rengo_env_overrides();

    // Empty cwd + empty search paths: no files found, defaults only
    let temp_dir = TempDir::new().unwrap();
    let _cwd = CwdGuard::enter(temp_dir.path()).unwrap();

    let loader = ConfigLoader::new().with_search_paths(vec![]);
    let config = loader.load().await.unwrap();
    assert_eq!(config.cluster.name, "default-cluster");
}

#[tokio::test]
async fn test_config_validation_edge_cases() {
    // Test minimum valid configuration
    let mut config = RengoConfig::default();
    config.storage.block_cache_size_mb = 1;
    config.storage.write_buffer_size_mb = 1;
    config.storage.max_write_buffer_number = 1;
    config.network.max_message_size_mb = 1;
    config.logging.max_file_size_mb = 1;
    config.logging.max_files = 1;

    assert!(ConfigValidator::validate(&config).is_ok());

    // Test zero values (should fail)
    config.storage.block_cache_size_mb = 0;
    assert!(ConfigValidator::validate(&config).is_err());

    config.storage.block_cache_size_mb = 1;
    config.network.max_message_size_mb = 0;
    assert!(ConfigValidator::validate(&config).is_err());
}
