mod dependency;
pub mod scope_info;

pub(crate) use self::dependency::{
  StatementPath, is_this_free_function, is_this_free_function_expression,
};
pub use self::{dependency::*, scope_info::*};
