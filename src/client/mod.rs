pub mod h3_client;
pub mod pool;

// Re-export public types
pub use pool::{ConnectionPoolState, ErrorStats, ResponseResult};