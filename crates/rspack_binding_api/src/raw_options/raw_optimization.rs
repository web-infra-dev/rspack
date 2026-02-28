use napi_derive::napi;
use rspack_core::{MangleExportsOption, Optimization, SideEffectOption, UsedExportsOption};

use super::WithBool;

#[derive(Debug, Default)]
#[napi(object, object_to_js = false)]
pub struct RawOptimizationOptions {
  pub remove_available_modules: bool,
  #[napi(ts_type = "boolean | string")]
  pub side_effects: WithBool<String>,
  #[napi(ts_type = "boolean | string")]
  pub used_exports: WithBool<String>,
  pub provided_exports: bool,
  pub inner_graph: bool,
  pub real_content_hash: bool,
  #[napi(ts_type = "boolean | string")]
  pub mangle_exports: WithBool<String>,
  pub inline_exports: bool,
  pub concatenate_modules: bool,
  pub avoid_entry_iife: bool,
}

macro_rules! impl_from_with_bool {
  ($ident:ident) => {
    impl From<WithBool<String>> for $ident {
      fn from(value: WithBool<String>) -> Self {
        match value {
          WithBool::True => Self::True,
          WithBool::False => Self::False,
          WithBool::Value(s) => Self::from(s.as_str()),
        }
      }
    }
  };
}

impl_from_with_bool!(UsedExportsOption);
impl_from_with_bool!(MangleExportsOption);
impl_from_with_bool!(SideEffectOption);

impl TryFrom<RawOptimizationOptions> for Optimization {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptimizationOptions) -> rspack_error::Result<Self> {
    Ok(Optimization {
      remove_available_modules: value.remove_available_modules,
      side_effects: value.side_effects.into(),
      provided_exports: value.provided_exports,
      used_exports: value.used_exports.into(),
      inner_graph: value.inner_graph,
      mangle_exports: value.mangle_exports.into(),
      inline_exports: value.inline_exports,
      concatenate_modules: value.concatenate_modules,
      avoid_entry_iife: value.avoid_entry_iife,
      real_content_hash: value.real_content_hash,
    })
  }
}
