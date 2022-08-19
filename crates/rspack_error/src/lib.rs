mod diagnostic;
mod error;
pub use diagnostic::*;
pub use error::*;
pub mod emitter;

pub type Result<T> = std::result::Result<T, Error>;
