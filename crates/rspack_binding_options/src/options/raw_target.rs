use rspack_core::Target;

use crate::JsLoaderRunner;
use crate::RawOptionsApply;

pub type RawTarget = Vec<String>;

impl RawOptionsApply for RawTarget {
  type Options = Target;

  fn apply(
    self,
    _: &mut Vec<rspack_core::BoxPlugin>,
    _: &JsLoaderRunner,
  ) -> Result<Self::Options, rspack_error::Error> {
    Ok(Target::new(&self)?)
  }
}
