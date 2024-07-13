mod swc_css_compiler;

use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{rspack_sources::MapOptions, Compilation, CompilationProcessAssets, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;
use rspack_util::try_any_sync;
use swc_css_compiler::{SwcCssCompiler, SwcCssSourceMapGenConfig};

static CSS_ASSET_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css(\?.*)?$").expect("Invalid RegExp"));

#[derive(Debug, Default)]
pub struct SwcCssMinimizerRspackPluginOptions {
  pub test: Option<SwcCssMinimizerRules>,
  pub include: Option<SwcCssMinimizerRules>,
  pub exclude: Option<SwcCssMinimizerRules>,
}


#[plugin]
#[derive(Debug, Default)]
pub struct SwcCssMinimizerRspackPlugin {
  options: SwcCssMinimizerRspackPluginOptions,
}

impl SwcCssMinimizerRspackPlugin {
  pub fn new(options: SwcCssMinimizerRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationProcessAssets for SwcCssMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let minify_options = &self.options;

  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, original)| {
      if !CSS_ASSET_REGEXP.is_match(filename) {
        return false;
      }

      let is_matched = match_object(minify_options, filename).unwrap_or(false);

      if !is_matched || original.get_info().minimized {
        return false;
      }

      true
    })
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

#[derive(Debug, Clone, Hash)]
pub enum SwcCssMinimizerRule {
  String(String),
  Regexp(RspackRegex),
}

impl SwcCssMinimizerRule {
  pub fn try_match(&self, data: &str) -> rspack_error::Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum SwcCssMinimizerRules {
  String(String),
  Regexp(rspack_regex::RspackRegex),
  Array(Vec<SwcCssMinimizerRule>),
}

impl SwcCssMinimizerRules {
  pub fn try_match(&self, data: &str) -> rspack_error::Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
      Self::Array(l) => try_any_sync(l, |i| i.try_match(data)),
    }
  }
}

pub fn match_object(obj: &SwcCssMinimizerRspackPluginOptions, str: &str) -> Result<bool> {
  if let Some(condition) = &obj.test {
    if !condition.try_match(str)? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.include {
    if !condition.try_match(str)? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.exclude {
    if condition.try_match(str)? {
      return Ok(false);
    }
  }
  Ok(true)
}
