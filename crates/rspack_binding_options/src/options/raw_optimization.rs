use better_scoped_tls::scoped_tls;
use napi_derive::napi;
use rspack_core::{Optimization, PluginExt, SideEffectOption};
use rspack_error::internal_error;
use rspack_ids::{DeterministicModuleIdsPlugin, NamedModuleIdsPlugin};
use rspack_plugin_split_chunks::SplitChunksPlugin;
use serde::Deserialize;

use crate::JsLoaderRunner;
use crate::{RawOptionsApply, RawSplitChunksOptions};

scoped_tls!(pub(crate) static IS_ENABLE_NEW_SPLIT_CHUNKS: bool);

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub split_chunks: Option<RawSplitChunksOptions>,
  pub module_ids: String,
  pub remove_available_modules: bool,
  pub remove_empty_chunks: bool,
  pub side_effects: String,
  pub real_content_hash: bool,
}

impl RawOptionsApply for RawOptimizationOptions {
  type Options = Optimization;

  fn apply(
    self,
    plugins: &mut Vec<Box<dyn rspack_core::Plugin>>,
    _: &JsLoaderRunner,
  ) -> Result<Self::Options, rspack_error::Error> {
    if let Some(options) = self.split_chunks {
      let split_chunks_plugin = IS_ENABLE_NEW_SPLIT_CHUNKS.with(|is_enable_new_split_chunks| {
        if *is_enable_new_split_chunks {
          rspack_plugin_split_chunks_new::SplitChunksPlugin::new(options.into()).boxed()
        } else {
          SplitChunksPlugin::new(options.into()).boxed()
        }
      });

      plugins.push(split_chunks_plugin);
    }
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
    })
  }
}
