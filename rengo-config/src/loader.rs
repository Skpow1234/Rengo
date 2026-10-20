//! Configuration loading and management

use std::path::Path;

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
            search_paths: vec![
                "/etc/rengo".to_string(),
                "~/.rengo".to_string(),
                ".".to_string(),
            ],
        }
    }

    pub fn with_search_paths(mut self, paths: Vec<String>) -> Self {
        self.search_paths = paths;
        self
    }

    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest precedence)
    /// 2. Local configuration file (rengo.toml)
    /// 3. User configuration (~/.rengo/config.toml)
    /// 4. System configuration (/etc/rengo/config.toml)
    /// 5. Default values (lowest precedence)
    pub async fn load(&self) -> Result<RengoConfig> {
        let mut builder = config::Config::builder();

        // Start with default configuration
        builder = builder
            .add_source(config::File::from_str(&self.default_config(), config::FileFormat::Toml));

        // Add system-wide configuration
        if let Some(path) = self.find_config_file("config.toml") {
            builder = builder.add_source(config::File::with_name(path.to_str().unwrap()));
        }

        // Add user configuration
        if let Some(path) = self.find_user_config_file("config.toml") {
            builder = builder.add_source(config::File::with_name(path.to_str().unwrap()));
        }

        // Add local configuration
        if Path::new("rengo.toml").exists() {
            builder = builder.add_source(config::File::with_name("rengo.toml"));
        }

        // Add environment variables with RENGO_ prefix
        builder = builder.add_source(
            config::Environment::with_prefix("RENGO").separator("_").try_parsing(true),
        );

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
        builder = builder.add_source(config::File::with_name(path.as_ref().to_str().unwrap()));

        // Add environment variables
        builder = builder.add_source(
            config::Environment::with_prefix("RENGO").separator("_").try_parsing(true),
        );

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
    pub async fn save_to_file<P: AsRef<Path>>(
        &self,
        config: &RengoConfig,
        path: P,
    ) -> Result<()> {
        let toml_string = toml::to_string_pretty(config).map_err(|e| {
            rengo_core::RengoError::Config {
                message: format!("Failed to serialize configuration: {}", e),
            }
        })?;

        tokio::fs::write(path, toml_string).await.map_err(|e| {
            rengo_core::RengoError::Config {
                message: format!("Failed to write configuration file: {}", e),
            }
        })?;

        Ok(())
    }

    fn default_config(&self) -> String {
        toml::to_string_pretty(&RengoConfig::default())
            .expect("Failed to serialize default config")
    }

    fn find_config_file(&self, filename: &str) -> Option<std::path::PathBuf> {
        for search_path in &self.search_paths {
            let expanded_path = if search_path.starts_with('~') {
                if let Some(home) = std::env::var_os("HOME") {
                    let mut path = std::path::PathBuf::from(home);
                    path.push(&search_path[2..]); // Remove ~/
                    path.push(filename);
                    path
                } else {
                    continue;
                }
            } else {
                let mut path = std::path::PathBuf::from(search_path);
                path.push(filename);
                path
            };

            if expanded_path.exists() {
                return Some(expanded_path);
            }
        }
        None
    }

    fn find_user_config_file(&self, filename: &str) -> Option<std::path::PathBuf> {
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = std::path::PathBuf::from(home);
            path.push(".rengo");
            path.push(filename);

            if path.exists() {
                return Some(path);
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
        if let Some(home) = std::env::var_os("HOME") {
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
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_load_default_config() {
        let loader = ConfigLoader::new();
        let config = loader.load().await.unwrap();

        assert_eq!(config.cluster.name, "default-cluster");
        assert_eq!(config.node.bind_address, "127.0.0.1:5432");
        assert!(!config.network.tls_enabled);
    }

    #[tokio::test]
    async fn test_load_from_file() {
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
}
