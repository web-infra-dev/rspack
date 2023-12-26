#![feature(let_chains)]

use std::collections::HashSet;
use std::hash::Hasher;
use std::sync::Arc;
use std::{hash::Hash, path::Path};

use async_recursion::async_recursion;
use dashmap::DashMap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use regex::{Captures, Regex};
use rspack_core::{
  contextify,
  rspack_sources::{BoxSource, ConcatSource, MapOptions, RawSource, Source, SourceExt, SourceMap},
  AssetInfo, Compilation, CompilationAsset, JsChunkHashArgs, PathData, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginProcessAssetsOutput, PluginRenderModuleContentOutput,
  ProcessAssetsArgs, RenderModuleContentArgs, SourceType,
};
use rspack_core::{
  ChunkGraph, CompilationArgs, CompilationParams, Context, Filename, Logger, Module,
  ModuleIdentifier, OutputOptions, PluginCompilationHookOutput,
};
use rspack_error::miette::IntoDiagnostic;
use rspack_error::{Error, Result};
use rspack_hash::RspackHash;
use rspack_regex::RspackRegex;
use rspack_util::identifier::make_paths_absolute;
use rspack_util::swc::normalize_custom_filename;
use rspack_util::try_any_sync;
use rustc_hash::FxHashMap as HashMap;
use serde_json::json;

static CSS_EXTENSION_DETECT_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css($|\?)").expect("Failed to compile CSS_EXTENSION_DETECT_REGEXP"));
static URL_FORMATTING_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\n\/\/(.*)$").expect("Failed to compile URL_FORMATTING_REGEXP regex"));

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
  pub module_id: String,
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

type AppendFn = Arc<dyn for<'a> Fn() -> Option<String> + Send + Sync>;

pub enum Append {
  String(String),
  Fn(AppendFn),
  Disabled,
}

#[derive(Debug)]
pub enum Rule {
  String(String),
  Regexp(RspackRegex),
}

impl Rule {
  pub fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

#[derive(Debug)]
pub enum Rules {
  Single(Rule),
  Array(Vec<Rule>),
}

impl Rules {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::Single(s) => s.try_match(data),
      Self::Array(l) => try_any_sync(l, |i| i.try_match(data)),
    }
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPluginOptions {
  // Appends the given value to the original asset. Usually the #sourceMappingURL comment. [url] is replaced with a URL to the source map file. false disables the appending.
  #[derivative(Debug = "ignore")]
  pub append: Option<Append>,
  // Indicates whether column mappings should be used (defaults to true).
  pub columns: bool,
  // Exclude modules that match the given value from source map generation.
  pub exclude: Option<Rules>,
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap used only if 'moduleFilenameTemplate' would result in a conflict.
  #[derivative(Debug = "ignore")]
  pub fallback_module_filename_template: Option<ModuleFilenameTemplate>,
  // Path prefix to which the [file] placeholder is relative to.
  pub file_context: Option<String>,
  // Defines the output filename of the SourceMap (will be inlined if no value is provided).
  pub filename: Option<String>,
  // Include source maps for module paths that match the given value.
  pub include: Option<Rules>,
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
  // Include source maps for modules based on their extension (defaults to .js and .css).
  pub test: Option<Rules>,
}

enum SourceMappingUrlComment {
  String(String),
  Fn(AppendFn),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPlugin {
  filename: Option<Filename>,
  #[derivative(Debug = "ignore")]
  source_mapping_url_comment: Option<SourceMappingUrlComment>,
  #[derivative(Debug = "ignore")]
  fallback_module_filename_template: ModuleFilenameTemplate,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
  module: bool,
  test: Option<Rules>,
  include: Option<Rules>,
  exclude: Option<Rules>,
}

struct Task<'a> {
  file: &'a String,
  asset: &'a Arc<dyn Source>,
  source_map: Option<SourceMap>,
  modules: Vec<ModuleOrSource<'a>>,
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
      filename: options.filename.map(Filename::from),
      source_mapping_url_comment,
      fallback_module_filename_template,
      module_filename_template,
      namespace: options.namespace.unwrap_or("".to_string()),
      columns: options.columns,
      no_sources: options.no_sources,
      public_path: options.public_path,
      module: options.module,
      test: options.test,
      include: options.include,
      exclude: options.exclude,
    }
  }
}

#[async_trait::async_trait]
impl Plugin for SourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    "rspack.SourceMapDevToolPlugin"
  }

  async fn compilation(
    &self,
    _args: CompilationArgs<'_>,
    _params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    // TODO: Temporarily use `devtool` to pass source map configuration information
    let mut devtool = _args.compilation.options.devtool.lock().await;
    devtool.add_source_map();
    if self.filename.is_none() {
      devtool.add_inline();
    }
    if self.source_mapping_url_comment.is_none() {
      devtool.add_hidden();
    }
    if !self.columns {
      devtool.add_cheap();
    }
    if self.no_sources {
      devtool.add_no_sources();
    }
    if self.module {
      devtool.add_module();
    }
    Ok(())
  }

  async fn process_assets_stage_dev_tooling(
    &self,
    _ctx: PluginContext,
    ProcessAssetsArgs { compilation }: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("collect source maps");
    let context = &compilation.options.context;
    let output_options = &compilation.options.output;

    let mut tasks = vec![];
    let mut module_to_source_name_mapping = HashMap::<ModuleOrSource, String>::default();

    // TODO: maybe can be parallelized
    let mut assets: Vec<(&String, &Arc<dyn Source>)> = vec![];
    for (file, asset) in compilation.assets() {
      let is_match = self.match_object(file).await.unwrap_or(false);
      if is_match {
        if let Some(source) = asset.get_source() {
          assets.push((file, source));
        }
      }
    }

    for (file, asset) in assets {
      let source_map = asset.map(&MapOptions::new(self.columns));

      let mut modules = vec![];
      if let Some(source_map) = &source_map {
        for source in source_map.sources() {
          let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
            let source = make_paths_absolute(context.as_str(), stripped);
            // TODO: is true way to use source to find module?
            let identifier = ModuleIdentifier::from(source.clone());
            match compilation.module_graph.module_by_identifier(&identifier) {
              Some(module) => ModuleOrSource::Module(module.as_ref()),
              None => ModuleOrSource::Source(source),
            }
          } else {
            ModuleOrSource::Source(source.clone())
          };

          let source_name = self.create_filename(
            &module_or_source,
            context,
            &compilation.chunk_graph,
            &self.module_filename_template,
            output_options,
          );
          module_to_source_name_mapping.insert(module_or_source.clone(), source_name);
          modules.push(module_or_source);
        }
      }
      let task = Task {
        file,
        asset,
        source_map,
        modules,
      };
      tasks.push(task)
    }

    let mut used_names_set: HashSet<String> =
      module_to_source_name_mapping.values().cloned().collect();
    let mut conflict_detection_set = HashSet::<String>::new();

    // all modules in defined order (longest identifier first)
    let mut all_modules: Vec<ModuleOrSource> =
      module_to_source_name_mapping.keys().cloned().collect();
    all_modules.sort_by_key(|module| match module {
      ModuleOrSource::Module(module) => module.identifier().len(),
      ModuleOrSource::Source(source) => source.len(),
    });

    for module in all_modules {
      let mut source_name = module_to_source_name_mapping
        .get(&module)
        .expect("Expected to find a source name for the module, but none was present.")
        .clone();
      let mut has_name = conflict_detection_set.contains(&source_name);

      if !has_name {
        conflict_detection_set.insert(source_name);
        continue;
      }

      // Try the fallback name first
      source_name = self.create_filename(
        &module,
        context,
        &compilation.chunk_graph,
        &self.fallback_module_filename_template,
        output_options,
      );
      has_name = used_names_set.contains(&source_name);
      if !has_name {
        module_to_source_name_mapping.insert(module, source_name.clone());
        used_names_set.insert(source_name);
        continue;
      }

      // Otherwise, append stars until we have a valid name
      while has_name {
        source_name.push('*');
        has_name = used_names_set.contains(&source_name);
      }
      module_to_source_name_mapping.insert(module, source_name.clone());
      used_names_set.insert(source_name);
    }

    let maps: HashMap<String, (Vec<u8>, Option<Vec<u8>>)> = tasks
      .into_iter()
      .map(|task| {
        let Task {
          file,
          asset,
          source_map,
          modules,
        } = task;

        let source_map_buffer = source_map
          .map(|mut source_map| {
            source_map.set_file(Some(file.clone()));
            let source_names = modules
              .iter()
              .map(|module| {
                module_to_source_name_mapping
                  .get(module)
                  .expect("Expected to find a source name for the module, but none was present.")
                  .clone()
              })
              .collect::<Vec<String>>();
            for (idx, source) in source_map.sources_mut().iter_mut().enumerate() {
              *source = source_names[idx].clone();
            }
            if self.no_sources {
              for content in source_map.sources_content_mut() {
                *content = String::default();
              }
            }
            let mut source_map_buffer = Vec::new();
            source_map
              .to_writer(&mut source_map_buffer)
              .unwrap_or_else(|e| panic!("{}", e.to_string()));
            Ok::<Vec<u8>, Error>(source_map_buffer)
          })
          .transpose()?;
        let mut code_buffer = Vec::new();
        asset.to_writer(&mut code_buffer).into_diagnostic()?;
        Ok((file.to_owned(), (code_buffer, source_map_buffer)))
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
      let css_extension_detected = CSS_EXTENSION_DETECT_REGEXP.is_match(&filename);
      let current_source_mapping_url_comment = if let Some(SourceMappingUrlComment::String(s)) =
        self.source_mapping_url_comment.as_ref()
      {
        let s = if css_extension_detected {
          URL_FORMATTING_REGEXP.replace_all(s, "\n/*$1*/").to_string()
        } else {
          s.clone()
        };
        Some(s)
      } else {
        None
      };
      if let Some(source_map_filename_config) = &self.filename {
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

#[derive(Eq, Hash, PartialEq, Clone)]
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

impl SourceMapDevToolPlugin {
  fn create_filename(
    &self,
    module_or_source: &ModuleOrSource,
    context: &Context,
    chunk_graph: &ChunkGraph,
    module_filename_template: &ModuleFilenameTemplate,
    output_options: &OutputOptions,
  ) -> String {
    let ctx = match module_or_source {
      ModuleOrSource::Module(module) => {
        let module_identifier = module.identifier();

        let short_identifier = module.readable_identifier(context).to_string();
        let identifier = contextify(context, &module_identifier);
        let module_id = chunk_graph
          .get_module_id(module_identifier)
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
          namespace: self.namespace.clone(),
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
          namespace: self.namespace.clone(),
        }
      }
    };

    return match module_filename_template {
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
            let full_match = caps
              .get(0)
              .expect("The regex must match the whole tag, but it did not match anything.")
              .as_str();
            let content = caps
              .get(1)
              .expect("The regex must match the whole tag, but it did not match anything.")
              .as_str();

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

  // TODO: together with Rules in rspack_plugin_banner
  #[async_recursion]
  async fn match_object(&self, str: &str) -> Result<bool> {
    if let Some(condition) = &self.test {
      if !condition.try_match(str).await? {
        return Ok(false);
      }
    }
    if let Some(condition) = &self.include {
      if !condition.try_match(str).await? {
        return Ok(false);
      }
    }
    if let Some(condition) = &self.exclude {
      if condition.try_match(str).await? {
        return Ok(false);
      }
    }
    Ok(true)
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

  async fn compilation(
    &self,
    _args: CompilationArgs<'_>,
    _params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    // TODO: Temporarily use `devtool` to pass source map configuration information
    let mut devtool = _args.compilation.options.devtool.lock().await;
    devtool.add_source_map();
    devtool.add_eval();
    if !self.columns {
      devtool.add_cheap();
    }
    if self.no_sources {
      devtool.add_no_sources();
    }
    Ok(())
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
