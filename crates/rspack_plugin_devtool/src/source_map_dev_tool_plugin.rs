use std::{
  borrow::Cow,
  hash::Hasher,
  path::{Component, Path, PathBuf},
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use derive_more::Debug;
use futures::future::{join_all, BoxFuture};
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use rspack_collections::DatabaseItem;
use rspack_core::{
  rspack_sources::{ConcatSource, MapOptions, RawStringSource, Source, SourceExt},
  AssetInfo, Chunk, ChunkUkey, Compilation, CompilationAsset, CompilationProcessAssets, Filename,
  Logger, ModuleIdentifier, PathData, Plugin, PluginContext,
};
use rspack_error::{error, miette::IntoDiagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{asset_condition::AssetConditions, identifier::make_paths_absolute};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sugar_path::SugarPath;

use crate::{
  mapped_assets_cache::MappedAssetsCache, module_filename_helpers::ModuleFilenameHelpers,
  ModuleFilenameTemplateFn, ModuleOrSource,
};

static SCHEMA_SOURCE_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(data|https?):").expect("failed to compile SCHEMA_SOURCE_REGEXP"));

static CSS_EXTENSION_DETECT_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"\.css($|\?)").expect("failed to compile CSS_EXTENSION_DETECT_REGEXP")
});
static URL_FORMATTING_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^\n\/\/(.*)$").expect("failed to compile URL_FORMATTING_REGEXP regex")
});

#[derive(Clone)]
pub enum ModuleFilenameTemplate {
  String(String),
  Fn(ModuleFilenameTemplateFn),
}

type AppendFn = Box<dyn Fn(PathData) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub enum Append {
  String(String),
  Fn(AppendFn),
  Disabled,
}

#[derive(Debug)]
pub struct SourceMapDevToolPluginOptions {
  // Appends the given value to the original asset. Usually the #sourceMappingURL comment. [url] is replaced with a URL to the source map file. false disables the appending.
  #[debug(skip)]
  pub append: Option<Append>,
  // Indicates whether column mappings should be used (defaults to true).
  pub columns: bool,
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap used only if 'moduleFilenameTemplate' would result in a conflict.
  #[debug(skip)]
  pub fallback_module_filename_template: Option<ModuleFilenameTemplate>,
  // Path prefix to which the [file] placeholder is relative to.
  pub file_context: Option<String>,
  // Defines the output filename of the SourceMap (will be inlined if no value is provided).
  pub filename: Option<String>,
  // Indicates whether SourceMaps from loaders should be used (defaults to true).
  pub module: bool,
  // Generator string or function to create identifiers of modules for the 'sources' array in the SourceMap.
  #[debug(skip)]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  // Namespace prefix to allow multiple webpack roots in the devtools.
  pub namespace: Option<String>,
  // Omit the 'sourceContents' array from the SourceMap.
  pub no_sources: bool,
  // Provide a custom public path for the SourceMapping comment.
  pub public_path: Option<String>,
  // Provide a custom value for the 'sourceRoot' property in the SourceMap.
  pub source_root: Option<String>,
  pub test: Option<AssetConditions>,
  pub include: Option<AssetConditions>,
  pub exclude: Option<AssetConditions>,
  pub debug_ids: bool,
}

fn create_debug_id(filename: &str, source: &[u8]) -> String {
  // We need two 64 bit hashes to give us the 128 bits required for a uuid
  // The first 64 bit hash is built from the source
  let mut hasher = RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
  hasher.write(source);
  let hash1 = hasher.finish().to_le_bytes();

  // The second 64 bit hash is built from the filename and the source hash
  let mut hasher = RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
  hasher.write(filename.as_bytes());
  hasher.write(&hash1);
  let hash2 = hasher.finish().to_le_bytes();

  let mut bytes = [hash1, hash2].concat();

  // Build the uuid from the 16 bytes
  let mut uuid = String::with_capacity(36);
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  for (i, byte) in bytes.iter().enumerate() {
    if i == 4 || i == 6 || i == 8 || i == 10 {
      uuid.push('-');
    }
    uuid.push_str(&format!("{byte:02x}"));
  }
  uuid
}

enum SourceMappingUrlComment {
  String(String),
  Fn(AppendFn),
}

enum SourceMappingUrlCommentRef<'a> {
  String(Cow<'a, str>),
  Fn(&'a AppendFn),
}

#[derive(Debug, Clone)]
pub(crate) struct MappedAsset {
  pub(crate) asset: (String, CompilationAsset),
  pub(crate) source_map: Option<(String, CompilationAsset)>,
}

#[plugin]
#[derive(Debug)]
pub struct SourceMapDevToolPlugin {
  source_map_filename: Option<Filename>,
  #[debug(skip)]
  source_mapping_url_comment: Option<SourceMappingUrlComment>,
  file_context: Option<String>,
  #[debug(skip)]
  module_filename_template: ModuleFilenameTemplate,
  #[debug(skip)]
  fallback_module_filename_template: ModuleFilenameTemplate,
  namespace: String,
  columns: bool,
  no_sources: bool,
  public_path: Option<String>,
  #[expect(dead_code)]
  module: bool,
  source_root: Option<Arc<str>>,
  test: Option<AssetConditions>,
  include: Option<AssetConditions>,
  exclude: Option<AssetConditions>,
  debug_ids: bool,
  mapped_assets_cache: MappedAssetsCache,
}

fn match_object(obj: &SourceMapDevToolPlugin, str: &str) -> bool {
  if let Some(condition) = &obj.test {
    if !condition.try_match(str) {
      return false;
    }
  }
  if let Some(condition) = &obj.include {
    if !condition.try_match(str) {
      return false;
    }
  }
  if let Some(condition) = &obj.exclude {
    if condition.try_match(str) {
      return false;
    }
  }
  true
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
      options.filename.map(Filename::from),
      source_mapping_url_comment,
      options.file_context,
      module_filename_template,
      fallback_module_filename_template,
      options.namespace.unwrap_or("".to_string()),
      options.columns,
      options.no_sources,
      options.public_path,
      options.module,
      options.source_root.map(Arc::from),
      options.test,
      options.include,
      options.exclude,
      options.debug_ids,
      MappedAssetsCache::new(),
    )
  }

  async fn map_assets(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&String, &Chunk>,
    raw_assets: Vec<(String, &CompilationAsset)>,
  ) -> Result<Vec<MappedAsset>> {
    let output_options = &compilation.options.output;
    let map_options = MapOptions::new(self.columns);
    let need_match = self.test.is_some() || self.include.is_some() || self.exclude.is_some();

    let mut mapped_sources = raw_assets
      .into_par_iter()
      .filter_map(|(file, asset)| {
        let is_match = if need_match {
          match_object(self, &file)
        } else {
          true
        };
        let source = if is_match {
          asset.get_source().map(|source| {
            let source_map = source.map(&map_options);
            (file, source, source_map)
          })
        } else {
          None
        };
        source
      })
      .collect::<Vec<_>>();

    let source_map_modules = mapped_sources
      .par_iter()
      .filter_map(|(_file, _asset, source_map)| source_map.as_ref())
      .flat_map(|source_map| source_map.sources())
      .map(|source| {
        let module_or_source = if let Some(stripped) = source.strip_prefix("webpack://") {
          let source = make_paths_absolute(compilation.options.context.as_str(), stripped);
          let identifier = ModuleIdentifier::from(source.as_str());
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
        (source.to_string(), module_or_source)
      })
      .collect::<HashMap<_, _>>();

    let module_source_names = source_map_modules.values().collect::<Vec<_>>();
    let mut module_to_source_name = match &self.module_filename_template {
      ModuleFilenameTemplate::String(s) => module_source_names
        .into_par_iter()
        .map(|module_or_source| {
          if let ModuleOrSource::Source(source) = module_or_source {
            if SCHEMA_SOURCE_REGEXP.is_match(source) {
              return (module_or_source, source.to_string());
            }
          }

          let source_name = ModuleFilenameHelpers::create_filename_of_string_template(
            module_or_source,
            compilation,
            s,
            output_options,
            &self.namespace,
          );
          (module_or_source, source_name)
        })
        .collect::<HashMap<_, _>>(),
      ModuleFilenameTemplate::Fn(f) => {
        let features = module_source_names
          .into_iter()
          .map(|module_or_source| async move {
            if let ModuleOrSource::Source(source) = module_or_source {
              if SCHEMA_SOURCE_REGEXP.is_match(source) {
                return Ok((module_or_source, source.to_string()));
              }
            }

            let source_name = ModuleFilenameHelpers::create_filename_of_fn_template(
              module_or_source,
              compilation,
              f,
              output_options,
              &self.namespace,
            )
            .await?;
            Ok((module_or_source, source_name))
          })
          .collect::<Vec<_>>();
        join_all(features)
          .await
          .into_iter()
          .collect::<Result<HashMap<_, _>>>()?
      }
    };

    let mut used_names_set = HashSet::<&String>::default();
    for (module_or_source, source_name) in
      module_to_source_name
        .iter_mut()
        .sorted_by(|(key_a, _), (key_b, _)| {
          let ident_a = match key_a {
            ModuleOrSource::Module(identifier) => identifier,
            ModuleOrSource::Source(source) => source.as_str(),
          };
          let ident_b = match key_b {
            ModuleOrSource::Module(identifier) => identifier,
            ModuleOrSource::Source(source) => source.as_str(),
          };
          ident_a.len().cmp(&ident_b.len())
        })
    {
      let mut has_name = used_names_set.contains(source_name);
      if !has_name {
        used_names_set.insert(source_name);
        continue;
      }

      // Try the fallback name first
      let mut new_source_name = match &self.fallback_module_filename_template {
        ModuleFilenameTemplate::String(s) => {
          ModuleFilenameHelpers::create_filename_of_string_template(
            module_or_source,
            compilation,
            s,
            output_options,
            self.namespace.as_str(),
          )
        }
        ModuleFilenameTemplate::Fn(f) => {
          ModuleFilenameHelpers::create_filename_of_fn_template(
            module_or_source,
            compilation,
            f,
            output_options,
            self.namespace.as_str(),
          )
          .await?
        }
      };

      has_name = used_names_set.contains(&new_source_name);
      if !has_name {
        *source_name = new_source_name;
        used_names_set.insert(source_name);
        continue;
      }

      // Otherwise, append stars until we have a valid name
      while has_name {
        new_source_name.push('*');
        has_name = used_names_set.contains(&new_source_name);
      }
      *source_name = new_source_name;
      used_names_set.insert(source_name);
    }

    for (filename, _asset, source_map) in mapped_sources.iter_mut() {
      if let Some(source_map) = source_map {
        source_map.set_file(Some(filename.clone()));

        source_map.set_sources(
          source_map
            .sources()
            .iter()
            .map(|source| {
              let module_or_source = source_map_modules
                .get(source)
                .expect("expected a module or source");
              module_to_source_name
                .get(module_or_source)
                .expect("expected a filename at the given index but found None")
                .clone()
            })
            .collect::<Vec<_>>(),
        );
        if self.no_sources {
          source_map.set_sources_content([]);
        }
        if let Some(source_root) = &self.source_root {
          source_map.set_source_root(Some(source_root.clone()));
        }
      }
    }

    let futures = mapped_sources
      .into_iter()
      .map(|(source_filename, source, source_map)| {
        async {
          let (source_map_json, debug_id) = match source_map {
            Some(mut map) => {
              let debug_id = self.debug_ids.then(|| {
                let debug_id = create_debug_id(&source_filename, &source.buffer());
                map.set_debug_id(Some(debug_id.clone()));
                debug_id
              });

              (Some(map.to_json().into_diagnostic()?), debug_id)
            }
            None => (None, None),
          };

          let mut asset = compilation
            .assets()
            .get(&source_filename)
            .unwrap_or_else(|| {
              panic!(
                "expected to find filename '{}' in compilation.assets, but it was not present",
                &source_filename
              )
            })
            .clone();
          let Some(source_map_json) = source_map_json else {
            return Ok(MappedAsset {
              asset: (source_filename, asset),
              source_map: None,
            });
          };
          let css_extension_detected = CSS_EXTENSION_DETECT_REGEXP.is_match(&source_filename);
          let current_source_mapping_url_comment = match &self.source_mapping_url_comment {
            Some(SourceMappingUrlComment::String(s)) => {
              let s = if css_extension_detected {
                URL_FORMATTING_REGEXP.replace_all(s, "\n/*$1*/")
              } else {
                Cow::from(s)
              };
              Some(SourceMappingUrlCommentRef::String(s))
            }
            Some(SourceMappingUrlComment::Fn(f)) => Some(SourceMappingUrlCommentRef::Fn(f)),
            None => None,
          };

          if let Some(source_map_filename_config) = &self.source_map_filename {
            let chunk = file_to_chunk.get(&source_filename);
            let filename = match &self.file_context {
              Some(file_context) => Cow::Owned(
                Path::new(&source_filename)
                  .relative(Path::new(file_context))
                  .to_string_lossy()
                  .to_string(),
              ),
              None => Cow::Borrowed(&source_filename),
            };

            let mut hasher = RspackHash::from(&compilation.options.output);
            hasher.write(source_map_json.as_bytes());
            let digest = hasher.digest(&compilation.options.output.hash_digest);

            let data = PathData::default().filename(&filename);
            let data = match chunk {
              Some(chunk) => data
                .chunk_id_optional(
                  chunk
                    .id(&compilation.chunk_ids_artifact)
                    .map(|id| id.as_str()),
                )
                .chunk_hash_optional(chunk.rendered_hash(
                  &compilation.chunk_hashes_artifact,
                  compilation.options.output.hash_digest_length,
                ))
                .chunk_name_optional(
                  chunk.name_for_filename_template(&compilation.chunk_ids_artifact),
                )
                .content_hash_optional(Some(digest.encoded())),
              None => data,
            };
            let source_map_filename = compilation
              .get_asset_path(source_map_filename_config, data)
              .await?;

            if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
              let source_map_url = if let Some(public_path) = &self.public_path {
                format!("{public_path}{source_map_filename}")
              } else {
                let mut file_path = PathBuf::new();
                file_path.push(Component::RootDir);
                file_path.extend(Path::new(filename.as_ref()).components());

                let mut source_map_path = PathBuf::new();
                source_map_path.push(Component::RootDir);
                source_map_path.extend(Path::new(&source_map_filename).components());

                source_map_path
                  .relative(
                    #[allow(clippy::unwrap_used)]
                    file_path.parent().unwrap(),
                  )
                  .to_string_lossy()
                  .to_string()
              };
              let data = data.url(&source_map_url);
              let current_source_mapping_url_comment = match &current_source_mapping_url_comment {
                SourceMappingUrlCommentRef::String(s) => {
                  compilation
                    .get_asset_path(&Filename::from(s.as_ref()), data)
                    .await?
                }
                SourceMappingUrlCommentRef::Fn(f) => {
                  let comment = f(data).await?;
                  Filename::from(comment).render(data, None).await?
                }
              };
              let current_source_mapping_url_comment = current_source_mapping_url_comment
                .cow_replace("[url]", &source_map_url)
                .into_owned();

              let debug_id_comment = debug_id
                .map(|id| format!("\n//# debugId={id}"))
                .unwrap_or_default();

              asset.source = Some(
                ConcatSource::new([
                  source.clone(),
                  RawStringSource::from(debug_id_comment).boxed(),
                  RawStringSource::from(current_source_mapping_url_comment).boxed(),
                ])
                .boxed(),
              );
              asset.info.related.source_map = Some(source_map_filename.clone());
            } else {
              asset.source = Some(source.clone());
            }
            let mut source_map_asset_info = AssetInfo::default().with_development(Some(true));
            if let Some(asset) = compilation.assets().get(&source_filename) {
              // set source map asset version to be the same as the target asset
              source_map_asset_info.version = asset.info.version.clone();
            }
            let source_map_asset = CompilationAsset::new(
              Some(RawStringSource::from(source_map_json).boxed()),
              source_map_asset_info,
            );
            Ok(MappedAsset {
              asset: (source_filename, asset),
              source_map: Some((source_map_filename, source_map_asset)),
            })
          } else {
            let current_source_mapping_url_comment = current_source_mapping_url_comment.expect(
              "SourceMapDevToolPlugin: append can't be false when no filename is provided.",
            );
            let current_source_mapping_url_comment = match &current_source_mapping_url_comment {
              SourceMappingUrlCommentRef::String(s) => s,
              SourceMappingUrlCommentRef::Fn(_) => {
                return Err(error!(
                  "SourceMapDevToolPlugin: append can't be a function when no filename is provided"
                ))
              }
            };
            let base64 = rspack_base64::encode_to_string(source_map_json.as_bytes());
            asset.source = Some(
              ConcatSource::new([
                source.clone(),
                RawStringSource::from(
                  current_source_mapping_url_comment
                    .cow_replace(
                      "[url]",
                      &format!("data:application/json;charset=utf-8;base64,{base64}"),
                    )
                    .into_owned(),
                )
                .boxed(),
              ])
              .boxed(),
            );
            Ok(MappedAsset {
              asset: (source_filename, asset),
              source_map: None,
            })
          }
        }
      });
    join_all(futures).await.into_iter().collect()
  }
}

#[plugin_hook(CompilationProcessAssets for SourceMapDevToolPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_DEV_TOOLING)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.SourceMapDevToolPlugin");

  // use to read
  let mut file_to_chunk: HashMap<&String, &Chunk> = HashMap::default();
  // use to write
  let mut file_to_chunk_ukey: HashMap<String, ChunkUkey> = HashMap::default();
  for chunk in compilation.chunk_by_ukey.values() {
    for file in chunk.files() {
      file_to_chunk.insert(file, chunk);
      file_to_chunk_ukey.insert(file.to_string(), chunk.ukey());
    }
    for file in chunk.auxiliary_files() {
      file_to_chunk.insert(file, chunk);
      file_to_chunk_ukey.insert(file.to_string(), chunk.ukey());
    }
  }

  let start = logger.time("collect source maps");
  let raw_assets = compilation
    .assets()
    .iter()
    .filter(|(_filename, asset)| asset.info.related.source_map.is_none())
    .collect::<Vec<_>>();
  let mapped_asstes = self
    .mapped_assets_cache
    .use_cache(raw_assets, |assets| {
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
      if let Some((ref source_map_filename, _)) = source_map {
        source_asset.info.related.source_map = Some(source_map_filename.clone());
      }
    }

    let chunk_ukey = file_to_chunk_ukey.get(&source_filename);
    compilation.emit_asset(source_filename, source_asset);
    if let Some((source_map_filename, source_map_asset)) = source_map {
      compilation.emit_asset(source_map_filename.to_owned(), source_map_asset);

      let chunk = chunk_ukey.map(|ukey| compilation.chunk_by_ukey.expect_get_mut(ukey));
      if let Some(chunk) = chunk {
        chunk.add_auxiliary_file(source_map_filename);
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
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
