// Core modules
pub mod core;
pub mod accounting;
pub mod storage;
pub mod server;
pub mod config;
pub mod demo;

// Re-exports for convenience
pub use core::{Account, Transfer, TransferFlags};
pub use accounting::{DoubleEntryEngine, TransferResult, AccountingError, StorageBackend};
pub use storage::InMemoryStorage;
