use std::{borrow::Cow, path::Path};

use derivative::Derivative;
use futures::future::{join_all, BoxFuture};
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{ConcatSource, MapOptions, RawSource, Source, SourceExt},
  AssetInfo, Chunk, ChunkUkey, Compilation, CompilationAsset, CompilationProcessAssets,
  FilenameTemplate, Logger, ModuleIdentifier, PathData, Plugin, PluginContext, SourceType,
};
use rspack_error::{error, miette::IntoDiagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{
  identifier::make_paths_absolute, infallible::ResultInfallibleExt, path::relative,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  mapped_assets_cache::MappedAssetsCache, module_filename_helpers::ModuleFilenameHelpers,
  ModuleFilenameTemplateFn, ModuleOrSource,
};

static CSS_EXTENSION_DETECT_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.css($|\?)").expect("failed to compile CSS_EXTENSION_DETECT_REGEXP"));
static URL_FORMATTING_REGEXP: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\n\/\/(.*)$").expect("failed to compile URL_FORMATTING_REGEXP regex"));

#[derive(Clone)]
pub enum ModuleFilenameTemplate {
  String(String),
  Fn(ModuleFilenameTemplateFn),
}

type AppendFn = Box<dyn for<'a> Fn(PathData) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub enum Append {
  String(String),
  Fn(AppendFn),
  Disabled,
}

pub type TestFn = Box<dyn Fn(String) -> Result<bool> + Sync + Send>;

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

enum SourceMappingUrlCommentRef<'a> {
  String(String),
  Fn(&'a AppendFn),
}

#[derive(Debug, Clone)]
pub(crate) struct MappedAsset {
  pub(crate) asset: (String, CompilationAsset),
  pub(crate) source_map: Option<(String, CompilationAsset)>,
}

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SourceMapDevToolPlugin {
  source_map_filename: Option<FilenameTemplate>,
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
  mapped_assets_cache: MappedAssetsCache,
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

    Self::new_inner(
      options.filename.map(FilenameTemplate::from),
      source_mapping_url_comment,
      options.file_context,
      module_filename_template,
      fallback_module_filename_template,
      options.namespace.unwrap_or("".to_string()),
      options.columns,
      options.no_sources,
      options.public_path,
      options.module,
      options.source_root,
      options.test,
      MappedAssetsCache::new(),
    )
  }

  async fn map_assets(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<String, &Chunk>,
    raw_assets: Vec<(String, &CompilationAsset)>,
  ) -> Result<Vec<MappedAsset>> {
    let output_options = &compilation.options.output;
    let map_options = MapOptions::new(self.columns);

    let mapped_sources = raw_assets
      .par_iter()
      .map(|(file, asset)| {
        let is_match = match &self.test {
          Some(test) => test(file.to_owned()),
          None => Ok(true),
        }?;
        let source = if is_match {
          asset.get_source().map(|source| {
            let source_map = source.map(&map_options);
            (file, source, source_map)
          })
        } else {
          None
        };
        Ok(source)
      })
      .collect::<Result<Vec<Option<_>>>>()?
      .into_iter()
      .flatten()
      .collect::<Vec<_>>();

    let mut used_names_set = HashSet::<String>::default();
    let mut mapped_buffer: Vec<(String, Vec<u8>, Option<Vec<u8>>)> =
      Vec::with_capacity(mapped_sources.len());

    let mut default_filenames = match &self.module_filename_template {
      ModuleFilenameTemplate::String(s) => mapped_sources
        .iter()
        .filter_map(|(_file, _asset, source_map)| source_map.as_ref())
        .flat_map(|source_map| source_map.sources())
        .collect::<Vec<_>>()
        .par_iter()
        .map(|source| {
          let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
            let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
            let identifier = ModuleIdentifier::from(source.clone());
            match compilation
              .get_module_graph()
              .module_by_identifier(&identifier)
            {
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
        let features = mapped_sources
          .iter()
          .filter_map(|(_file, _asset, source_map)| source_map.as_ref())
          .flat_map(|source_map| source_map.sources())
          .map(|source| async {
            let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
              let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
              let identifier = ModuleIdentifier::from(source.clone());
              match compilation
                .get_module_graph()
                .module_by_identifier(&identifier)
              {
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

    for (filename, asset, source_map) in mapped_sources {
      let source_map_buffer = match source_map {
        Some(mut source_map) => {
          source_map.set_file(Some(filename.clone()));

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

      let mut code_buffer = Vec::new();
      asset.to_writer(&mut code_buffer).into_diagnostic()?;
      mapped_buffer.push((filename.to_owned(), code_buffer, source_map_buffer));
    }

    let mut mapped_asstes: Vec<MappedAsset> = Vec::with_capacity(raw_assets.len());
    for (filename, code_buffer, source_map_buffer) in mapped_buffer {
      let mut asset = compilation
        .assets()
        .get(&filename)
        .unwrap_or_else(|| {
          panic!(
            "expected to find filename '{}' in compilation.assets, but it was not present",
            &filename
          )
        })
        .clone();
      // convert to RawSource to reduce one time source map calculation when convert to JsCompatSource
      let raw_source = RawSource::from(code_buffer).boxed();
      let Some(source_map_buffer) = source_map_buffer else {
        asset.source = Some(raw_source);
        mapped_asstes.push(MappedAsset {
          asset: (filename, asset),
          source_map: None,
        });
        continue;
      };
      let css_extension_detected = CSS_EXTENSION_DETECT_REGEXP.is_match(&filename);
      let current_source_mapping_url_comment = match &self.source_mapping_url_comment {
        Some(SourceMappingUrlComment::String(s)) => {
          let s = if css_extension_detected {
            URL_FORMATTING_REGEXP.replace_all(s, "\n/*$1*/")
          } else {
            Cow::from(s)
          };
          Some(SourceMappingUrlCommentRef::String(s.to_string()))
        }
        Some(SourceMappingUrlComment::Fn(f)) => Some(SourceMappingUrlCommentRef::Fn(f)),
        None => None,
      };

      if let Some(source_map_filename_config) = &self.source_map_filename {
        let chunk = file_to_chunk.get(&filename);
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
        let data = PathData::default().filename(&filename);
        let data = match chunk {
          Some(chunk) => data
            .chunk(chunk)
            .content_hash_optional(chunk.content_hash.get(source_type).map(|i| i.encoded())),
          None => data,
        };
        let source_map_filename = compilation
          .get_asset_path(source_map_filename_config, data)
          .always_ok();

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
          let data = data.url(&source_map_url);
          let current_source_mapping_url_comment = match &current_source_mapping_url_comment {
            SourceMappingUrlCommentRef::String(s) => compilation
              .get_asset_path(&FilenameTemplate::from(s.to_string()), data)
              .always_ok(),
            SourceMappingUrlCommentRef::Fn(f) => {
              let comment = f(data).await?;
              FilenameTemplate::from(comment)
                .render(data, None)
                .always_ok()
            }
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
        mapped_asstes.push(MappedAsset {
          asset: (filename, asset.clone()),
          source_map: Some((source_map_filename, source_map_asset)),
        });
      } else {
        let current_source_mapping_url_comment = current_source_mapping_url_comment
          .expect("SourceMapDevToolPlugin: append can't be false when no filename is provided.");
        let current_source_mapping_url_comment = match &current_source_mapping_url_comment {
          SourceMappingUrlCommentRef::String(s) => s,
          SourceMappingUrlCommentRef::Fn(_) => {
            return Err(error!(
              "SourceMapDevToolPlugin: append can't be a function when no filename is provided"
            ))
          }
        };
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
        mapped_asstes.push(MappedAsset {
          asset: (filename, asset),
          source_map: None,
        });
      }
    }
    Ok(mapped_asstes)
  }
}

#[plugin_hook(CompilationProcessAssets for SourceMapDevToolPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_DEV_TOOLING)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.SourceMapDevToolPlugin");
  let start = logger.time("collect source maps");

  // use to read
  let mut file_to_chunk: HashMap<String, &Chunk> = HashMap::default();
  // use to write
  let mut file_to_chunk_ukey: HashMap<String, ChunkUkey> = HashMap::default();
  for chunk in compilation.chunk_by_ukey.values() {
    for file in &chunk.files {
      file_to_chunk.insert(file.clone(), chunk);
      file_to_chunk_ukey.insert(file.clone(), chunk.ukey);
    }
    for file in &chunk.auxiliary_files {
      file_to_chunk.insert(file.clone(), chunk);
      file_to_chunk_ukey.insert(file.clone(), chunk.ukey);
    }
  }

  let mapped_asstes = self
    .mapped_assets_cache
    .use_cache(compilation.assets(), |assets| {
      self.map_assets(compilation, &file_to_chunk, assets)
    })
    .await?;

  logger.time_end(start);

  let start = logger.time("emit source map assets");

  for mapped_asset in mapped_asstes {
    let MappedAsset {
      asset: (source_filename, mut source_asset),
      source_map,
    } = mapped_asset;
    if let Some(asset) = compilation.assets_mut().remove(&source_filename) {
      source_asset.info = asset.info;
    }
    compilation.emit_asset(source_filename.to_owned(), source_asset.clone());
    if let Some((source_map_filename, source_map_asset)) = source_map {
      compilation.emit_asset(source_map_filename.to_owned(), source_map_asset.clone());

      let chunk_ukey = file_to_chunk_ukey.get(&source_filename);
      let chunk = chunk_ukey.map(|ukey| compilation.chunk_by_ukey.expect_get_mut(ukey));
      if let Some(chunk) = chunk {
        chunk.auxiliary_files.insert(source_map_filename.to_owned());
      }
    }
  }

  logger.time_end(start);
  Ok(())
}

impl Plugin for SourceMapDevToolPlugin {
  fn name(&self) -> &'static str {
    "rspack.SourceMapDevToolPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
