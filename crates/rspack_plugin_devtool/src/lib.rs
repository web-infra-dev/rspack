#![feature(let_chains)]

use std::collections::HashSet;
use std::{hash::Hash, path::Path};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  contextify,
  rspack_sources::{BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap},
  AssetInfo, Compilation, CompilationAsset, JsChunkHashArgs, PathData, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginProcessAssetsOutput, PluginRenderModuleContentOutput,
  ProcessAssetsArgs, RenderModuleContentArgs, SourceType,
};
use rspack_core::{Filename, Logger};
use rspack_error::miette::IntoDiagnostic;
use rspack_error::{Error, Result};
use rspack_util::swc::normalize_custom_filename;
use rustc_hash::FxHashMap as HashMap;
use serde_json::json;

static IS_CSS_FILE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.css($|\?)").expect("TODO:"));

#[derive(Debug)]
pub struct SourceMapDevToolPluginOptions {
  pub filename: Option<String>,
  pub append: Option<bool>,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
}

#[derive(Debug)]
pub struct SourceMapDevToolPlugin {
  filename: Option<Filename>,
  source_mapping_url_comment: Option<String>,
  module_filename_template: String,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
}

impl SourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    Self {
      filename: options.filename.map(|f| Filename::from(f)),
      source_mapping_url_comment: (!matches!(options.append, Some(false)))
        .then(|| "# sourceMappingURL=[url]".to_string()),
      module_filename_template: "[resourcePath]".to_string(),
      namespace: options.namespace,
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for SourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    "rspack.SourceMapDevToolPlugin"
  }

  async fn process_assets_stage_dev_tooling(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let logger = args.compilation.get_logger(self.name());
    let start = logger.time("collect source maps");
    let context = args.compilation.options.context.clone();
    let maps: HashMap<String, (Vec<u8>, Option<Vec<u8>>)> = args
      .compilation
      .assets_mut()
      .par_iter()
      .filter_map(|(filename, asset)| asset.get_source().map(|s| (filename, s)))
      .map(|(filename, source)| {
        let map = source
          .map(&MapOptions::new(self.columns))
          .map(|mut map| {
            map.set_file(Some(filename.clone()));
            for source in map.sources_mut() {
              let resource_path = normalize_custom_filename(source);
              let resource_path = contextify(&context, resource_path);
              *source = self
                .module_filename_template
                .replace("[namespace]", &self.namespace)
                .replace("[resourcePath]", &resource_path);
            }
            if self.no_sources {
              for content in map.sources_content_mut() {
                *content = String::default();
              }
            }
            let mut map_buffer = Vec::new();
            map
              .to_writer(&mut map_buffer)
              .unwrap_or_else(|e| panic!("{}", e.to_string()));
            Ok::<Vec<u8>, Error>(map_buffer)
          })
          .transpose()?;
        let mut code_buffer = Vec::new();
        source.to_writer(&mut code_buffer).into_diagnostic()?;
        Ok((filename.to_owned(), (code_buffer, map)))
      })
      .collect::<Result<_>>()?;
    logger.time_end(start);

    let start = logger.time("emit source map assets");
    for (filename, (code_buffer, map_buffer)) in maps {
      let mut asset = args
        .compilation
        .assets_mut()
        .remove(&filename)
        .expect("should have filename in compilation.assets");
      // convert to RawSource to reduce one time source map calculation when convert to JsCompatSource
      let raw_source = RawSource::from(code_buffer).boxed();
      let Some(map_buffer) = map_buffer else {
        asset.source = Some(raw_source);
        args.compilation.emit_asset(filename, asset);
        continue;
      };
      let is_css = IS_CSS_FILE.is_match(&filename);
      let current_source_mapping_url_comment =
        self.source_mapping_url_comment.as_ref().map(|comment| {
          if is_css {
            format!("\n/*{comment}*/")
          } else {
            format!("\n//{comment}")
          }
        });
      if let Some(source_map_filename_config) = &self.filename {
        let mut source_map_filename = filename.to_owned() + ".map";
        // TODO(ahabhgk): refactor remove the for loop
        for chunk in args.compilation.chunk_by_ukey.values() {
          let files: HashSet<String> = chunk.files.union(&chunk.auxiliary_files).cloned().collect();

          for file in &files {
            if file == &filename {
              let source_type = if is_css {
                &SourceType::Css
              } else {
                &SourceType::JavaScript
              };
              source_map_filename = args.compilation.get_asset_path(
                source_map_filename_config,
                PathData::default()
                  .chunk(chunk)
                  .filename(&filename)
                  .content_hash_optional(
                    chunk
                      .content_hash
                      .get(source_type)
                      .map(|i| i.rendered(args.compilation.options.output.hash_digest_length)),
                  ),
              );
              break;
            }
          }
        }

        if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
          let source_map_url = if let Some(public_path) = &self.public_path {
            format!("{public_path}{source_map_filename}")
          } else if let Some(dirname) = Path::new(&filename).parent()
            && let Some(relative) = diff_paths(&source_map_filename, dirname)
          {
            relative.to_string_lossy().into_owned()
          } else {
            source_map_filename.clone()
          };
          asset.source = Some(
            ConcatSource::new([
              raw_source,
              RawSource::from(current_source_mapping_url_comment.replace("[url]", &source_map_url))
                .boxed(),
            ])
            .boxed(),
          );
          asset.info.related.source_map = Some(source_map_filename.clone());
        } else {
          asset.source = Some(raw_source);
        }
        args.compilation.emit_asset(filename.clone(), asset);
        let mut source_map_asset_info = AssetInfo::default().with_development(true);
        if let Some(asset) = args.compilation.assets().get(&filename) {
          // set source map asset version to be the same as the target asset
          source_map_asset_info.version = asset.info.version.clone();
        }
        args.compilation.emit_asset(
          source_map_filename,
          CompilationAsset::new(
            Some(RawSource::from(map_buffer).boxed()),
            source_map_asset_info,
          ),
        );
      } else {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("SourceMapDevToolPlugin: append can't be false when no filename is provided.");
        let base64 = rspack_base64::encode_to_string(&map_buffer);
        asset.source = Some(
          ConcatSource::new([
            raw_source,
            RawSource::from(current_source_mapping_url_comment.replace(
              "[url]",
              &format!("data:application/json;charset=utf-8;base64,{base64}"),
            ))
            .boxed(),
          ])
          .boxed(),
        );
        args.compilation.emit_asset(filename, asset);
        // TODO
        // chunk.auxiliary_files.add(filename);
      }
    }
    logger.time_end(start);
    Ok(())
  }
}

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

#[derive(Debug)]
pub struct EvalSourceMapDevToolPlugin {
  columns: bool,
  no_sources: bool,
}

impl EvalSourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    Self {
      columns: options.columns,
      no_sources: options.no_sources,
    }
  }

  pub fn wrap_eval_source_map(
    &self,
    source: &str,
    mut map: SourceMap,
    compilation: &Compilation,
  ) -> Result<BoxSource> {
    for source in map.sources_mut() {
      let resource_path = normalize_custom_filename(source);
      let resource_path = contextify(&compilation.options.context, resource_path);
      *source = resource_path;
    }
    if self.no_sources {
      for content in map.sources_content_mut() {
        *content = String::default();
      }
    }
    let mut map_buffer = Vec::new();
    map
      .to_writer(&mut map_buffer)
      .unwrap_or_else(|e| panic!("{}", e.to_string()));
    let base64 = rspack_base64::encode_to_string(&map_buffer);
    let footer =
      format!("\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}");
    let result = RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed();
    Ok(result)
  }
}

#[async_trait::async_trait]
impl Plugin for EvalSourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    "rspack.EvalSourceMapDevToolPlugin"
  }

  fn render_module_content<'a>(
    &'a self,
    _ctx: PluginContext,
    mut args: RenderModuleContentArgs<'a>,
  ) -> PluginRenderModuleContentOutput<'a> {
    let origin_source = args.module_source.clone();
    if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
      args.module_source = cached.value().clone();
      return Ok(args);
    } else if let Some(map) = origin_source.map(&MapOptions::new(self.columns)) {
      let source = self.wrap_eval_source_map(&origin_source.source(), map, args.compilation)?;
      MODULE_RENDER_CACHE.insert(origin_source, source.clone());
      args.module_source = source;
      return Ok(args);
    }
    Ok(args)
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    Ok(())
  }
}
