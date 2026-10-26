mod typedef;

#[cfg(feature = "noop")]
mod noop;
#[cfg(feature = "noop")]
pub use self::noop::*;

#[cfg(not(feature = "noop"))]
mod napi;
#[cfg(not(feature = "noop"))]
pub use self::napi::*;
