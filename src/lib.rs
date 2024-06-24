#![no_std]

pub(crate) mod macros;
pub(crate) use macros::*;

pub mod memory;
pub mod message;
pub mod settings;
pub mod spi;

pub use spi::ConfigError;
pub use spi::Error;
pub use spi::MCP2518FD;
