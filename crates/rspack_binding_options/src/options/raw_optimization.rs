use better_scoped_tls::scoped_tls;
use napi_derive::napi;
use rspack_core::{
  MangleExportsOption, Optimization, PluginExt, SideEffectOption, UsedExportsOption,
};
use rspack_error::internal_error;
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin,
};
use serde::Deserialize;

use crate::{RawOptionsApply, RawSplitChunksOptions};

scoped_tls!(pub(crate) static IS_ENABLE_NEW_SPLIT_CHUNKS: bool);

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub split_chunks: Option<RawSplitChunksOptions>,
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

impl RawOptionsApply for RawOptimizationOptions {
  type Options = Optimization;

  fn apply(
    self,
    plugins: &mut Vec<Box<dyn rspack_core::Plugin>>,
  ) -> Result<Self::Options, rspack_error::Error> {
    let chunk_ids_plugin = match self.chunk_ids.as_ref() {
      "named" => NamedChunkIdsPlugin::new(None, None).boxed(),
      "deterministic" => DeterministicChunkIdsPlugin::default().boxed(),
      _ => {
        return Err(internal_error!(
          "'chunk_ids' should be 'named' or 'deterministic'."
        ))
      }
    };
    plugins.push(chunk_ids_plugin);
    let module_ids_plugin = match self.module_ids.as_ref() {
      "named" => NamedModuleIdsPlugin::default().boxed(),
      "deterministic" => DeterministicModuleIdsPlugin::default().boxed(),
      _ => {
        return Err(internal_error!(
          "'module_ids' should be 'named' or 'deterministic'."
        ))
      }
    };
    plugins.push(module_ids_plugin);
    if self.real_content_hash {
      plugins.push(rspack_plugin_real_content_hash::RealContentHashPlugin.boxed());
    }
    Ok(Optimization {
      remove_available_modules: self.remove_available_modules,
      remove_empty_chunks: self.remove_empty_chunks,
      side_effects: SideEffectOption::from(self.side_effects.as_str()),
      provided_exports: self.provided_exports,
      used_exports: UsedExportsOption::from(self.used_exports.as_str()),
      inner_graph: self.inner_graph,
      mangle_exports: MangleExportsOption::from(self.mangle_exports.as_str()),
    })
  }
}
