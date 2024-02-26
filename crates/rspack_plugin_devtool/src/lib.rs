#![feature(let_chains)]

use std::borrow::Cow;
use std::hash::Hasher;
use std::sync::{Arc, RwLock};
use std::{hash::Hash, path::Path};

use dashmap::DashMap;
use derivative::Derivative;
use futures::future::{join_all, BoxFuture};
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
use rspack_core::{
  Chunk, Filename, Logger, Module, ModuleIdentifier, OutputOptions, RuntimeModule,
};
use rspack_error::miette::IntoDiagnostic;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHash};
use rspack_util::identifier::make_paths_absolute;
use rspack_util::source_map::SourceMapKind;
use rspack_util::{path::relative, swc::normalize_custom_filename};
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use serde_json::json;

static CSS_EXTENSION_DETECT_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css($|\?)").expect("failed to compile CSS_EXTENSION_DETECT_REGEXP"));
static URL_FORMATTING_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\n\/\/(.*)$").expect("failed to compile URL_FORMATTING_REGEXP regex"));

static REGEXP_ALL_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[all-?loaders\]\[resource\]").expect("failed to compile SQUARE_BRACKET_TAG_REGEXP")
});
static SQUARE_BRACKET_TAG_REGEXP: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[\\*([\w-]+)\\*\]").expect("failed to compile SQUARE_BRACKET_TAG_REGEXP")
});
static REGEXP_LOADERS_RESOURCE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"\[loaders\]\[resource\]").expect("failed to compile SQUARE_BRACKET_TAG_REGEXP")
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
  pub module_id: String,
  pub hash: String,
  pub namespace: String,
}

type ModuleFilenameTemplateFn =
  Box<dyn Fn(ModuleFilenameTemplateFnCtx) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub enum ModuleFilenameTemplate {
  String(String),
  Fn(ModuleFilenameTemplateFn),
}

pub struct ModuleFilenameHelpers;

type AppendFn = Arc<dyn for<'a> Fn() -> Option<String> + Send + Sync>;

pub enum Append {
  String(String),
  Fn(AppendFn),
  Disabled,
}

pub type TestFn = Box<dyn Fn(String) -> bool + Sync + Send>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPluginOptions {
  // Appends the given value to the original asset. Usually the #sourceMappingURL comment. [url] is replaced with a URL to the source map file. false disables the appending.
  #[derivative(Debug = "ignore")]
  pub append: Option<Append>,
  // Indicates whether column mappings should be used (defaults to true).
  pub columns: bool,
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap used only if 'moduleFilenameTemplate' would result in a conflict.
  #[derivative(Debug = "ignore")]
  pub fallback_module_filename_template: Option<ModuleFilenameTemplate>,
  // Path prefix to which the [file] placeholder is relative to.
  pub file_context: Option<String>,
  // Defines the output filename of the SourceMap (will be inlined if no value is provided).
  pub filename: Option<String>,
  // Indicates whether SourceMaps from loaders should be used (defaults to true).
  pub module: bool,
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap.
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  // Namespace prefix to allow multiple webpack roots in the devtools.
  pub namespace: Option<String>,
  // Omit the 'sourceContents' array from the SourceMap.
  pub no_sources: bool,
  // Provide a custom public path for the SourceMapping comment.
  pub public_path: Option<String>,
  // Provide a custom value for the 'sourceRoot' property in the SourceMap.
  pub source_root: Option<String>,
  // Include or exclude source maps for modules based on their extension (defaults to .js and .css).
  #[derivative(Debug = "ignore")]
  pub test: Option<TestFn>,
}

enum SourceMappingUrlComment {
  String(String),
  Fn(AppendFn),
}

pub enum ModuleOrSource {
  Source(String),
  Module(ModuleIdentifier),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPlugin {
  source_map_filename: Option<Filename>,
  #[derivative(Debug = "ignore")]
  source_mapping_url_comment: Option<SourceMappingUrlComment>,
  file_context: Option<String>,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  #[derivative(Debug = "ignore")]
  fallback_module_filename_template: ModuleFilenameTemplate,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
  module: bool,
  source_root: Option<String>,
  #[derivative(Debug = "ignore")]
  test: Option<TestFn>,
  source_and_map_assets_cache:
    RwLock<HashMap<String, (u64, CompilationAsset, Option<(String, CompilationAsset)>)>>,
}

impl SourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    let source_mapping_url_comment = match options.append {
      Some(append) => match append {
        Append::String(s) => Some(SourceMappingUrlComment::String(s)),
        Append::Fn(f) => Some(SourceMappingUrlComment::Fn(f)),
        Append::Disabled => None,
      },
      None => Some(SourceMappingUrlComment::String(
        "\n//# sourceMappingURL=[url]".to_string(),
      )),
    };

    let fallback_module_filename_template =
      options
        .fallback_module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resourcePath]?[hash]".to_string(),
        ));

    let module_filename_template =
      options
        .module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resourcePath]".to_string(),
        ));

    Self {
      source_map_filename: options.filename.map(Filename::from),
      source_mapping_url_comment,
      fallback_module_filename_template,
      file_context: options.file_context,
      module_filename_template,
      namespace: options.namespace.unwrap_or("".to_string()),
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
      module: options.module,
      source_root: options.source_root,
      test: options.test,
      source_and_map_assets_cache: RwLock::new(HashMap::default()),
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
    let output_options = &compilation.options.output;

    let compilation_assets = compilation.assets();
    let mut source_and_map_asstes: Vec<(
      String,
      u64,
      CompilationAsset,
      Option<(String, CompilationAsset)>,
    )> = Vec::with_capacity(compilation_assets.len());
    let mut recompute_assets = vec![];

    {
      let cache = self.source_and_map_assets_cache.read().unwrap();
      for (file, asset) in compilation.assets().iter() {
        let source = asset.get_source();
        if let Some(source) = source {
          if let Some((cached_hash, cached_source_asset, cached_map_asset)) = cache.get(file) {
            let mut hasher = RspackHash::new(&HashFunction::MD4);
            source.update_hash(&mut hasher);
            let hash = hasher.finish();
            if hash == *cached_hash {
              source_and_map_asstes.push((
                file.to_owned(),
                hash,
                cached_source_asset.clone(),
                cached_map_asset.clone(),
              ));
              continue;
            }
          }
          recompute_assets.push((file, asset, source));
        }
      }
    }

    let assets = compilation
      .assets()
      .par_iter()
      .filter_map(|(file, asset)| {
        let is_match = match &self.test {
          Some(test) => test(file.clone()),
          None => true,
        };

        if is_match {
          asset.get_source().map(|source| {
            let map_options = MapOptions::new(self.columns);
            let source_map = source.map(&map_options);
            (file, source, source_map)
          })
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    let mut used_names_set = HashSet::<String>::default();
    let mut maps: Vec<(String, u64, Vec<u8>, Option<Vec<u8>>)> = Vec::with_capacity(assets.len());

    let mut default_filenames = match &self.module_filename_template {
      ModuleFilenameTemplate::String(s) => assets
        .iter()
        .filter_map(|(_file, _asset, source_map)| source_map.as_ref())
        .flat_map(|source_map| source_map.sources())
        .collect::<Vec<_>>()
        .par_iter()
        .map(|source| {
          let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
            let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
            let identifier = ModuleIdentifier::from(source.clone());
            match compilation.module_graph.module_by_identifier(&identifier) {
              Some(module) => ModuleOrSource::Module(module.identifier()),
              None => ModuleOrSource::Source(source),
            }
          } else {
            ModuleOrSource::Source(source.to_string())
          };
          Some((
            ModuleFilenameHelpers::create_filename_of_string_template(
              &module_or_source,
              compilation,
              s,
              output_options,
              self.namespace.as_str(),
            ),
            module_or_source,
          ))
        })
        .collect::<Vec<_>>(),
      ModuleFilenameTemplate::Fn(f) => {
        let features = assets
          .iter()
          .filter_map(|(_file, _asset, source_map)| source_map.as_ref())
          .flat_map(|source_map| source_map.sources())
          .map(|source| async {
            let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
              let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
              let identifier = ModuleIdentifier::from(source.clone());
              match compilation.module_graph.module_by_identifier(&identifier) {
                Some(module) => ModuleOrSource::Module(module.identifier()),
                None => ModuleOrSource::Source(source),
              }
            } else {
              ModuleOrSource::Source(source.to_string())
            };

            let filename = ModuleFilenameHelpers::create_filename_of_fn_template(
              &module_or_source,
              compilation,
              f,
              output_options,
              self.namespace.as_str(),
            )
            .await;

            match filename {
              Ok(filename) => Ok(Some((filename, module_or_source))),
              Err(err) => Err(err),
            }
          })
          .collect::<Vec<_>>();
        join_all(features)
          .await
          .into_iter()
          .collect::<Result<Vec<_>>>()?
      }
    };
    let mut default_filenames_index = 0;

    for (file, asset, source_map) in assets {
      let source_map_buffer = match source_map {
        Some(mut source_map) => {
          source_map.set_file(Some(file.clone()));

          let sources = source_map.sources_mut();
          for source in sources {
            let (source_name, module_or_source) = default_filenames[default_filenames_index]
              .take()
              .expect("expected a filename at the given index but found None");
            default_filenames_index += 1;

            let mut has_name = used_names_set.contains(&source_name);
            if !has_name {
              used_names_set.insert(source_name.clone());
              *source = Cow::from(source_name);
              continue;
            }

            // Try the fallback name first
            let mut source_name = match &self.fallback_module_filename_template {
              ModuleFilenameTemplate::String(s) => {
                ModuleFilenameHelpers::create_filename_of_string_template(
                  &module_or_source,
                  compilation,
                  s,
                  output_options,
                  self.namespace.as_str(),
                )
              }
              ModuleFilenameTemplate::Fn(f) => {
                ModuleFilenameHelpers::create_filename_of_fn_template(
                  &module_or_source,
                  compilation,
                  f,
                  output_options,
                  self.namespace.as_str(),
                )
                .await?
              }
            };

            has_name = used_names_set.contains(&source_name);
            if !has_name {
              used_names_set.insert(source_name.clone());
              *source = Cow::from(source_name);
              continue;
            }

            // Otherwise, append stars until we have a valid name
            while has_name {
              source_name.push('*');
              has_name = used_names_set.contains(&source_name);
            }
            used_names_set.insert(source_name.clone());
            *source = Cow::from(source_name);
          }
          if self.no_sources {
            for content in source_map.sources_content_mut() {
              *content = Cow::from(String::default());
            }
          }
          if let Some(source_root) = &self.source_root {
            source_map.set_source_root(Some(source_root.clone()));
          }
          let mut source_map_buffer = Vec::new();
          source_map
            .to_writer(&mut source_map_buffer)
            .unwrap_or_else(|e| panic!("{}", e.to_string()));
          Some(source_map_buffer)
        }
        None => None,
      };

      let mut hasher = RspackHash::new(&HashFunction::MD4);
      asset.update_hash(&mut hasher);
      let hash = hasher.finish();

      let mut code_buffer = Vec::new();
      asset.to_writer(&mut code_buffer).into_diagnostic()?;
      maps.push((file.to_owned(), hash, code_buffer, source_map_buffer));
    }

    logger.time_end(start);

    let start = logger.time("emit source map assets");
    for (filename, hash, code_buffer, source_map_buffer) in maps {
      let mut asset = compilation
        .assets_mut()
        .remove(&filename)
        .expect("should have filename in compilation.assets");
      // convert to RawSource to reduce one time source map calculation when convert to JsCompatSource
      let raw_source = RawSource::from(code_buffer).boxed();
      let Some(source_map_buffer) = source_map_buffer else {
        continue;
      };
      let css_extension_detected = CSS_EXTENSION_DETECT_REGEXP.is_match(&filename);
      let current_source_mapping_url_comment =
        if let Some(SourceMappingUrlComment::String(s)) = &self.source_mapping_url_comment {
          let s = if css_extension_detected {
            URL_FORMATTING_REGEXP.replace_all(s, "\n/*$1*/")
          } else {
            Cow::from(s)
          };
          Some(s)
        } else {
          None
        };

      if let Some(source_map_filename_config) = &self.source_map_filename {
        let mut source_map_filename = filename.to_owned() + ".map";
        // TODO(ahabhgk): refactor remove the for loop
        for chunk in compilation.chunk_by_ukey.values() {
          let files: HashSet<String> = chunk.files.union(&chunk.auxiliary_files).cloned().collect();

          for file in &files {
            if file == &filename {
              let source_type = if css_extension_detected {
                &SourceType::Css
              } else {
                &SourceType::JavaScript
              };
              let filename = match &self.file_context {
                Some(file_context) => relative(Path::new(file_context), Path::new(&filename))
                  .to_string_lossy()
                  .to_string(),
                None => filename.clone(),
              };
              source_map_filename = compilation.get_asset_path(
                source_map_filename_config,
                PathData::default()
                  .chunk(chunk)
                  .filename(&filename)
                  .content_hash_optional(chunk.content_hash.get(source_type).map(|i| i.encoded())),
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
        let mut source_map_asset_info = AssetInfo::default().with_development(true);
        if let Some(asset) = compilation.assets().get(&filename) {
          // set source map asset version to be the same as the target asset
          source_map_asset_info.version = asset.info.version.clone();
        }
        let source_map_asset = CompilationAsset::new(
          Some(RawSource::from(source_map_buffer).boxed()),
          source_map_asset_info,
        );
        source_and_map_asstes.push((
          filename,
          hash,
          asset,
          Some((source_map_filename, source_map_asset)),
        ));
      } else {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("SourceMapDevToolPlugin: append can't be false when no filename is provided.");
        let base64 = rspack_base64::encode_to_string(&source_map_buffer);
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
        source_and_map_asstes.push((filename, hash, asset, None));
        // TODO
        // chunk.auxiliary_files.add(filename);
      }

      {
        let mut cache = self.source_and_map_assets_cache.write().unwrap();
        cache.clear();
        for (source_filename, hash, source_asset, source_map) in &source_and_map_asstes {
          compilation.emit_asset(source_filename.to_owned(), source_asset.clone());
          if let Some((source_map_filename, source_map_asset)) = source_map {
            compilation.emit_asset(source_map_filename.to_owned(), source_map_asset.clone());
            cache.insert(
              source_filename.to_owned(),
              (
                *hash,
                source_asset.clone(),
                Some((source_map_filename.to_owned(), source_map_asset.clone())),
              ),
            );
          } else {
            cache.insert(
              source_filename.to_owned(),
              (*hash, source_asset.clone(), None),
            );
          }
        }
      }
    }
    logger.time_end(start);
    Ok(())
  }
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

fn get_hash(text: &str, output_options: &OutputOptions) -> String {
  let OutputOptions {
    hash_function,
    hash_salt,
    ..
  } = output_options;
  let mut hasher = RspackHash::with_salt(hash_function, hash_salt);
  text.as_bytes().hash(&mut hasher);
  format!("{:x}", hasher.finish())[..4].to_string()
}

impl ModuleFilenameHelpers {
  fn create_module_filename_template_fn_ctx(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> ModuleFilenameTemplateFnCtx {
    let Compilation {
      chunk_graph,
      module_graph,
      options,
      ..
    } = compilation;
    let context = &options.context;

    match module_or_source {
      ModuleOrSource::Module(module_identifier) => {
        let module = module_graph
          .module_by_identifier(module_identifier)
          .expect("failed to find a module for the given identifier");

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(context, module_identifier);
        let module_id = chunk_graph
          .get_module_id(*module_identifier)
          .clone()
          .unwrap_or("".to_string());
        let absolute_resource_path = "".to_string();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .clone()
          .split('!')
          .last()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&resource, "?");

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
          hash,
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: namespace.to_string(),
        }
      }
      ModuleOrSource::Source(source) => {
        let short_identifier = contextify(context, source);
        let identifier = short_identifier.clone();

        let hash = get_hash(&identifier, output_options);

        let resource = short_identifier
          .clone()
          .split('!')
          .last()
          .unwrap_or("")
          .to_string();

        let loaders = get_before(&short_identifier, "!");
        let all_loaders = get_before(&identifier, "!");
        let query = get_after(&resource, "?");

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
          hash,
          resource,
          loaders,
          all_loaders,
          query,
          resource_path,
          namespace: namespace.to_string(),
        }
      }
    }
  }

  async fn create_filename_of_fn_template(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    module_filename_template: &ModuleFilenameTemplateFn,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> Result<String> {
    let ctx = ModuleFilenameHelpers::create_module_filename_template_fn_ctx(
      module_or_source,
      compilation,
      output_options,
      namespace,
    );

    module_filename_template(ctx).await
  }

  fn create_filename_of_string_template(
    module_or_source: &ModuleOrSource,
    compilation: &Compilation,
    module_filename_template: &str,
    output_options: &OutputOptions,
    namespace: &str,
  ) -> String {
    let ctx = ModuleFilenameHelpers::create_module_filename_template_fn_ctx(
      module_or_source,
      compilation,
      output_options,
      namespace,
    );

    let s = REGEXP_ALL_LOADERS_RESOURCE.replace_all(module_filename_template, "[identifier]");
    let s = REGEXP_LOADERS_RESOURCE.replace_all(&s, "[short-identifier]");
    SQUARE_BRACKET_TAG_REGEXP
      .replace_all(&s, |caps: &Captures| {
        let full_match = caps
          .get(0)
          .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
          .as_str();
        let content = caps
          .get(1)
          .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
          .as_str();

        if content.len() + 2 == full_match.len() {
          match content.to_lowercase().as_str() {
            "identifier" => Cow::from(&ctx.identifier),
            "short-identifier" => Cow::from(&ctx.short_identifier),
            "resource" => Cow::from(&ctx.resource),

            "resource-path" |  "resourcepath" => Cow::from(&ctx.resource_path),

            "absolute-resource-path" |
            "abs-resource-path" |
            "absoluteresource-path" |
            "absresource-path" |
            "absolute-resourcepath" |
            "abs-resourcepath" |
            "absoluteresourcepath" |
            "absresourcepath" => Cow::from(&ctx.absolute_resource_path),

            "all-loaders" | "allloaders" => Cow::from(&ctx.all_loaders),
            "loaders" => Cow::from(&ctx.loaders),

            "query" => Cow::from(&ctx.query),
            "id" => Cow::from(&ctx.module_id),
            "hash" => Cow::from(&ctx.hash),
            "namespace" => Cow::from(&ctx.namespace),

            _ => Cow::from(full_match.to_string())
          }
        } else if full_match.starts_with("[\\") && full_match.ends_with("\\]") {
          Cow::from(format!("[{}]", &full_match[2..full_match.len() - 2]))
        } else {
          Cow::from(full_match.to_string())
        }
      })
      .to_string()
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
      *source = Cow::from(resource_path);
    }
    if self.no_sources {
      for content in map.sources_content_mut() {
        *content = Cow::from(String::default());
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

pub struct SourceMapDevToolModuleOptionsPluginOptions {
  pub module: bool,
}

#[derive(Debug)]
pub struct SourceMapDevToolModuleOptionsPlugin {
  module: bool,
}

impl SourceMapDevToolModuleOptionsPlugin {
  pub fn new(options: SourceMapDevToolModuleOptionsPluginOptions) -> Self {
    Self {
      module: options.module,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for SourceMapDevToolModuleOptionsPlugin {
  fn name(&self) -> &'static str {
    "SourceMapDevToolModuleOptionsPlugin"
  }

  async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    if self.module {
      module.set_source_map_kind(SourceMapKind::SourceMap);
    } else {
      module.set_source_map_kind(SourceMapKind::SimpleSourceMap);
    }
    Ok(())
  }

  async fn runtime_module(
    &self,
    module: &mut dyn RuntimeModule,
    _source: Arc<dyn Source>,
    _chunk: &Chunk,
  ) -> Result<Option<String>> {
    if self.module {
      module.set_source_map_kind(SourceMapKind::SourceMap);
    } else {
      module.set_source_map_kind(SourceMapKind::SimpleSourceMap);
    }
    Ok(None)
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct EvalDevToolModulePluginOptions {
  pub namespace: Option<String>,
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  pub source_url_comment: Option<String>,
}

static EVAL_MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

#[derive(Derivative)]
#[derivative(Debug)]
pub struct EvalDevToolModulePlugin {
  namespace: String,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  source_url_comment: String,
}

impl EvalDevToolModulePlugin {
  pub fn new(options: EvalDevToolModulePluginOptions) -> Self {
    let source_url_comment = options
      .source_url_comment
      .unwrap_or("\n//# sourceURL=[url]".to_string());

    let module_filename_template =
      options
        .module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resourcePath]?[loaders]".to_string(),
        ));

    Self {
      module_filename_template,
      namespace: options.namespace.unwrap_or("".to_string()),
      source_url_comment,
    }
  }

  pub fn wrap_eval(&self, source: &str, source_name: &str) -> Result<BoxSource> {
    let footer = self.source_url_comment.replace("[url]", source_name);
    let result = RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed();
    Ok(result)
  }
}

#[async_trait::async_trait]
impl Plugin for EvalDevToolModulePlugin {
  fn name(&self) -> &'static str {
    "rspack.EvalDevToolModulePlugin"
  }

  fn render_module_content<'a>(
    &'a self,
    _ctx: PluginContext,
    mut args: RenderModuleContentArgs<'a>,
  ) -> PluginRenderModuleContentOutput<'a> {
    let origin_source = args.module_source.clone();
    if let Some(cached) = EVAL_MODULE_RENDER_CACHE.get(&origin_source) {
      args.module_source = cached.value().clone();
      return Ok(args);
    } else if args.module.as_external_module().is_some() {
      return Ok(args);
    }
    let output_options = &args.compilation.options.output;
    let compilation = args.compilation;
    let source_name = match &self.module_filename_template {
      ModuleFilenameTemplate::String(s) => {
        ModuleFilenameHelpers::create_filename_of_string_template(
          &ModuleOrSource::Module(args.module.identifier()),
          args.compilation,
          s,
          output_options,
          self.namespace.as_str(),
        )
      }
      ModuleFilenameTemplate::Fn(f) => {
        futures::executor::block_on(ModuleFilenameHelpers::create_filename_of_fn_template(
          &ModuleOrSource::Module(args.module.identifier()),
          compilation,
          f,
          output_options,
          self.namespace.as_str(),
        ))
        .expect("todo!")
      }
    };
    let source = self.wrap_eval(&origin_source.source(), source_name.as_str())?;

    EVAL_MODULE_RENDER_CACHE.insert(origin_source, source.clone());
    args.module_source = source;
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
