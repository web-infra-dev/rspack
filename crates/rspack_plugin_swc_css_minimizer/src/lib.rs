use async_trait::async_trait;
use rayon::prelude::*;
use rspack_core::{rspack_sources::MapOptions, Plugin};
use rspack_error::Result;
use rspack_plugin_css::swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig};

#[derive(Debug)]
pub struct SwcCssMinimizerRspackPlugin;

#[async_trait]
impl Plugin for SwcCssMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.SwcCssMinimizerRspackPlugin"
  }

  // TODO: chunk hash

  async fn process_assets_stage_optimize_size(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let compilation = args.compilation;

    let gen_source_map_config = SwcCssSourceMapGenConfig {
      enable: compilation.options.devtool.source_map(),
      inline_sources_content: !compilation.options.devtool.no_sources(),
      emit_columns: !compilation.options.devtool.cheap(),
    };

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
          let minimized_source = SwcCssCompiler::default().minify(
            filename,
            input,
            input_source_map,
            gen_source_map_config.clone(),
          )?;
          original.set_source(Some(minimized_source));
        }
        original.get_info_mut().minimized = true;
        Ok(())
      })?;

    Ok(())
  }
}
