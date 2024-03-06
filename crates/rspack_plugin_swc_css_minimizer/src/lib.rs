use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::{rspack_sources::MapOptions, Compilation, Plugin};
use rspack_error::Result;
use rspack_hook::AsyncSeries;
use rspack_plugin_css::swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig};

#[derive(Debug)]
pub struct SwcCssMinimizerRspackPlugin;

struct SwcCssMinimizerRspackPluginProcessAssetsHook;

#[async_trait]
impl AsyncSeries<Compilation> for SwcCssMinimizerRspackPluginProcessAssetsHook {
  async fn run(&self, compilation: &mut Compilation) -> Result<()> {
    compilation
      .assets_mut()
      .par_iter_mut()
      .filter(|(filename, _)| filename.ends_with(".css"))
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

  fn stage(&self) -> i32 {
    Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE
  }
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
      .tap(Box::new(SwcCssMinimizerRspackPluginProcessAssetsHook));
    Ok(())
  }

  // TODO: chunk hash
}
