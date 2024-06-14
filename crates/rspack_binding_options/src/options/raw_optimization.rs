use napi_derive::napi;
use rspack_core::{MangleExportsOption, Optimization, SideEffectOption, UsedExportsOption};

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub side_effects: String,
  pub used_exports: String,
  pub provided_exports: bool,
  pub inner_graph: bool,
  pub mangle_exports: String,
  pub concatenate_modules: bool,
}

impl TryFrom<RawOptimizationOptions> for Optimization {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptimizationOptions) -> rspack_error::Result<Self> {
    Ok(Optimization {
      side_effects: SideEffectOption::from(value.side_effects.as_str()),
      provided_exports: value.provided_exports,
      used_exports: UsedExportsOption::from(value.used_exports.as_str()),
      inner_graph: value.inner_graph,
      mangle_exports: MangleExportsOption::from(value.mangle_exports.as_str()),
      concatenate_modules: value.concatenate_modules,
    })
  }
}
