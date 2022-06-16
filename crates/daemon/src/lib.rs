pub mod client;
pub mod compile;
pub mod constants;
pub mod daemon;
pub mod error;
pub mod nvim;
pub mod run;
pub mod state;
pub mod store;
pub mod types;
pub mod util;
pub mod watch;
pub mod xcodegen;
pub use error::{CompileError, Error, LoopError};
pub type Result<T> = std::result::Result<T, Error>;
