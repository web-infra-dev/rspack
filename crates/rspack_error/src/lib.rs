mod diagnostic;
mod error;
pub use diagnostic::*;
pub use error::*;

pub type Result<T> = std::result::Result<T, Error>;
