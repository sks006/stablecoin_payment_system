// shared_memory/src/lib.rs

pub mod error;
pub mod instructions;
pub mod state;

// Re-export core layouts for zero-copy downstream access
pub use error::*;
pub use instructions::*;
pub use state::*;