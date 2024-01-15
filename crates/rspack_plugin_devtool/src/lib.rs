#![feature(let_chains)]

use std::borrow::Cow;
use std::hash::Hasher;
use std::sync::Arc;
use std::{hash::Hash, path::Path};

use dashmap::DashMap;
use derivative::Derivative;
use futures::{future::BoxFuture, StreamExt};
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
use rspack_core::{CompilerOptions, Filename, Logger, Module, OutputOptions};
use rspack_error::{miette::IntoDiagnostic, Error, Result};
use rspack_hash::RspackHash;
use rspack_util::source_map::SourceMapKind;
use rspack_util::{path::relative, swc::normalize_custom_filename};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
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

type ModuleFilenameTemplateFn = Box<dyn Fn(ModuleFilenameTemplateFnCtx) -> String + Send + Sync>;

pub enum ModuleFilenameTemplate {
  String(String),
  Fn(ModuleFilenameTemplateFn),
}

type AppendFn = Arc<dyn for<'a> Fn() -> Option<String> + Send + Sync>;

pub enum Append {
  String(String),
  Fn(AppendFn),
  Disabled,
}

pub type TestFn = Arc<dyn Fn(String) -> BoxFuture<'static, Result<bool>> + Sync + Send>;

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
  #[derivative(Debug = "ignore")]
  test: Option<TestFn>,
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
      test: options.test,
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
    let compiler_options = &compilation.options;

    let compilation_assets = compilation.assets();
    let assets = futures::stream::iter(compilation_assets.iter())
      .filter_map(|(file, asset)| {
        let test = self.test.clone();
        async move {
          let is_match = match &test {
            Some(test) => test(file.clone()).await.ok()?,
            None => true,
          };

          if is_match {
            asset
              .get_source()
              .map(|source| (file, source, source.map(&MapOptions::new(self.columns))))
          } else {
            None
          }
        }
      })
      .collect::<Vec<(&String, &Arc<dyn Source>, Option<SourceMap>)>>()
      .await;

    let mut used_names_set = HashSet::<String>::default();
    let mut maps: Vec<(String, Vec<u8>, Option<Vec<u8>>)> = Vec::with_capacity(assets.len());

    for (file, asset, source_map) in assets {
      let source_map_buffer = source_map
        .map(|mut source_map| {
          source_map.set_file(Some(file.clone()));
          for source in source_map.sources_mut() {
            // try the fallback name first
            let mut source_name = self.create_filename(
              source,
              compiler_options,
              &self.module_filename_template,
              output_options,
            );

            let mut has_name = used_names_set.contains(&source_name);
            if !has_name {
              used_names_set.insert(source_name.clone());
              *source = source_name;
              continue;
            }

            // Try the fallback name first
            source_name = self.create_filename(
              source,
              compiler_options,
              &self.fallback_module_filename_template,
              output_options,
            );
            has_name = used_names_set.contains(&source_name);
            if !has_name {
              used_names_set.insert(source_name.clone());
              *source = source_name;
              continue;
            }

            // Otherwise, append stars until we have a valid name
            while has_name {
              source_name.push('*');
              has_name = used_names_set.contains(&source_name);
            }
            used_names_set.insert(source_name.clone());
            *source = source_name;
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
      maps.push((file.to_owned(), code_buffer, source_map_buffer));
    }

    logger.time_end(start);

    let start = logger.time("emit source map assets");
    for (filename, code_buffer, source_map_buffer) in maps {
      let mut asset = compilation
        .assets_mut()
        .remove(&filename)
        .expect("should have filename in compilation.assets");
      // convert to RawSource to reduce one time source map calculation when convert to JsCompatSource
      let raw_source = RawSource::from(code_buffer).boxed();
      let Some(source_map_buffer) = source_map_buffer else {
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
            Some(RawSource::from(source_map_buffer).boxed()),
            source_map_asset_info,
          ),
        );
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
        compilation.emit_asset(filename, asset);
        // TODO
        // chunk.auxiliary_files.add(filename);
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

impl SourceMapDevToolPlugin {
  fn create_filename(
    &self,
    source: &str,
    options: &CompilerOptions,
    module_filename_template: &ModuleFilenameTemplate,
    output_options: &OutputOptions,
  ) -> String {
    let context = &options.context;

    let short_identifier = contextify(context, source);
    let identifier = &short_identifier;
    let module_id = "".to_string();
    let absolute_resource_path = source.split('!').last().unwrap_or("");

    let text = identifier.clone();
    let hash = Lazy::new(|| get_hash(&text, output_options));

    let resource = short_identifier.split('!').last().unwrap_or("");

    let all_loaders = get_before(identifier, "!");
    let query = get_after(resource, "?");

    let q = query.len();
    let resource_path = if q == 0 {
      resource
    } else {
      &resource[..resource.len().saturating_sub(q)]
    };

    return match module_filename_template {
      ModuleFilenameTemplate::Fn(f) => {
        let loaders = get_before(&short_identifier, "!");

        let ctx = ModuleFilenameTemplateFnCtx {
          short_identifier: short_identifier.clone(),
          identifier: identifier.clone(),
          module_id,
          absolute_resource_path: absolute_resource_path.to_string(),
          hash: hash.clone(),
          resource: resource.to_string(),
          loaders,
          all_loaders,
          query,
          resource_path: resource_path.to_string(),
          namespace: self.namespace.clone(),
        };
        f(ctx)
      }
      ModuleFilenameTemplate::String(s) => {
        let mut replacements: HashMap<&str, &str> = HashMap::default();
        replacements.insert("identifier", identifier);
        replacements.insert("short-identifier", &short_identifier);
        replacements.insert("resource", resource);

        replacements.insert("resource-path", resource_path);
        replacements.insert("resourcepath", resource_path);

        replacements.insert("absolute-resource-path", absolute_resource_path);
        replacements.insert("abs-resource-path", absolute_resource_path);
        replacements.insert("absoluteresource-path", absolute_resource_path);
        replacements.insert("absresource-path", absolute_resource_path);
        replacements.insert("absolute-resourcepath", absolute_resource_path);
        replacements.insert("abs-resourcepath", absolute_resource_path);
        replacements.insert("absoluteresourcepath", absolute_resource_path);
        replacements.insert("absresourcepath", absolute_resource_path);

        replacements.insert("all-loaders", &all_loaders);
        replacements.insert("allloaders", &all_loaders);

        replacements.insert("query", &query);
        replacements.insert("id", &module_id);
        replacements.insert("hash", &hash);
        replacements.insert("namespace", &self.namespace);

        let s = REGEXP_ALL_LOADERS_RESOURCE.replace_all(s, "[identifier]");
        let s = REGEXP_LOADERS_RESOURCE.replace_all(&s, "[short-identifier]");
        SQUARE_BRACKET_TAG_REGEXP
          .replace_all(&s, |caps: &Captures| {
            let full_match = caps
              .get(0)
              .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
              .as_str()
              .to_string();
            let content = caps
              .get(1)
              .expect("the SQUARE_BRACKET_TAG_REGEXP must match the whole tag, but it did not match anything.")
              .as_str();

            if content.len() + 2 == full_match.len() {
              if let Some(replacement) = replacements.get(content.to_lowercase().as_str()) {
                Cow::from(*replacement)
              } else {
                Cow::from(full_match)
              }
            } else if full_match.starts_with("[\\") && full_match.ends_with("\\]") {
              Cow::from(format!("[{}]", &full_match[2..full_match.len() - 2]))
            } else {
              Cow::from(full_match)
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

  fn runtime_module(&self, module: &mut dyn Module) -> Result<()> {
    if self.module {
      module.set_source_map_kind(SourceMapKind::SourceMap);
    } else {
      module.set_source_map_kind(SourceMapKind::SimpleSourceMap);
    }
    Ok(())
  }
}
