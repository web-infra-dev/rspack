use better_scoped_tls::scoped_tls;
use napi_derive::napi;
use rspack_core::{MangleExportsOption, Optimization, SideEffectOption, UsedExportsOption};
use serde::Deserialize;

scoped_tls!(pub(crate) static IS_ENABLE_NEW_SPLIT_CHUNKS: bool);

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub module_ids: String,
  pub chunk_ids: String,
  pub remove_available_modules: bool,
  pub remove_empty_chunks: bool,
  pub side_effects: String,
  pub used_exports: String,
  pub provided_exports: bool,
  pub inner_graph: bool,
  pub real_content_hash: bool,
  pub mangle_exports: String,
}

impl TryFrom<RawOptimizationOptions> for Optimization {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptimizationOptions) -> rspack_error::Result<Self> {
    Ok(Optimization {
      remove_available_modules: value.remove_available_modules,
      remove_empty_chunks: value.remove_empty_chunks,
      side_effects: SideEffectOption::from(value.side_effects.as_str()),
      provided_exports: value.provided_exports,
      used_exports: UsedExportsOption::from(value.used_exports.as_str()),
      inner_graph: value.inner_graph,
      mangle_exports: MangleExportsOption::from(value.mangle_exports.as_str()),
    })
  }
}
