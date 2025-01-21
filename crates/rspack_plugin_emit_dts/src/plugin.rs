use std::sync::{Arc, RwLock};

use rspack_core::{
  rspack_sources::RawSource, AssetInfo, Compilation, CompilationAsset, CompilationFinishModules,
  CompilationProcessAssets, Logger, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;

use crate::SwcDtsEmitRspackPluginOptions;

#[derive(Debug, Clone)]
struct DtsOutput {
  filename: String,
  dts_filename: String,
  dts_code: String,
}

#[plugin]
#[derive(Debug)]
pub struct SwcDtsEmitRspackPlugin {
  pub(crate) options: Arc<SwcDtsEmitRspackPluginOptions>,
  dts_outputs: Arc<RwLock<Vec<DtsOutput>>>,
}

impl Eq for SwcDtsEmitRspackPlugin {}

impl PartialEq for SwcDtsEmitRspackPlugin {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.options, &other.options)
  }
}

const PLUGIN_NAME: &str = "rspack.SwcDtsEmitPlugin";

impl SwcDtsEmitRspackPlugin {
  pub fn new(options: SwcDtsEmitRspackPluginOptions) -> Self {
    Self::new_inner(Arc::new(options), Arc::new(RwLock::new(Vec::new())))
  }
}

#[plugin_hook(CompilationFinishModules for SwcDtsEmitRspackPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.SwcDtsEmitRspackPlugin");
  let start = logger.time("run dts collect in finishModules");
  let module_graph = compilation.get_module_graph();
  let mut dts_outputs = self
    .dts_outputs
    .write()
    .expect("failed to write dts_outputs");

  for (_, module) in module_graph.modules() {
    let Some(build_info) = &module.build_info() else {
      continue;
    };
    let parse_meta = &build_info.parse_meta;
    let Some(filename) = parse_meta.get("swc-dts-emit-plugin-filename") else {
      continue;
    };
    let Some(dts_filename) = parse_meta.get("swc-dts-emit-plugin-dts-filename") else {
      continue;
    };
    let Some(dts_code) = parse_meta.get("swc-dts-emit-plugin-dts-code") else {
      continue;
    };

    dts_outputs.push(DtsOutput {
      filename: filename.to_string(),
      dts_filename: dts_filename.to_string(),
      dts_code: dts_code.to_string(),
    });
  }
  logger.time_end(start);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for SwcDtsEmitRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_DERIVED)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.SwcDtsEmitRspackPlugin");
  let start = logger.time("run dts emit in processAssets");

  let dts_outputs = self.dts_outputs.read().expect("failed to read dts_outputs");

  let extension = self.options.extension.clone();

  for dts_output in dts_outputs.iter() {
    let DtsOutput {
      dts_code,
      dts_filename,
      filename,
    } = dts_output;

    let asset_info = AssetInfo {
      source_filename: Some(filename.to_string()),
      ..Default::default()
    };

    compilation.emit_asset(
      Utf8PathBuf::from(dts_filename)
        .with_extension(&extension)
        .to_string(),
      CompilationAsset {
        source: Some(Arc::new(RawSource::from(dts_code.as_str()))),
        info: asset_info,
      },
    );
  }
  logger.time_end(start);
  Ok(())
}

impl Plugin for SwcDtsEmitRspackPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}
