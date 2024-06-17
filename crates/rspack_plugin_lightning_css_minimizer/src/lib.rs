#![feature(let_chains)]

use std::{
  collections::HashSet,
  hash::Hash,
  sync::{Arc, RwLock},
};

use lightningcss::{
  printer::PrinterOptions,
  stylesheet::{MinifyOptions, ParserFlags, ParserOptions, StyleSheet},
  targets::{Browsers, Targets},
};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{
    MapOptions, RawSource, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
  },
  ChunkUkey, Compilation, CompilationChunkHash, CompilationProcessAssets, Plugin,
};
use rspack_error::{error, Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};

static CSS_ASSET_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css(\?.*)?$").expect("Invalid RegExp"));

#[derive(Debug, Hash)]
pub struct LightningCssMinimizerOptions {
  pub error_recovery: bool,
  pub unused_symbols: Vec<String>,
  pub remove_unused_local_idents: bool,
  pub browserlist: Vec<String>,
}

#[plugin]
#[derive(Debug)]
pub struct LightningCssMinimizerRspackPlugin {
  options: LightningCssMinimizerOptions,
}

impl LightningCssMinimizerRspackPlugin {
  pub fn new(options: LightningCssMinimizerOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationChunkHash for LightningCssMinimizerRspackPlugin)]
fn chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  self.options.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for LightningCssMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let all_warnings: RwLock<Vec<_>> = Default::default();
  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, _)| CSS_ASSET_REGEXP.is_match(filename))
    .try_for_each(|(filename, original)| -> Result<()> {
      if original.get_info().minimized {
        return Ok(());
      }

      if let Some(original_source) = original.get_source() {
        let input = original_source.source().into_owned();
        let input_source_map = original_source.map(&MapOptions::default());
        let mut source_map = input_source_map
          .clone()
          .map(|input_source_map| -> Result<_> {
            let mut map =
              parcel_sourcemap::SourceMap::new(input_source_map.source_root().unwrap_or_default());
            map
              .add_vlq_map(
                input_source_map.mappings().as_bytes(),
                // TODO: move instead of clone
                input_source_map.sources().to_vec(),
                input_source_map.sources_content().to_vec(),
                input_source_map.names().to_vec(),
                0,
                0,
              )
              .map_err(|e| error!(e.to_string()))?;
            Ok(map)
          })
          .transpose()?;
        let result = {
          let warnings: Arc<RwLock<Vec<_>>> = Default::default();
          let mut stylesheet = StyleSheet::parse(
            &input,
            ParserOptions {
              filename: filename.to_string(),
              css_modules: None,
              source_index: 0,
              error_recovery: self.options.error_recovery,
              warnings: Some(warnings.clone()),
              flags: ParserFlags::all(),
            },
          )
          .map_err(|e| error!(e.to_string()))?;
          let targets = Targets::from(
            Browsers::from_browserslist(&self.options.browserlist)
              .map_err(|e| error!(e.to_string()))?,
          );
          let mut unused_symbols = HashSet::from_iter(self.options.unused_symbols.clone());
          if self.options.remove_unused_local_idents
            && let Some(css_unused_idents) = original.info.css_unused_idents.take()
          {
            unused_symbols.extend(css_unused_idents);
          }
          stylesheet
            .minify(MinifyOptions {
              targets,
              unused_symbols,
            })
            .map_err(|e| error!(e.to_string()))?;
          let result = stylesheet
            .to_css(PrinterOptions {
              minify: true,
              source_map: source_map.as_mut(),
              project_root: None,
              targets,
              analyze_dependencies: None,
              pseudo_classes: None,
            })
            .map_err(|e| error!(e.to_string()))?;
          let warnings = warnings.read().expect("should lock");
          all_warnings.write().expect("should lock").extend(
            warnings
              .iter()
              .map(|e| Diagnostic::error("Css minimize error".to_string(), e.to_string())),
          );
          result
        };

        let minimized_source = if let Some(mut source_map) = source_map {
          SourceMapSource::new(SourceMapSourceOptions {
            value: result.code,
            name: filename,
            source_map: SourceMap::from_json(
              &source_map
                .to_json(None)
                .map_err(|e| error!(e.to_string()))?,
            )
            .expect("should be able to generate source-map"),
            original_source: Some(input),
            inner_source_map: input_source_map,
            remove_original_source: true,
          })
          .boxed()
        } else {
          RawSource::from(result.code).boxed()
        };

        original.set_source(Some(minimized_source));
      }
      original.get_info_mut().minimized = true;
      Ok(())
    })?;

  compilation.extend_diagnostics(all_warnings.into_inner().expect("should lock"));

  Ok(())
}

impl Plugin for LightningCssMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.LightningCssMinimizerRspackPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .chunk_hash
      .tap(chunk_hash::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
