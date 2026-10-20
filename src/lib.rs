//! # ShardForge Distributed Database
//!
//! A high-performance, distributed database system built in Rust, designed for
//! strong consistency, horizontal scalability, and PostgreSQL compatibility.
//!
//! ## Architecture
//!
//! ShardForge follows a layered architecture with clear separation of concerns:
//!
//! - **Core Layer**: Core types, error handling, and utilities
//! - **Storage Layer**: Pluggable storage engines and persistence
//! - **SQL Layer**: SQL parsing, query planning, and execution
//! - **Transaction Layer**: ACID transactions and concurrency control
//! - **Consensus Layer**: RAFT consensus and distributed coordination
//! - **Network Layer**: gRPC / TCP communication and client connections
//! - **Configuration Layer**: Hierarchical configuration management
//! - **Server Layer**: Database node orchestration (`DatabaseNode`)

#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]
#![deny(unsafe_code)]

// Re-export workspace crates
pub use shardforge_config as config;
pub use shardforge_core as core;
// Re-export commonly used types for convenience
pub use shardforge_core::*;
pub use shardforge_storage as storage;

// Local modules
pub mod network;
pub mod raft;
pub mod server;
pub mod sql;
pub mod transaction;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = "Development build";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_initialization() {
        assert!(!VERSION.is_empty());
        assert!(!BUILD_INFO.is_empty());
    }
}
