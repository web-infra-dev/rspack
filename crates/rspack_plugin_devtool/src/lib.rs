#![feature(let_chains)]

use std::{hash::Hash, path::Path};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap},
  AssetInfo, Compilation, CompilationAsset, JsChunkHashArgs, PathData, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginProcessAssetsOutput, PluginRenderModuleContentOutput,
  ProcessAssetsArgs, RenderModuleContentArgs, SourceType,
};
use rspack_error::{internal_error, Result};
use rspack_util::swc::normalize_custom_filename;
use rustc_hash::FxHashMap as HashMap;
use serde_json::json;

static IS_CSS_FILE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.css($|\?)").expect("TODO:"));

#[derive(Debug)]
pub struct DevtoolPluginOptions {
  pub inline: bool,
  pub append: bool,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
}

#[derive(Debug)]
pub struct DevtoolPlugin {
  inline: bool,
  source_mapping_url_comment: Option<String>,
  module_filename_template: String,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
}

impl DevtoolPlugin {
  pub fn new(options: DevtoolPluginOptions) -> Self {
    Self {
      inline: options.inline,
      source_mapping_url_comment: options
        .append
        .then(|| "# sourceMappingURL=[url]".to_string()),
      module_filename_template: "[resourcePath]".to_string(),
      namespace: options.namespace,
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
    }
  }
}

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

#[async_trait::async_trait]
impl Plugin for DevtoolPlugin {
  fn name(&self) -> &'static str {
    "devtool"
  }
  fn render_module_content(
    &self,
    _ctx: PluginContext,
    args: &RenderModuleContentArgs,
  ) -> PluginRenderModuleContentOutput {
    let devtool = &args.compilation.options.devtool;
    let origin_source = args.module_source.clone();
    if devtool.eval() && devtool.source_map() {
      if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
        return Ok(Some(cached.value().clone()));
      } else if let Some(map) = origin_source.map(&MapOptions::new(devtool.cheap())) {
        let source = wrap_eval_source_map(&origin_source.source(), map, args.compilation)?;
        MODULE_RENDER_CACHE.insert(origin_source, source.clone());
        return Ok(Some(source));
      }
    }
    Ok(Some(origin_source))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    args.compilation.options.devtool.hash(&mut args.hasher);
    Ok(())
  }

  async fn process_assets_stage_dev_tooling(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    if !args.compilation.options.devtool.source_map() || args.compilation.options.devtool.eval() {
      return Ok(());
    }
    let context = args.compilation.options.context.clone();
    let maps: HashMap<String, Vec<u8>> = args
      .compilation
      .assets_mut()
      .par_iter()
      .filter_map(|(filename, asset)| asset.get_source().map(|s| (filename, s)))
      .filter_map(|(filename, source)| {
        source.map(&MapOptions::new(self.columns)).map(|mut map| {
          map.set_file(Some(filename.clone()));
          for source in map.sources_mut() {
            let uri = normalize_custom_filename(source);
            let resource_path = if let Some(relative_path) = diff_paths(uri, &context) {
              relative_path.to_string_lossy().to_string()
            } else {
              uri.to_owned()
            };
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
            .map_err(|e| internal_error!(e.to_string()))?;
          Ok((filename.to_owned(), map_buffer))
        })
      })
      .collect::<Result<_>>()?;
    for (filename, map_buffer) in maps {
      let is_css = IS_CSS_FILE.is_match(&filename);
      let current_source_mapping_url_comment =
        self.source_mapping_url_comment.as_ref().map(|comment| {
          if is_css {
            format!("\n/*{comment}*/")
          } else {
            format!("\n//{comment}")
          }
        });
      if self.inline {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("DevToolPlugin: append can't be false when inline is true.");
        let base64 = rspack_base64::encode_to_string(&map_buffer);
        let mut asset = args
          .compilation
          .assets_mut()
          .remove(&filename)
          .unwrap_or_else(|| panic!("TODO:"));
        asset.source = Some(
          ConcatSource::new([
            asset.source.expect("source should never be `None` here, because `maps` is collected by asset with `Some(source)`"),
            RawSource::from(current_source_mapping_url_comment.replace(
              "[url]",
              &format!("data:application/json;charset=utf-8;base64,{base64}"),
            ))
            .boxed(),
          ])
          .boxed(),
        );
        args.compilation.emit_asset(filename, asset);
      } else {
        let mut source_map_filename = filename.to_owned() + ".map";
        // TODO(ahabhgk): refactor remove the for loop
        // https://webpack.docschina.org/configuration/output/#outputsourcemapfilename
        if args.compilation.options.devtool.source_map() {
          let source_map_filename_config = &args.compilation.options.output.source_map_filename;
          for chunk in args.compilation.chunk_by_ukey.values() {
            for file in &chunk.files {
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
                    .filename(Path::new(&filename))
                    .content_hash_optional(chunk.content_hash.get(source_type).map(|i| i.as_str())),
                );
                break;
              }
            }
          }
        }

        if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
          let source_map_url = if let Some(public_path) = &self.public_path {
            format!("{public_path}{source_map_filename}")
          } else if let Some(dirname) = Path::new(&filename).parent() && let Some(relative) = diff_paths(&source_map_filename, dirname) {
            relative.to_string_lossy().into_owned()
          } else {
            source_map_filename.clone()
          };
          let mut asset = args
            .compilation
            .assets_mut()
            .remove(&filename)
            .expect("TODO:");
          asset.source = Some(ConcatSource::new([
            asset.source.expect("source should never be `None` here, because `maps` is collected by asset with `Some(source)`"),
            RawSource::from(current_source_mapping_url_comment.replace("[url]", &source_map_url))
              .boxed(),
          ])
          .boxed());
          asset.info.related.source_map = Some(source_map_filename.clone());
          args.compilation.emit_asset(filename, asset);
        }
        let source_map_asset_info = AssetInfo::default().with_development(true);
        args.compilation.emit_asset(
          source_map_filename,
          CompilationAsset::new(
            Some(RawSource::from(map_buffer).boxed()),
            source_map_asset_info,
          ),
        );
      }
    }
    Ok(())
  }
}

pub fn wrap_eval_source_map(
  source: &str,
  mut map: SourceMap,
  compilation: &Compilation,
) -> Result<BoxSource> {
  for source in map.sources_mut() {
    let uri = normalize_custom_filename(source);
    *source = if let Some(relative_path) = diff_paths(uri, &*compilation.options.context) {
      relative_path.to_string_lossy().to_string()
    } else {
      uri.to_owned()
    };
  }
  if compilation.options.devtool.no_sources() {
    for content in map.sources_content_mut() {
      *content = String::default();
    }
  }
  let mut map_buffer = Vec::new();
  map
    .to_writer(&mut map_buffer)
    .map_err(|e| internal_error!(e.to_string()))?;
  let base64 = rspack_base64::encode_to_string(&map_buffer);
  let footer =
    format!("\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}");
  let result = RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed();
  Ok(result)
}
