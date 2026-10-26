#[cfg(all(feature = "type-def", not(feature = "noop")))]
mod type_def;
#[cfg(all(feature = "type-def", not(feature = "noop")))]
pub use self::type_def::*;

#[cfg(not(feature = "type-def"))]
pub mod noop;
#[cfg(not(feature = "type-def"))]
pub use self::noop::*;
