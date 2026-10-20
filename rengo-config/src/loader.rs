//! Configuration loading and management

use std::path::{Path, PathBuf};

use rengo_core::Result;

use crate::RengoConfig;

/// Configuration loader with hierarchical loading support
pub struct ConfigLoader {
    /// Search paths for configuration files
    search_paths: Vec<String>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            search_paths: vec!["/etc/rengo".to_string(), "~/.rengo".to_string(), ".".to_string()],
        }
    }

    pub fn with_search_paths(mut self, paths: Vec<String>) -> Self {
        self.search_paths = paths;
        self
    }

    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest precedence)
    /// 2. Local configuration file (`rengo.toml` in cwd or search paths)
    /// 3. User configuration (`~/.rengo/config.toml`)
    /// 4. System configuration (`config.toml` in search paths)
    /// 5. Default values (lowest precedence)
    pub async fn load(&self) -> Result<RengoConfig> {
        let mut builder = config::Config::builder();

        // Start with default configuration
        builder = builder
            .add_source(config::File::from_str(&self.default_config(), config::FileFormat::Toml));

        // Add system-wide configuration (first matching config.toml in search paths)
        if let Some(path) = self.find_config_file("config.toml") {
            builder = builder.add_source(config::File::from(path));
        }

        // Add user configuration
        if let Some(path) = self.find_user_config_file("config.toml") {
            builder = builder.add_source(config::File::from(path));
        }

        // Add local configuration (cwd first, then search paths last-to-first)
        if let Some(path) = self.find_local_config_file() {
            builder = builder.add_source(config::File::from(path));
        }

        // Environment variables: RENGO_CLUSTER__NAME -> cluster.name
        builder = builder.add_source(Self::environment_source());

        // Build configuration
        let config = builder.build().map_err(|e| rengo_core::RengoError::Config {
            message: format!("Failed to build configuration: {}", e),
        })?;

        // Deserialize into our config structure
        let mut rengo_config: RengoConfig =
            config.try_deserialize().map_err(|e| rengo_core::RengoError::Config {
                message: format!("Failed to deserialize configuration: {}", e),
            })?;

        // Post-process configuration
        self.post_process(&mut rengo_config)?;

        Ok(rengo_config)
    }

    /// Load configuration from a specific file
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<RengoConfig> {
        let mut builder = config::Config::builder();

        // Start with default configuration
        builder = builder
            .add_source(config::File::from_str(&self.default_config(), config::FileFormat::Toml));

        // Add the specified file
        builder = builder.add_source(config::File::from(path.as_ref()));

        // Add environment variables (same nesting rules as load())
        builder = builder.add_source(Self::environment_source());

        let config = builder.build().map_err(|e| rengo_core::RengoError::Config {
            message: format!("Failed to build configuration: {}", e),
        })?;

        let mut rengo_config: RengoConfig =
            config.try_deserialize().map_err(|e| rengo_core::RengoError::Config {
                message: format!("Failed to deserialize configuration: {}", e),
            })?;

        self.post_process(&mut rengo_config)?;

        Ok(rengo_config)
    }

    /// Save current configuration to a file
    pub async fn save_to_file<P: AsRef<Path>>(&self, config: &RengoConfig, path: P) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(config).map_err(|e| rengo_core::RengoError::Config {
                message: format!("Failed to serialize configuration: {}", e),
            })?;

        tokio::fs::write(path, toml_string)
            .await
            .map_err(|e| rengo_core::RengoError::Config {
                message: format!("Failed to write configuration file: {}", e),
            })?;

        Ok(())
    }

    fn environment_source() -> config::Environment {
        config::Environment::with_prefix("RENGO")
            .prefix_separator("_")
            .separator("__")
            .try_parsing(true)
    }

    fn default_config(&self) -> String {
        toml::to_string_pretty(&RengoConfig::default()).expect("Failed to serialize default config")
    }

    /// Resolve the user home directory (HOME on Unix, USERPROFILE on Windows).
    fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }

    fn expand_search_path(&self, search_path: &str) -> Option<PathBuf> {
        if let Some(rest) = search_path.strip_prefix("~/") {
            let mut path = Self::home_dir()?;
            path.push(rest);
            Some(path)
        } else if search_path == "~" {
            Self::home_dir()
        } else {
            Some(PathBuf::from(search_path))
        }
    }

    fn find_config_file(&self, filename: &str) -> Option<PathBuf> {
        for search_path in &self.search_paths {
            let mut path = self.expand_search_path(search_path)?;
            path.push(filename);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    fn find_user_config_file(&self, filename: &str) -> Option<PathBuf> {
        let mut path = Self::home_dir()?;
        path.push(".rengo");
        path.push(filename);
        path.exists().then_some(path)
    }

    /// Local project config: `./rengo.toml`, else `rengo.toml` under a search path
    /// (later search paths win).
    fn find_local_config_file(&self) -> Option<PathBuf> {
        let cwd = PathBuf::from("rengo.toml");
        if cwd.exists() {
            return Some(cwd);
        }

        for search_path in self.search_paths.iter().rev() {
            if let Some(mut path) = self.expand_search_path(search_path) {
                path.push("rengo.toml");
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }

    fn post_process(&self, config: &mut RengoConfig) -> Result<()> {
        // Auto-generate node ID if not provided
        if config.cluster.node_id.is_none() {
            config.cluster.node_id = Some(rengo_core::NodeId::new().to_string());
        }

        // Expand tilde in paths
        if let Some(home) = Self::home_dir() {
            let home_str = home.to_string_lossy();

            if let Some(cert_path) = &config.network.certificate_path {
                if cert_path.starts_with("~") {
                    config.network.certificate_path =
                        Some(cert_path.to_string_lossy().replacen("~", &home_str, 1).into());
                }
            }

            if let Some(key_path) = &config.network.private_key_path {
                if key_path.starts_with("~") {
                    config.network.private_key_path =
                        Some(key_path.to_string_lossy().replacen("~", &home_str, 1).into());
                }
            }

            if let Some(ca_path) = &config.network.ca_certificate_path {
                if ca_path.starts_with("~") {
                    config.network.ca_certificate_path =
                        Some(ca_path.to_string_lossy().replacen("~", &home_str, 1).into());
                }
            }

            if let Some(log_path) = &config.logging.file_path {
                if log_path.starts_with("~") {
                    config.logging.file_path =
                        Some(log_path.to_string_lossy().replacen("~", &home_str, 1).into());
                }
            }
        }

        // Validate configuration
        self.validate_config(config)?;

        Ok(())
    }

    fn validate_config(&self, config: &RengoConfig) -> Result<()> {
        // Validate network configuration
        if config.network.tls_enabled {
            if config.network.certificate_path.is_none() {
                return Err(rengo_core::RengoError::Config {
                    message: "TLS enabled but certificate path not specified".to_string(),
                });
            }
            if config.network.private_key_path.is_none() {
                return Err(rengo_core::RengoError::Config {
                    message: "TLS enabled but private key path not specified".to_string(),
                });
            }
        }

        // Validate storage configuration
        if config.storage.block_cache_size_mb == 0 {
            return Err(rengo_core::RengoError::Config {
                message: "Block cache size must be greater than 0".to_string(),
            });
        }

        // Validate network addresses
        if config.node.bind_address.parse::<std::net::SocketAddr>().is_err() {
            return Err(rengo_core::RengoError::Config {
                message: format!("Invalid bind address: {}", config.node.bind_address),
            });
        }

        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use tempfile::TempDir;

    use super::*;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
    }

    #[tokio::test]
    async fn test_load_default_config() {
        let _guard = env_lock();
        std::env::remove_var("RENGO_CLUSTER__NAME");

        let loader = ConfigLoader::new();
        let config = loader.load().await.unwrap();

        assert_eq!(config.cluster.name, "default-cluster");
        assert_eq!(config.node.bind_address, "127.0.0.1:5432");
        assert!(!config.network.tls_enabled);
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let _guard = env_lock();
        std::env::remove_var("RENGO_CLUSTER__NAME");

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config_content = r#"
            [cluster]
            name = "test-cluster"

            [node]
            bind_address = "127.0.0.1:8080"
        "#;

        tokio::fs::write(&config_path, config_content).await.unwrap();

        let loader = ConfigLoader::new();
        let config = loader.load_from_file(&config_path).await.unwrap();

        assert_eq!(config.cluster.name, "test-cluster");
        assert_eq!(config.node.bind_address, "127.0.0.1:8080");
    }

    #[tokio::test]
    async fn test_environment_override() {
        let _guard = env_lock();
        std::env::set_var("RENGO_CLUSTER__NAME", "from-env");

        let loader = ConfigLoader::new();
        let config = loader.load().await.unwrap();
        assert_eq!(config.cluster.name, "from-env");

        std::env::remove_var("RENGO_CLUSTER__NAME");
    }
}
