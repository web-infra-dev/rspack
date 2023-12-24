#![feature(let_chains)]

use std::collections::HashSet;
use std::sync::Arc;
use std::{hash::Hash, path::Path};

use dashmap::DashMap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::{Captures, Regex};
use rspack_core::{
  contextify,
  rspack_sources::{BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap},
  AssetInfo, Compilation, CompilationAsset, JsChunkHashArgs, PathData, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginProcessAssetsOutput, PluginRenderModuleContentOutput,
  ProcessAssetsArgs, RenderModuleContentArgs, SourceType,
};
use rspack_core::{ChunkGraph, Context, Filename, Logger, Module, ModuleIdentifier};
use rspack_error::miette::IntoDiagnostic;
use rspack_error::{Error, Result};
use rspack_util::identifier::make_paths_absolute;
use rspack_util::swc::normalize_custom_filename;
use rustc_hash::FxHashMap as HashMap;
use serde_json::json;

static CSS_EXTENSION_DETECT_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css($|\?)").expect("Failed to compile CSS_EXTENSION_DETECT_REGEXP"));

static REGEXP_ALL_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[all-?loaders\]\[resource\]").expect("Failed to compile SQUARE_BRACKET_TAG_REGEXP")
});
static SQUARE_BRACKET_TAG_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[\\*([\w-]+)\\*\]").expect("Failed to compile SQUARE_BRACKET_TAG_REGEXP")
});
static REGEXP_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[loaders\]\[resource\]").expect("Failed to compile SQUARE_BRACKET_TAG_REGEXP")
});

pub struct ModuleFilenameTemplateFnCtx {
  pub identifier: String,
  pub short_identifier: String,
  pub resource: String,
  pub resource_path: String,
  pub absolute_resource_path: String,
  pub loaders: String,
  pub all_loaders: String,
  pub query: String,
  pub module_id: String, // TODO: string | number
  pub hash: String,
  pub namespace: String,
}

type ModuleFilenameTemplateFn =
  Arc<dyn for<'a> Fn(ModuleFilenameTemplateFnCtx) -> String + Send + Sync>;

pub enum ModuleFilenameTemplate {
  String(String),
  Fn(ModuleFilenameTemplateFn),
}

pub enum SourceOrModule {
  Source(String),
  Module(ModuleIdentifier),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPluginOptions {
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap.
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  pub filename: Option<String>,
  pub append: Option<bool>,
  pub namespace: String,
  pub columns: bool,
  pub no_sources: bool,
  pub public_path: Option<String>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPlugin {
  filename: Option<Filename>,
  source_mapping_url_comment: Option<String>,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
}

impl SourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    let module_filename_template =
      options
        .module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resourcePath]".to_string(),
        ));

    Self {
      filename: options.filename.map(Filename::from),
      source_mapping_url_comment: (!matches!(options.append, Some(false)))
        .then(|| "# sourceMappingURL=[url]".to_string()),
      module_filename_template,
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
    ProcessAssetsArgs { compilation }: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("collect source maps");
    let context = compilation.options.context.clone();

    let maps: HashMap<String, (Vec<u8>, Option<Vec<u8>>)> = compilation
      .assets()
      .par_iter()
      .filter_map(|(filename, asset)| asset.get_source().map(|s| (filename, s)))
      .map(|(filename, source)| {
        let map = source
          .map(&MapOptions::new(self.columns))
          .map(|mut map| {
            map.set_file(Some(filename.clone()));
            for source in map.sources_mut() {
              let module_or_source = if source.starts_with("webpack://") {
                ModuleOrSource::Source(source.clone())
              } else {
                let source = make_paths_absolute(context.as_str(), &source[10..]);
                // TODO: how to use source to find module
                let identifier = ModuleIdentifier::from(source.clone());
                match compilation.module_graph.module_by_identifier(&identifier) {
                  Some(module) => ModuleOrSource::Module(module.as_ref()),
                  None => ModuleOrSource::Source(source),
                }
              };
              *source = self.create_filename(module_or_source, &context, &compilation.chunk_graph);
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
      let mut asset = compilation
        .assets_mut()
        .remove(&filename)
        .expect("should have filename in compilation.assets");
      // convert to RawSource to reduce one time source map calculation when convert to JsCompatSource
      let raw_source = RawSource::from(code_buffer).boxed();
      let Some(map_buffer) = map_buffer else {
        asset.source = Some(raw_source);
        compilation.emit_asset(filename, asset);
        continue;
      };
      let is_css = CSS_EXTENSION_DETECT_REGEXP.is_match(&filename);
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
        for chunk in compilation.chunk_by_ukey.values() {
          let files: HashSet<String> = chunk.files.union(&chunk.auxiliary_files).cloned().collect();

          for file in &files {
            if file == &filename {
              let source_type = if is_css {
                &SourceType::Css
              } else {
                &SourceType::JavaScript
              };
              source_map_filename = compilation.get_asset_path(
                source_map_filename_config,
                PathData::default()
                  .chunk(chunk)
                  .filename(&filename)
                  .content_hash_optional(
                    chunk
                      .content_hash
                      .get(source_type)
                      .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
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
        compilation.emit_asset(filename.clone(), asset);
        let mut source_map_asset_info = AssetInfo::default().with_development(true);
        if let Some(asset) = compilation.assets().get(&filename) {
          // set source map asset version to be the same as the target asset
          source_map_asset_info.version = asset.info.version.clone();
        }
        compilation.emit_asset(
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
        compilation.emit_asset(filename, asset);
        // TODO
        // chunk.auxiliary_files.add(filename);
      }
    }
    logger.time_end(start);
    Ok(())
  }
}

enum ModuleOrSource<'a> {
  Module(&'a dyn Module),
  Source(String),
}

fn get_before(s: &str, token: &str) -> String {
  match s.rfind(token) {
    Some(idx) => s[..idx].to_string(),
    None => "".to_string(),
  }
}

fn get_after(s: &str, token: &str) -> String {
  s.find(token)
    .map(|idx| s[idx..].to_string())
    .unwrap_or("".to_string())
}

impl SourceMapDevToolPlugin {
  fn create_filename(
    &self,
    module_or_source: ModuleOrSource,
    context: &Context,
    chunk_graph: &ChunkGraph,
  ) -> String {
    let ctx = match module_or_source {
      ModuleOrSource::Module(module) => {
        let module_identifier = module.identifier();

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(&context, &module_identifier);
        let module_id = chunk_graph
          .get_module_id(module_identifier)
          .clone()
          .unwrap_or("".to_string());
        // TODO: get resource from module
        let absolute_resource_path = "".to_string();

        let resource = short_identifier
          .clone()
          .split('!')
          .last()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&identifier, "!");

        let q = query.len();
        let resource_path = if q == 0 {
          resource.clone()
        } else {
          resource[..resource.len().saturating_sub(q)].to_string()
        };

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id,
          absolute_resource_path,
          hash: "".to_string(),
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: self.namespace.clone(),
        }
      }
      ModuleOrSource::Source(source) => {
        let short_identifier = contextify(&context, &source);
        let identifier = short_identifier.clone();

        let resource = short_identifier
          .clone()
          .split('!')
          .last()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&identifier, "!");

        let q = query.len();
        let resource_path = if q == 0 {
          resource.clone()
        } else {
          resource[..resource.len().saturating_sub(q)].to_string()
        };

        ModuleFilenameTemplateFnCtx {
          short_identifier,
          identifier,
          module_id: "".to_string(),
          absolute_resource_path: source.split('!').last().unwrap_or("").to_string(),
          hash: "".to_string(),
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: self.namespace.clone(),
        }
      }
    };

    return match &self.module_filename_template {
      ModuleFilenameTemplate::Fn(f) => f(ctx),
      ModuleFilenameTemplate::String(s) => {
        let mut replacements: HashMap<String, &str> = HashMap::default();
        replacements.insert("identifier".to_string(), &ctx.identifier);
        replacements.insert("short-identifier".to_string(), &ctx.short_identifier);
        replacements.insert("resource".to_string(), &ctx.resource);

        replacements.insert("resource-path".to_string(), &ctx.resource_path);
        replacements.insert("resourcepath".to_string(), &ctx.resource_path);

        replacements.insert(
          "absolute-resource-path".to_string(),
          &ctx.absolute_resource_path,
        );
        replacements.insert("abs-resource-path".to_string(), &ctx.absolute_resource_path);
        replacements.insert(
          "absoluteresource-path".to_string(),
          &ctx.absolute_resource_path,
        );
        replacements.insert("absresource-path".to_string(), &ctx.absolute_resource_path);
        replacements.insert(
          "absolute-resourcepath".to_string(),
          &ctx.absolute_resource_path,
        );
        replacements.insert("abs-resourcepath".to_string(), &ctx.absolute_resource_path);
        replacements.insert(
          "absoluteresourcepath".to_string(),
          &ctx.absolute_resource_path,
        );
        replacements.insert("absresourcepath".to_string(), &ctx.absolute_resource_path);

        replacements.insert("all-loaders".to_string(), &ctx.all_loaders);
        replacements.insert("allloaders".to_string(), &ctx.all_loaders);

        replacements.insert("query".to_string(), &ctx.query);
        replacements.insert("id".to_string(), &ctx.module_id);
        replacements.insert("hash".to_string(), &ctx.hash);
        replacements.insert("namespace".to_string(), &ctx.namespace);

        let s = REGEXP_ALL_LOADERS_RESOURCE.replace_all(s, "[identifier]");
        let s = REGEXP_LOADERS_RESOURCE.replace_all(&s, "[short-identifier]");
        SQUARE_BRACKET_TAG_REGEXP
          .replace_all(&s, |caps: &Captures| {
            let full_match = caps.get(0).unwrap().as_str();
            let content = caps.get(1).unwrap().as_str();

            if content.len() + 2 == full_match.len() {
              if let Some(replacement) = replacements.get(&content.to_lowercase()) {
                replacement.to_string()
              } else {
                full_match.to_string()
              }
            } else if full_match.starts_with("[\\") && full_match.ends_with("\\]") {
              format!("[{}]", &full_match[2..full_match.len() - 2])
            } else {
              full_match.to_string()
            }
          })
          .to_string()
      }
    };
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
