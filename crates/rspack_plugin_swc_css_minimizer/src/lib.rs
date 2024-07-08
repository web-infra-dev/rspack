mod swc_css_compiler;

use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{rspack_sources::MapOptions, Compilation, CompilationProcessAssets, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig};

static CSS_ASSET_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css(\?.*)?$").expect("Invalid RegExp"));

#[plugin]
#[derive(Debug, Default)]
pub struct SwcCssMinimizerRspackPlugin;

#[plugin_hook(CompilationProcessAssets for SwcCssMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, _)| CSS_ASSET_REGEXP.is_match(filename))
    .try_for_each(|(filename, original)| -> Result<()> {
      if original.get_info().minimized {
        return Ok(());
      }

      if let Some(original_source) = original.get_source() {
        let input = original_source.source().to_string();
        let input_source_map = original_source.map(&MapOptions::default());
        let enable_source_map = input_source_map.is_some();
        let minimized_source = SwcCssCompiler::default().minify(
          filename,
          input,
          input_source_map,
          SwcCssSourceMapGenConfig {
            enable: enable_source_map,
            inline_sources_content: false,
            emit_columns: true,
          },
        )?;
        original.set_source(Some(minimized_source));
      }
      original.get_info_mut().minimized = true;
      Ok(())
    })?;

  Ok(())
}

impl Plugin for SwcCssMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.SwcCssMinimizerRspackPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }

  // TODO: chunk hash
}
