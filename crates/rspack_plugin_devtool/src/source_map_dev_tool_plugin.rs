use std::{
  borrow::Cow,
  hash::Hasher,
  path::{Component, Path, PathBuf},
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use derive_more::Debug;
use futures::future::BoxFuture;
use itertools::Itertools;
use regex::Regex;
use rspack_collections::DatabaseItem;
use rspack_core::{
  AssetInfo, Chunk, ChunkUkey, Compilation, CompilationAsset, CompilationProcessAssets, Filename,
  Logger, ModuleIdentifier, PathData, Plugin, has_content_hash_placeholder,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, ObjectPool, RawStringSource, Source, SourceExt, SourceMap,
  },
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{
  asset_condition::{AssetConditions, AssetConditionsObject, match_object},
  base64,
  identifier::make_paths_absolute,
  node_path::NodePath,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sugar_path::SugarPath;
use thread_local::ThreadLocal;

use crate::{
  ModuleFilenameTemplateFn, SourceReference, generate_debug_id::generate_debug_id,
  mapped_assets_cache::MappedAssetsCache, module_filename_helpers::ModuleFilenameHelpers,
};

/// Check if a string starts with `data:`, `http:`, or `https:`
fn is_schema_source(s: &str) -> bool {
  s.starts_with("data:") || s.starts_with("http:") || s.starts_with("https:")
}

/// Check if a string ends with `.css` or contains `.css?`
fn has_css_extension(s: &str) -> bool {
  s.ends_with(".css") || s.contains(".css?")
}

/// Compute source references from a source map's sources list.
fn compute_source_references(
  compilation: &Compilation,
  source_map: &SourceMap,
) -> Vec<SourceReference> {
  source_map
    .sources()
    .iter()
    .map(|source_name| {
      if let Some(stripped) = source_name.strip_prefix("webpack://") {
        let source_name = make_paths_absolute(compilation.options.context.as_str(), stripped);
        let identifier = ModuleIdentifier::from(source_name.as_str());
        match compilation
          .get_module_graph()
          .module_by_identifier(&identifier)
        {
          Some(module) => SourceReference::Module(module.identifier()),
          None => SourceReference::Source(Arc::from(source_name)),
        }
      } else {
        SourceReference::Source(Arc::from(source_name.clone()))
      }
    })
    .collect()
}

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
  // Decide whether to ignore source files that match the specified value in the SourceMap.
  pub ignore_list: Option<AssetConditions>,
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

enum SourceMappingUrlComment {
  String(String),
  Fn(AppendFn),
}

enum SourceMappingUrlCommentRef<'a> {
  String(Cow<'a, str>),
  Fn(&'a AppendFn),
}

struct SourceMapTask {
  pub asset_filename: Arc<str>,
  pub source: BoxSource,
  pub source_map: SourceMap,
  pub source_references: Vec<SourceReference>,
}

#[derive(Debug, Clone)]
pub(crate) struct MappedAsset {
  pub(crate) asset: (Arc<str>, CompilationAsset),
  pub(crate) source_map: Option<(String, CompilationAsset)>,
}

type TaskAndSourceNames = (
  SourceMapTask,
  Vec<(SourceReference, (String, Option<Utf8PathBuf>))>,
);

#[plugin]
#[derive(Debug)]
pub struct SourceMapDevToolPlugin {
  source_map_filename: Option<Filename>,
  ignore_list: Option<AssetConditions>,
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
      options.ignore_list,
      source_mapping_url_comment,
      options.file_context,
      module_filename_template,
      fallback_module_filename_template,
      options.namespace.unwrap_or_default(),
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

  // Only used when resolving [relative-resource-path].
  // It does not provide values for placeholders, so no rendering is performed here.
  async fn get_unresolved_source_map_path(
    &self,
    compilation: &Compilation,
    output_path: &Utf8Path,
    asset_filename: &str,
  ) -> Result<Option<Utf8PathBuf>> {
    match self.source_map_filename.as_ref() {
      Some(template) => {
        let filename = match &self.file_context {
          Some(file_context) => Cow::Owned(
            Path::new(asset_filename)
              .relative(Path::new(file_context))
              .to_string_lossy()
              .to_string(),
          ),
          None => Cow::Borrowed(asset_filename),
        };

        let data = PathData::default().filename(&filename);
        // The SourceMapDevToolPlugin 'filename' option is a plain string
        let filename = compilation.get_asset_path(template, data).await?;
        Ok(Some(output_path.node_join(filename.as_str())))
      }
      None => Ok(None),
    }
  }

  async fn map_assets(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    output_path: &Utf8Path,
    compilation_assets: Vec<(String, &CompilationAsset)>,
  ) -> Result<Vec<MappedAsset>> {
    let map_options = MapOptions::new(self.columns);

    let (tasks, mut reference_to_source_name_mapping) = self
      .collect_tasks(
        compilation,
        file_to_chunk,
        output_path,
        compilation_assets,
        &map_options,
      )
      .await?;

    self
      .deduplicate_source_names(compilation, &mut reference_to_source_name_mapping)
      .await?;

    self
      .compute_mapped_assets(
        compilation,
        file_to_chunk,
        &reference_to_source_name_mapping,
        tasks,
      )
      .await
  }

  /// Compute source maps and source names concurrently via `rspack_futures::scope`.
  /// Returns collected tasks and the reference-to-source-name mapping.
  async fn collect_tasks(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    output_path: &Utf8Path,
    compilation_assets: Vec<(String, &CompilationAsset)>,
    map_options: &MapOptions,
  ) -> Result<(
    Vec<SourceMapTask>,
    HashMap<SourceReference, (String, Option<Utf8PathBuf>)>,
  )> {
    let need_match = self.test.is_some() || self.include.is_some() || self.exclude.is_some();
    let condition_object = AssetConditionsObject {
      test: self.test.as_ref(),
      include: self.include.as_ref(),
      exclude: self.exclude.as_ref(),
    };
    let tls = ThreadLocal::new();

    let results: Vec<Result<Option<TaskAndSourceNames>>> = match &self.module_filename_template {
      ModuleFilenameTemplate::String(template) => {
        rspack_futures::scope::<_, Result<Option<TaskAndSourceNames>>>(|token| {
          for (asset_filename, asset) in compilation_assets {
            let is_match = if need_match {
              match_object(&condition_object, &asset_filename)
            } else {
              true
            };
            if !is_match {
              continue;
            }
            let source = match asset.get_source() {
              Some(s) => s.clone(),
              None => continue,
            };

            let map_options = map_options.clone();
            let s = unsafe {
              token.used((
                self,
                compilation,
                file_to_chunk,
                output_path,
                template,
                &tls,
              ))
            };
            s.spawn(
              |(plugin, compilation, file_to_chunk, output_path, template, tls)| async move {
                let source_map = {
                  let object_pool = tls.get_or(ObjectPool::default);
                  match source.map(object_pool, &map_options) {
                    Some(sm) => sm,
                    None => return Ok(None),
                  }
                };

                let source_references = compute_source_references(compilation, &source_map);

                let asset_filename: Arc<str> = Arc::from(asset_filename);
                let unresolved_source_map_path = plugin
                  .get_unresolved_source_map_path(compilation, output_path, &asset_filename)
                  .await?;

                let chunk = file_to_chunk.get(asset_filename.as_ref());
                let path_data = PathData::default()
                  .chunk_id_optional(chunk.and_then(|c| c.id().map(|id| id.as_str())))
                  .chunk_name_optional(chunk.and_then(|c| c.name()))
                  .chunk_hash_optional(chunk.and_then(|c| {
                    c.rendered_hash(
                      &compilation.chunk_hashes_artifact,
                      compilation.options.output.hash_digest_length,
                    )
                  }));

                let filename = Filename::from(plugin.namespace.clone());
                let namespace = compilation.get_path(&filename, path_data).await?;

                let mut source_name_entries = Vec::with_capacity(source_references.len());
                for source_reference in &source_references {
                  if let SourceReference::Source(sn) = source_reference
                    && is_schema_source(sn.as_ref())
                  {
                    source_name_entries.push((
                      source_reference.clone(),
                      (sn.to_string(), unresolved_source_map_path.clone()),
                    ));
                    continue;
                  }

                  let source_name = ModuleFilenameHelpers::create_filename_of_string_template(
                    source_reference,
                    compilation,
                    template,
                    &compilation.options.output,
                    &namespace,
                    unresolved_source_map_path.as_ref().map(|p| p.as_path()),
                  );
                  source_name_entries.push((
                    source_reference.clone(),
                    (source_name, unresolved_source_map_path.clone()),
                  ));
                }

                let task = SourceMapTask {
                  asset_filename,
                  source,
                  source_map,
                  source_references,
                };

                Ok(Some((task, source_name_entries)))
              },
            );
          }
        })
        .await
        .into_iter()
        .map(|r| r.to_rspack_result().flatten())
        .collect::<Vec<_>>()
      }
      ModuleFilenameTemplate::Fn(f) => {
        rspack_futures::scope::<_, Result<Option<TaskAndSourceNames>>>(|token| {
          for (asset_filename, asset) in compilation_assets {
            let is_match = if need_match {
              match_object(&condition_object, &asset_filename)
            } else {
              true
            };
            if !is_match {
              continue;
            }
            let source = match asset.get_source() {
              Some(s) => s.clone(),
              None => continue,
            };

            let asset_filename: Arc<str> = Arc::from(asset_filename);
            let map_options = map_options.clone();
            let s = unsafe {
              token.used((
                self,
                compilation,
                output_path,
                f,
                source,
                asset_filename,
                &tls,
              ))
            };
            s.spawn(
              |(plugin, compilation, output_path, f, source, asset_filename, tls)| async move {
                let source_map = {
                  let object_pool = tls.get_or(ObjectPool::default);
                  match source.map(object_pool, &map_options) {
                    Some(sm) => sm,
                    None => return Ok(None),
                  }
                };

                let source_references = compute_source_references(compilation, &source_map);

                let mut source_name_entries = Vec::with_capacity(source_references.len());
                for source_reference in &source_references {
                  let unresolved_source_map_path = plugin
                    .get_unresolved_source_map_path(compilation, output_path, &asset_filename)
                    .await?;

                  if let SourceReference::Source(sn) = source_reference
                    && is_schema_source(sn.as_ref())
                  {
                    source_name_entries.push((
                      source_reference.clone(),
                      (sn.to_string(), unresolved_source_map_path),
                    ));
                    continue;
                  }

                  let source_name = ModuleFilenameHelpers::create_filename_of_fn_template(
                    source_reference,
                    compilation,
                    f,
                    &compilation.options.output,
                    &plugin.namespace,
                    unresolved_source_map_path.as_ref().map(|p| p.as_path()),
                  )
                  .await?;
                  source_name_entries.push((
                    source_reference.clone(),
                    (source_name, unresolved_source_map_path),
                  ));
                }

                let task = SourceMapTask {
                  asset_filename,
                  source,
                  source_map,
                  source_references,
                };

                Ok(Some((task, source_name_entries)))
              },
            );
          }
        })
        .await
        .into_iter()
        .map(|r| r.to_rspack_result().flatten())
        .collect::<Vec<_>>()
      }
    };

    let mut tasks: Vec<SourceMapTask> = Vec::with_capacity(results.len());
    let mut reference_to_source_name_mapping: HashMap<
      SourceReference,
      (String, Option<Utf8PathBuf>),
    > = HashMap::default();

    for result in results {
      let Some((task, source_name_entries)) = result? else {
        continue;
      };
      for (source_ref, name_entry) in source_name_entries {
        reference_to_source_name_mapping.insert(source_ref, name_entry);
      }
      tasks.push(task);
    }

    Ok((tasks, reference_to_source_name_mapping))
  }

  /// Deduplicate source names by trying the fallback template and appending '*' if needed.
  async fn deduplicate_source_names(
    &self,
    compilation: &Compilation,
    reference_to_source_name_mapping: &mut HashMap<SourceReference, (String, Option<Utf8PathBuf>)>,
  ) -> Result<()> {
    let output_options = &compilation.options.output;
    let mut used_names_set = HashSet::<&String>::default();
    for (source_reference, (source_name, unresolved_source_map_path)) in
      reference_to_source_name_mapping
        .iter_mut()
        .sorted_by(|(key_a, _), (key_b, _)| {
          let ident_a = match key_a {
            SourceReference::Module(identifier) => identifier,
            SourceReference::Source(source) => source.as_ref(),
          };
          let ident_b = match key_b {
            SourceReference::Module(identifier) => identifier,
            SourceReference::Source(source) => source.as_ref(),
          };
          ident_a.len().cmp(&ident_b.len())
        })
    {
      let mut has_name = used_names_set.contains(source_name);
      if !has_name {
        used_names_set.insert(source_name);
        continue;
      }

      let mut new_source_name = match &self.fallback_module_filename_template {
        ModuleFilenameTemplate::String(s) => {
          ModuleFilenameHelpers::create_filename_of_string_template(
            source_reference,
            compilation,
            s,
            output_options,
            self.namespace.as_str(),
            unresolved_source_map_path.as_ref().map(|p| p.as_path()),
          )
        }
        ModuleFilenameTemplate::Fn(f) => {
          ModuleFilenameHelpers::create_filename_of_fn_template(
            source_reference,
            compilation,
            f,
            output_options,
            self.namespace.as_str(),
            unresolved_source_map_path.as_ref().map(|p| p.as_path()),
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

    Ok(())
  }

  /// Update source_maps with deduplicated names and compute MappedAssets.
  async fn compute_mapped_assets(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    reference_to_source_name_mapping: &HashMap<SourceReference, (String, Option<Utf8PathBuf>)>,
    tasks: Vec<SourceMapTask>,
  ) -> Result<Vec<MappedAsset>> {
    let mapped_assets = rspack_futures::scope::<_, Result<_>>(|token| {
      tasks.into_iter().for_each(
        |SourceMapTask {
           asset_filename,
           source,
           source_map,
           source_references,
         }| {
          let s = unsafe {
            token.used((
              &self,
              compilation,
              file_to_chunk,
              reference_to_source_name_mapping,
              asset_filename,
              source,
              source_map,
              source_references,
            ))
          };
          s.spawn(
            |(
              plugin,
              compilation,
              file_to_chunk,
              reference_to_source_name_mapping,
              asset_filename,
              source,
              source_map,
              source_references,
            )| async move {
              Self::create_mapped_asset(
                plugin,
                compilation,
                file_to_chunk,
                reference_to_source_name_mapping,
                asset_filename,
                source,
                source_map,
                source_references,
              )
              .await
            },
          );
        },
      );
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    mapped_assets.into_iter().collect::<Result<Vec<_>>>()
  }

  /// Create a single MappedAsset: update the source map and emit asset + optional source map file.
  #[allow(clippy::too_many_arguments)]
  async fn create_mapped_asset(
    plugin: &SourceMapDevToolPlugin,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    reference_to_source_name_mapping: &HashMap<SourceReference, (String, Option<Utf8PathBuf>)>,
    asset_filename: Arc<str>,
    source: BoxSource,
    mut source_map: SourceMap,
    source_references: Vec<SourceReference>,
  ) -> Result<MappedAsset> {
    // Update source_map with deduplicated source names
    source_map.set_file(Some(asset_filename.clone()));
    source_map.set_sources(
      source_references
        .iter()
        .map(|source_reference| {
          reference_to_source_name_mapping
            .get(source_reference)
            .unwrap_or_else(|| {
              panic!(
                "SourceMapDevToolPlugin: missing source name for reference '{source_reference:?}' in asset '{asset_filename}'."
              )
            })
            .0
            .clone()
        })
        .collect::<Vec<_>>(),
    );

    if let Some(asset_conditions) = &plugin.ignore_list {
      let ignore_list = source_map
        .sources()
        .iter()
        .enumerate()
        .filter_map(|(idx, source)| {
          if asset_conditions.try_match(source) {
            Some(idx as u32)
          } else {
            None
          }
        })
        .collect::<Vec<_>>();
      source_map.set_ignore_list(Some(ignore_list));
    }

    if plugin.no_sources {
      source_map.set_sources_content([]);
    }
    if let Some(source_root) = &plugin.source_root {
      source_map.set_source_root(Some(source_root.clone()));
    }

    // Generate debug ID and serialize source map
    let debug_id = plugin.debug_ids.then(|| {
      let debug_id = generate_debug_id(&asset_filename, &source.buffer());
      source_map.set_debug_id(Some(debug_id.clone()));
      debug_id
    });
    let source_map_json = source_map.to_json();

    let mut asset = compilation
      .assets()
      .get(asset_filename.as_ref())
      .unwrap_or_else(|| {
        panic!(
          "expected to find filename '{}' in compilation.assets, but it was not present",
          asset_filename.as_ref()
        )
      })
      .clone();

    let css_extension_detected = has_css_extension(&asset_filename);
    let current_source_mapping_url_comment = match &plugin.source_mapping_url_comment {
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

    if let Some(source_map_filename_config) = &plugin.source_map_filename {
      let chunk = file_to_chunk.get(asset_filename.as_ref());
      let filename = match &plugin.file_context {
        Some(file_context) => Cow::Owned(
          Path::new(asset_filename.as_ref())
            .relative(Path::new(file_context))
            .to_string_lossy()
            .to_string(),
        ),
        None => Cow::Borrowed(asset_filename.as_ref()),
      };

      let content_hash_digest =
        if chunk.is_some() && has_content_hash_placeholder(source_map_filename_config.as_str()) {
          let mut hasher = RspackHash::from(&compilation.options.output);
          hasher.write(source_map_json.as_bytes());
          let digest = hasher.digest(&compilation.options.output.hash_digest);
          Some(digest)
        } else {
          None
        };

      let data = PathData::default().filename(&filename);
      let data = match chunk {
        Some(chunk) => data
          .chunk_id_optional(chunk.id().map(|id| id.as_str()))
          .chunk_hash_optional(chunk.rendered_hash(
            &compilation.chunk_hashes_artifact,
            compilation.options.output.hash_digest_length,
          ))
          .chunk_name_optional(chunk.name_for_filename_template())
          .content_hash_optional(content_hash_digest.as_ref().map(|digest| digest.encoded())),
        None => data,
      };
      let source_map_filename = compilation
        .get_asset_path(source_map_filename_config, data)
        .await?;

      if let Some(current_source_mapping_url_comment) = current_source_mapping_url_comment {
        let source_map_url = if let Some(public_path) = &plugin.public_path {
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
      if let Some(asset) = compilation.assets().get(asset_filename.as_ref()) {
        // set source map asset version to be the same as the target asset
        source_map_asset_info.version = asset.info.version.clone();
      }
      let source_map_asset = CompilationAsset::new(
        Some(RawStringSource::from(source_map_json).boxed()),
        source_map_asset_info,
      );
      Ok(MappedAsset {
        asset: (asset_filename, asset),
        source_map: Some((source_map_filename.clone(), source_map_asset)),
      })
    } else {
      let current_source_mapping_url_comment = current_source_mapping_url_comment
        .expect("SourceMapDevToolPlugin: append can't be false when no filename is provided.");
      let current_source_mapping_url_comment = match &current_source_mapping_url_comment {
        SourceMappingUrlCommentRef::String(s) => s,
        SourceMappingUrlCommentRef::Fn(_) => {
          return Err(error!(
            "SourceMapDevToolPlugin: append can't be a function when no filename is provided"
          ));
        }
      };
      let base64 = base64::encode_to_string(source_map_json.as_bytes());
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
        asset: (asset_filename, asset),
        source_map: None,
      })
    }
  }
}

#[plugin_hook(CompilationProcessAssets for SourceMapDevToolPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_DEV_TOOLING)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.SourceMapDevToolPlugin");

  // use to read
  let mut file_to_chunk: HashMap<&str, &Chunk> = HashMap::default();
  // use to write
  let mut file_to_chunk_ukey: HashMap<String, ChunkUkey> = HashMap::default();
  for chunk in compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
  {
    for file in chunk.files() {
      file_to_chunk.insert(file, chunk);
      file_to_chunk_ukey.insert(file.clone(), chunk.ukey());
    }
    for file in chunk.auxiliary_files() {
      file_to_chunk.insert(file, chunk);
      file_to_chunk_ukey.insert(file.clone(), chunk.ukey());
    }
  }

  // When computing [relative-resource-path], we need the absolute path of the emitted source map file.
  // Use the output path to resolve the source map location against the asset directory.
  let output_path = Utf8PathBuf::from(
    compilation
      .get_path(
        &Filename::from(&compilation.options.output.path),
        Default::default(),
      )
      .await?,
  );

  let start = logger.time("collect source maps");
  let compilation_assets = compilation
    .assets()
    .iter()
    .filter(|(_filename, asset)| asset.info.related.source_map.is_none());
  let mapped_asstes = self
    .mapped_assets_cache
    .use_cache(compilation_assets, |assets| {
      self.map_assets(compilation, &file_to_chunk, &output_path, assets)
    })
    .await?;
  logger.time_end(start);

  let start = logger.time("emit source map assets");
  for mapped_asset in mapped_asstes {
    let MappedAsset {
      asset: (source_filename, mut source_asset),
      source_map,
    } = mapped_asset;
    if let Some(asset) = compilation.assets_mut().remove(source_filename.as_ref()) {
      source_asset.info = asset.info;
      if let Some((ref source_map_filename, _)) = source_map {
        source_asset.info.related.source_map = Some(source_map_filename.clone());
      }
    }

    let chunk_ukey = file_to_chunk_ukey.get(source_filename.as_ref());
    compilation.emit_asset(source_filename.to_string(), source_asset);
    if let Some((source_map_filename, source_map_asset)) = source_map {
      compilation.emit_asset(source_map_filename.clone(), source_map_asset);

      let chunk = chunk_ukey.map(|ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get_mut(ukey)
      });
      if let Some(chunk) = chunk {
        chunk.add_auxiliary_file(source_map_filename.clone());
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

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
