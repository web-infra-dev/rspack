mod dependency;
pub mod scope_info;
pub mod semicolon;

pub(crate) use self::dependency::StatementPath;
pub use self::{dependency::*, scope_info::*};
