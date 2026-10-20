//! ShardForge Command Line Interface

use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use shardforge::core::Result;
use shardforge::server::DatabaseNode;

#[derive(Parser)]
#[command(name = "shardforge")]
#[command(about = "ShardForge Distributed Database")]
#[command(version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ShardForge instance
    Init {
        /// Data directory path
        #[arg(short, long, default_value = "./data")]
        data_dir: String,

        /// Single-node mode (no clustering)
        #[arg(long)]
        single_node: bool,
    },

    /// Start the ShardForge database server
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "shardforge.toml")]
        config: String,
    },

    /// Show database status
    Status {
        /// Show detailed status
        #[arg(long)]
        detailed: bool,

        /// Configuration file path
        #[arg(short, long, default_value = "shardforge.toml")]
        config: String,
    },

    /// Execute SQL queries
    Sql {
        /// SQL query to execute
        query: Option<String>,

        /// Database name
        #[arg(short, long, default_value = "default")]
        database: String,

        /// Configuration file path
        #[arg(short, long, default_value = "shardforge.toml")]
        config: String,
    },

    /// Show version information
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init { data_dir, single_node }) => handle_init(&data_dir, single_node).await,
        Some(Commands::Start { config }) => handle_start(&config).await,
        Some(Commands::Status { detailed, config }) => handle_status(detailed, &config).await,
        Some(Commands::Sql { query, database, config }) => {
            handle_sql(query.as_deref(), &database, &config).await
        }
        Some(Commands::Version) => handle_version().await,
        None => {
            let _ = Cli::command().print_help();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn handle_init(data_dir: &str, single_node: bool) -> Result<()> {
    println!("Initializing ShardForge...");
    println!("Data directory: {}", data_dir);
    println!("Single node: {}", single_node);

    std::fs::create_dir_all(data_dir).map_err(|e| {
        shardforge::core::ShardForgeError::internal(format!(
            "Failed to create data directory: {}",
            e
        ))
    })?;

    let config_path = std::path::Path::new("shardforge.toml");
    if !config_path.exists() {
        let default_config = format!(
            r#"
[cluster]
name = "default-cluster"
data_directory = "{data_dir}"

[node]
name = "shardforge-node"
bind_address = "127.0.0.1:5432"
max_connections = 100
connection_timeout_sec = 30

[storage]
engine = "memory"
block_cache_size_mb = 256
write_buffer_size_mb = 64
max_write_buffer_number = 4
compression = "lz4"
enable_statistics = true

[network]
tls_enabled = false
max_message_size_mb = 4
connection_pool_size = 10
keep_alive_interval_sec = 60

[logging]
level = "info"
format = "pretty"
max_file_size_mb = 100
max_files = 5

[monitoring]
enabled = true
bind_address = "127.0.0.1:9090"
tracing_enabled = false
service_name = "shardforge"
"#,
            data_dir = data_dir.replace('\\', "/")
        );

        std::fs::write(config_path, default_config.trim_start()).map_err(|e| {
            shardforge::core::ShardForgeError::internal(format!(
                "Failed to create config file: {}",
                e
            ))
        })?;
        println!("Created default configuration at: {}", config_path.display());
    }

    println!("ShardForge initialized successfully!");
    println!("Start the database with: shardforge start --config shardforge.toml");
    Ok(())
}

async fn handle_start(config_path: &str) -> Result<()> {
    println!("Starting ShardForge...");
    println!("Configuration: {}", config_path);

    let node = DatabaseNode::from_config_file(config_path).await?;
    let status = node.status().await;
    println!(
        "Opened storage engine '{}' with {} table(s)",
        status.storage_engine, status.table_count
    );

    node.start().await
}

async fn handle_status(detailed: bool, config_path: &str) -> Result<()> {
    println!("ShardForge Status");
    println!("================");

    if std::path::Path::new(config_path).exists() {
        match DatabaseNode::from_config_file(config_path).await {
            Ok(node) => {
                let status = node.status().await;
                println!("Version: {}", status.version);
                println!("Config: {}", config_path);
                println!("Bind: {}", status.bind_address);
                println!("Storage: {}", status.storage_engine);
                println!("Tables: {}", status.table_count);
                println!(
                    "Server: {}",
                    if status.running {
                        "running"
                    } else {
                        "not listening (use `shardforge start`)"
                    }
                );
                if detailed {
                    println!("Build: {}", shardforge::BUILD_INFO);
                }
            }
            Err(e) => {
                println!("Version: {}", shardforge::VERSION);
                println!("Config load error: {}", e);
            }
        }
    } else {
        println!("Version: {}", shardforge::VERSION);
        println!("Config: {} (not found)", config_path);
        println!("Hint: run `shardforge init` first");
    }

    Ok(())
}

async fn handle_sql(query: Option<&str>, database: &str, config_path: &str) -> Result<()> {
    println!("SQL Execution");
    println!("=============");
    println!("Database: {}", database);

    let Some(q) = query else {
        println!("Interactive mode: use `shardforge sql \"SELECT 1\"`");
        println!("Or connect to a running node with newline-delimited JSON on the bind port.");
        return Ok(());
    };

    println!("Query: {}", q);

    let node = DatabaseNode::from_config_file(config_path).await?;
    let result = node.execute_sql(q).await?;

    if !result.columns.is_empty() {
        println!("{}", result.columns.join(" | "));
        println!("{}", "-".repeat(result.columns.join(" | ").len().max(8)));
        for row in &result.rows {
            let cells: Vec<String> = row
                .iter()
                .map(|v| String::from_utf8_lossy(v.as_slice()).into_owned())
                .collect();
            println!("{}", cells.join(" | "));
        }
    }
    println!("Affected rows: {}", result.affected_rows);
    Ok(())
}

async fn handle_version() -> Result<()> {
    println!("ShardForge {}", shardforge::VERSION);
    println!("Build: {}", shardforge::BUILD_INFO);
    println!("Phase: 1 (Foundation) — single-node DatabaseNode online");
    Ok(())
}
