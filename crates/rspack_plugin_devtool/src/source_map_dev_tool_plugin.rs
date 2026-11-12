use std::{
  borrow::Cow,
  hash::Hash,
  path::{Component, Path, PathBuf},
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use derive_more::Debug;
use futures::future::BoxFuture;
use itertools::Itertools;
use regex::Regex;
use rspack_core::{
  AssetInfo, CacheCount, CacheFacade, CacheValue, Chunk, ChunkUkey, Compilation, CompilationAsset,
  CompilationParams, CompilationProcessAssets, CompilerCompilation, Etag, Filename, Logger,
  ModuleIdentifier, PathData, Plugin,
  cache::CachedSourceMapDevToolPluginEntry,
  legacy_cache::persistent::occasion::SourceMapDevToolPluginCache,
  rspack_sources::{
    BoxSource, DevToolCodeSource, DevToolMapSource, MapOptions, RawStringSource, Source, SourceExt,
    SourceMap, update_content_hash,
  },
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hash::RspackHasher;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{
  asset_condition::{AssetConditions, AssetConditionsObject, match_object},
  base64,
  fx_hash::FxIndexMap,
  identifier::make_paths_absolute,
  node_path::NodePath,
  source_map::SourceMapKind,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use sugar_path::SugarPath;
use url::Url;

use crate::{
  ModuleFilenameTemplateFn, SourceReference, default_source_map_fallback_module_filename_template,
  default_source_map_module_filename_template, generate_debug_id::generate_debug_id,
  module_filename_helpers::ModuleFilenameHelpers,
};

/// Check if a string starts with `data:`, `http:`, or `https:`
fn is_schema_source(s: &str) -> bool {
  s.starts_with("data:") || s.starts_with("http:") || s.starts_with("https:")
}

/// Check if a string ends with `.css` or contains `.css?`
fn has_css_extension(s: &str) -> bool {
  s.ends_with(".css") || s.contains(".css?")
}

fn starts_with_url_scheme(s: &str) -> bool {
  let Some((scheme, _)) = s.split_once(':') else {
    return false;
  };
  let mut chars = scheme.chars();
  matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
    && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn starts_with_windows_drive_letter(s: &str) -> bool {
  let bytes = s.as_bytes();
  bytes.len() >= 2
    && bytes[0].is_ascii_alphabetic()
    && matches!(bytes[1], b':' | b'|')
    && (bytes.len() == 2 || matches!(bytes[2], b'/' | b'\\' | b'?' | b'#'))
}

fn is_relative_source_name(s: &str) -> bool {
  // Source map `sources` entries are URL references. Per ECMA-426
  // "9 Source map format", each source is a potentially relative URL; per
  // "9.3 Resolving sources", non-absolute sources are resolved against the
  // source map URL.
  //
  // WHATWG URL "4.3 URL writing" gives the useful forms here:
  // - absolute URLs: `webpack://...`, `file://...`, `data:...`
  // - scheme-relative URLs: `//host/path`
  // - path-absolute URLs: `/path`
  // - path-relative, scheme-less URLs: `index.js`, `./index.js`, `../index.js`
  //
  // Only path-relative, scheme-less URLs depend on the source map location, so
  // only they need to be canonicalized before names are deduplicated across
  // assets.
  //
  // WHATWG URL "4.2 URL miscellaneous" defines "Windows drive letter",
  // "normalized Windows drive letter", and "starts with a Windows drive
  // letter"; keep those Windows absolute paths (`C:/...`, `C:\...`, `C|/...`)
  // and backslash-rooted Windows paths (`\\server\share`) verbatim.
  !s.is_empty()
    && !s.starts_with(['/', '\\'])
    && !starts_with_url_scheme(s)
    && !starts_with_windows_drive_letter(s)
}

fn normalize_relative_source_name_url(
  source_name: &str,
  source_map_path: Option<&Utf8Path>,
) -> Option<String> {
  let source_map_path = source_map_path?;
  let base_url = Url::from_file_path(source_map_path.as_std_path()).ok()?;
  base_url.join(source_name).ok().map(|url| url.to_string())
}

fn relative_source_name_from_url(
  source_url: &str,
  source_map_path: Option<&Utf8Path>,
) -> Option<String> {
  let source_map_path = source_map_path?;
  let source_map_url = Url::from_file_path(source_map_path.as_std_path()).ok()?;
  let source_url = Url::parse(source_url).ok()?;
  source_map_url.make_relative(&source_url)
}

/// Compute source references from a source map's sources list.
fn compute_source_references(
  compilation: &Compilation,
  sources: &[String],
) -> Vec<SourceReference> {
  sources
    .iter()
    .map(|source_name| {
      if let Some(stripped) = source_name
        .strip_prefix("webpack://")
        .or_else(|| source_name.strip_prefix("rspack://"))
      {
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
        SourceReference::Source(Arc::from(source_name.as_str()))
      }
    })
    .collect()
}

static URL_FORMATTING_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^\n\/\/(.*)$").expect("failed to compile URL_FORMATTING_REGEXP regex")
});

const PLUGIN_NAME: &str = "rspack.SourceMapDevToolPlugin";

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

impl SourceMapDevToolPluginOptions {
  pub fn module_source_map_kind(&self) -> SourceMapKind {
    SourceMapKind::from_module(self.module)
      .with_cheap(!self.columns)
      .with_no_sources(self.no_sources)
      .with_inline(self.filename.is_none())
  }
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
  pub source_map: Option<SourceMap>,
  pub input_sources: Arc<[String]>,
  pub unresolved_source_map_path: Option<Utf8PathBuf>,
  pub source_references: Vec<SourceReference>,
}

struct PreparedSourceMaps {
  reuse_all: bool,
  tasks: Vec<SourceMapTask>,
  source_names: ReferenceToSourceNameMapping,
  cache_versions: HashMap<String, String>,
  input_sources: HashMap<String, Arc<[String]>>,
  compilation_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceName {
  /// A source name that should be emitted exactly as generated.
  Verbatim(String),
  /// A canonical resource URL used for deduplication, rendered relative to each source map.
  CanonicalResourceUrl(String),
}

impl SourceName {
  fn new(source_name: String, source_map_path: Option<&Utf8Path>) -> Self {
    if is_relative_source_name(&source_name)
      && let Some(source_url) = normalize_relative_source_name_url(&source_name, source_map_path)
    {
      return Self::CanonicalResourceUrl(source_url);
    }

    Self::Verbatim(source_name)
  }

  fn push(&mut self, ch: char) {
    match self {
      SourceName::Verbatim(source_name) | SourceName::CanonicalResourceUrl(source_name) => {
        source_name.push(ch)
      }
    }
  }
}

#[derive(Debug, Clone)]
struct SourceNameWithBaseUrl {
  source_name: SourceName,
  unresolved_source_map_path: Option<Utf8PathBuf>,
}

impl SourceNameWithBaseUrl {
  fn new(source_name: String, unresolved_source_map_path: Option<Utf8PathBuf>) -> Self {
    let source_name = SourceName::new(
      source_name,
      unresolved_source_map_path.as_ref().map(|p| p.as_path()),
    );
    Self {
      source_name,
      unresolved_source_map_path,
    }
  }

  fn render_for_source_map<'a>(&'a self, source_map_path: Option<&Utf8Path>) -> Cow<'a, str> {
    match &self.source_name {
      SourceName::Verbatim(source_name) => Cow::Borrowed(source_name),
      SourceName::CanonicalResourceUrl(source_url) => {
        relative_source_name_from_url(source_url, source_map_path)
          .map_or_else(|| Cow::Borrowed(source_url.as_str()), Cow::Owned)
      }
    }
  }
}

type ReferenceToSourceNameMapping = FxIndexMap<SourceReference, SourceNameWithBaseUrl>;

type TaskAndSourceNames = (SourceMapTask, Vec<(SourceReference, SourceNameWithBaseUrl)>);

#[derive(Debug, Clone)]
pub(crate) struct MappedAsset {
  pub(crate) asset: (Arc<str>, CompilationAsset),
  pub(crate) asset_append: Vec<BoxSource>,
  pub(crate) source_map: Option<(String, CompilationAsset)>,
}

#[plugin]
#[derive(Debug)]
pub struct SourceMapDevToolPlugin {
  source_map_filename: Option<Filename>,
  ignore_list: Option<AssetConditions>,
  #[debug(skip)]
  source_mapping_url_comment: Option<SourceMappingUrlComment>,
  file_context: Option<String>,
  #[debug(skip)]
  module_filename_template: Option<ModuleFilenameTemplate>,
  #[debug(skip)]
  fallback_module_filename_template: Option<ModuleFilenameTemplate>,
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
}

impl SourceMapDevToolPlugin {
  pub fn new(options: SourceMapDevToolPluginOptions) -> Self {
    let SourceMapDevToolPluginOptions {
      append,
      columns,
      fallback_module_filename_template,
      file_context,
      filename,
      ignore_list,
      module,
      module_filename_template,
      namespace,
      no_sources,
      public_path,
      source_root,
      test,
      include,
      exclude,
      debug_ids,
    } = options;

    let source_mapping_url_comment = match append {
      Some(append) => match append {
        Append::String(s) => Some(SourceMappingUrlComment::String(s)),
        Append::Fn(f) => Some(SourceMappingUrlComment::Fn(f)),
        Append::Disabled => None,
      },
      None => Some(SourceMappingUrlComment::String(
        "\n//# sourceMappingURL=[url]".to_string(),
      )),
    };

    let source_map_filename = filename.map(Filename::from);
    let namespace = namespace.unwrap_or_default();
    let source_root = source_root.map(Arc::from);
    Self::new_inner(
      source_map_filename,
      ignore_list,
      source_mapping_url_comment,
      file_context,
      module_filename_template,
      fallback_module_filename_template,
      namespace,
      columns,
      no_sources,
      public_path,
      module,
      source_root,
      test,
      include,
      exclude,
      debug_ids,
    )
  }

  // Only used when resolving [relative-resource-path].
  // It does not provide values for placeholders, so no rendering is performed here.
  // External source maps use the source map file path as the base; inline source maps
  // use the emitted asset path as the base because the source map lives in that asset.
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
      None => Ok(Some(output_path.node_join(asset_filename))),
    }
  }

  fn cache_version<'a>(
    compilation: &Compilation,
    context_version: &str,
    filenames: impl Iterator<Item = &'a str>,
  ) -> String {
    let mut hasher = RspackHasher::from(&compilation.options.output);
    context_version.hash(&mut hasher);
    for filename in filenames.sorted() {
      filename.hash(&mut hasher);
      compilation.assets()[filename]
        .info
        .version
        .hash(&mut hasher);
    }
    hasher
      .digest(&compilation.options.output.hash_digest)
      .encoded()
      .to_string()
  }

  #[allow(clippy::too_many_arguments)]
  async fn prepare_source_maps(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    output_path: &Utf8Path,
    compilation_assets: &[(String, &CompilationAsset)],
    cached_entries: &HashMap<String, Cow<'_, CachedSourceMapDevToolPluginEntry>>,
    // The legacy cache keeps one compilation version for all of its entries.
    legacy_version: Option<&str>,
    use_cache: bool,
    context_version: String,
  ) -> Result<PreparedSourceMaps> {
    let compilation_version = if use_cache {
      Self::cache_version(
        compilation,
        &context_version,
        cached_entries.keys().map(String::as_str),
      )
    } else {
      String::new()
    };
    let map_options = MapOptions::new(self.columns);
    let condition_object = AssetConditionsObject {
      test: self.test.as_ref(),
      include: self.include.as_ref(),
      exclude: self.exclude.as_ref(),
    };
    // Reuse resolved names when the collision context is unchanged and every map is cached.
    // Other assets may have no map (or be excluded), so they need no cache entry.
    if use_cache
      && !cached_entries.is_empty()
      && match legacy_version {
        Some(version) => version == compilation_version,
        None => cached_entries
          .values()
          .all(|entry| entry.compilation_version == compilation_version),
      }
      && compilation_assets.iter().all(|(filename, asset)| {
        cached_entries.contains_key(filename)
          || !match_object(&condition_object, filename)
          || asset
            .get_source()
            .is_none_or(|source| source.map(&map_options).is_none())
      })
    {
      return Ok(PreparedSourceMaps {
        reuse_all: true,
        tasks: Vec::new(),
        source_names: ReferenceToSourceNameMapping::default(),
        cache_versions: HashMap::default(),
        input_sources: HashMap::default(),
        compilation_version,
      });
    }
    // Name resolution is compilation-wide, including assets whose finished maps can be reused.
    let (tasks, mut source_names) = self
      .collect_tasks(
        compilation,
        file_to_chunk,
        output_path,
        compilation_assets.to_vec(),
        &map_options,
        cached_entries,
      )
      .await?;
    self
      .deduplicate_source_names(compilation, &mut source_names)
      .await?;

    let mut cache_versions = HashMap::default();
    for task in &tasks {
      let asset = &compilation.assets()[task.asset_filename.as_ref()];
      if !use_cache || asset.info.version.is_empty() {
        continue;
      }
      // The generated asset version alone cannot validate names chosen against other assets.
      // Hash this asset's resolved names so unrelated content changes still allow partial hits,
      // while adding or removing a competing source invalidates any affected finished map.
      let mut hasher = RspackHasher::from(&compilation.options.output);
      asset.info.version.hash(&mut hasher);
      for reference in &task.source_references {
        source_names[reference]
          .render_for_source_map(task.unresolved_source_map_path.as_deref())
          .hash(&mut hasher);
      }
      cache_versions.insert(
        task.asset_filename.to_string(),
        hasher
          .digest(&compilation.options.output.hash_digest)
          .encoded()
          .to_string(),
      );
    }
    let input_sources = tasks
      .iter()
      .filter(|_| use_cache)
      .map(|task| (task.asset_filename.to_string(), task.input_sources.clone()))
      .collect();
    let compilation_version = if use_cache {
      Self::cache_version(
        compilation,
        &context_version,
        tasks.iter().map(|task| task.asset_filename.as_ref()),
      )
    } else {
      String::new()
    };
    Ok(PreparedSourceMaps {
      reuse_all: false,
      tasks,
      source_names,
      cache_versions,
      input_sources,
      compilation_version,
    })
  }

  fn should_detach_source(&self, info: &AssetInfo) -> bool {
    // Keeping the code's input graph would retain inlined maps, omitted source contents,
    // or the minimizer's unminified input.
    self.source_map_filename.is_none() || self.no_sources || info.minimized == Some(true)
  }

  async fn use_new_cache(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    compilation_assets: Vec<(String, &CompilationAsset)>,
    prepared: PreparedSourceMaps,
    cache: &CacheFacade,
  ) -> Result<Vec<MappedAsset>> {
    let mut mapped_assets = Vec::with_capacity(compilation_assets.len());
    let mut cached_filenames = HashSet::default();

    for (filename, asset) in compilation_assets {
      let cacheable = asset.get_source().is_some() && !asset.info.version.is_empty();
      let cached = if cacheable {
        cache.get_validated::<CachedSourceMapDevToolPluginEntry>(
          &filename,
          Some(Etag::from(asset.info.version.clone())),
          |entry| {
            if prepared.reuse_all {
              entry.compilation_version == prepared.compilation_version
            } else {
              Some(&entry.name_version) == prepared.cache_versions.get(&filename)
            }
          },
        )
      } else {
        None
      };
      if let Some(cached) = cached
        && let Some((source_asset, source_map, asset_append)) =
          cached.restore(asset, self.should_detach_source(&asset.info))
      {
        cache.record_reuse();
        let filename: Arc<str> = Arc::from(filename);
        cached_filenames.insert(filename.clone());
        mapped_assets.push(MappedAsset {
          asset: (filename, source_asset),
          asset_append,
          source_map,
        });
      }
    }

    let tasks = prepared
      .tasks
      .into_iter()
      .filter(|task| !cached_filenames.contains(&task.asset_filename))
      .collect();
    let new_mapped_assets = self
      .compute_mapped_assets(compilation, file_to_chunk, &prepared.source_names, tasks)
      .await?;
    for mapped_asset in &new_mapped_assets {
      let asset = &mapped_asset.asset.1;
      if asset.get_source().is_none() || asset.info.version.is_empty() {
        continue;
      }
      let Some(mut cache_entry) = CachedSourceMapDevToolPluginEntry::from_assets(
        mapped_asset.asset_append.as_slice(),
        mapped_asset
          .source_map
          .as_ref()
          .map(|(filename, asset)| (filename.as_str(), asset)),
      ) else {
        continue;
      };
      let filename = mapped_asset.asset.0.as_ref();
      cache_entry.input_sources = prepared.input_sources[filename].clone();
      cache_entry.name_version = prepared.cache_versions[filename].clone();
      cache_entry.compilation_version = prepared.compilation_version.clone();
      cache.store(
        mapped_asset.asset.0.as_ref(),
        Some(Etag::from(asset.info.version.clone())),
        CacheValue::new(cache_entry),
      );
    }
    mapped_assets.extend(new_mapped_assets);
    Ok(mapped_assets)
  }

  async fn use_legacy_cache(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    compilation_assets: Vec<(String, &CompilationAsset)>,
    prepared: PreparedSourceMaps,
    cache: Option<&mut SourceMapDevToolPluginCache>,
    cache_counter: Option<&CacheCount>,
  ) -> Result<Vec<MappedAsset>> {
    let Some(cache) = cache else {
      return self
        .compute_mapped_assets(
          compilation,
          file_to_chunk,
          &prepared.source_names,
          prepared.tasks,
        )
        .await;
    };
    let mut mapped_assets = Vec::with_capacity(compilation_assets.len());
    let mut cached_filenames = HashSet::default();
    for (filename, asset) in compilation_assets {
      let cached = prepared
        .cache_versions
        .get(&filename)
        .and_then(|name_version| {
          cache.restore(
            &filename,
            asset,
            &asset.info.version,
            name_version,
            self.should_detach_source(&asset.info),
          )
        });
      if let Some((source_asset, source_map, asset_append)) = cached {
        if let Some(counter) = cache_counter {
          counter.hit();
        }
        let filename: Arc<str> = Arc::from(filename);
        cached_filenames.insert(filename.clone());
        mapped_assets.push(MappedAsset {
          asset: (filename, source_asset),
          asset_append,
          source_map,
        });
      } else if let Some(counter) = cache_counter {
        counter.miss();
      }
    }

    let PreparedSourceMaps {
      reuse_all: _,
      tasks,
      source_names,
      cache_versions,
      input_sources,
      compilation_version,
    } = prepared;
    let tasks = tasks
      .into_iter()
      .filter(|task| !cached_filenames.contains(&task.asset_filename))
      .collect();
    let new_mapped_assets = self
      .compute_mapped_assets(compilation, file_to_chunk, &source_names, tasks)
      .await?;
    mapped_assets.extend(new_mapped_assets);

    cache.store(
      &compilation_version,
      mapped_assets.iter().filter_map(|mapped_asset| {
        let filename = mapped_asset.asset.0.as_ref();
        let version = &mapped_asset.asset.1.info.version;
        let mut entry = CachedSourceMapDevToolPluginEntry::from_assets(
          &mapped_asset.asset_append,
          mapped_asset
            .source_map
            .as_ref()
            .map(|(filename, asset)| (filename.as_str(), asset)),
        )?;
        entry.input_sources = input_sources.get(filename)?.clone();
        entry.name_version = cache_versions.get(filename)?.clone();
        Some((filename, version.as_str(), entry))
      }),
    );
    Ok(mapped_assets)
  }

  /// Compute source maps and source names concurrently via `rspack_parallel::scope`.
  /// Returns collected tasks and the reference-to-source-name mapping.
  async fn collect_tasks(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    output_path: &Utf8Path,
    compilation_assets: Vec<(String, &CompilationAsset)>,
    map_options: &MapOptions,
    cached_entries: &HashMap<String, Cow<'_, CachedSourceMapDevToolPluginEntry>>,
  ) -> Result<(Vec<SourceMapTask>, ReferenceToSourceNameMapping)> {
    let default_module_filename_template =
      default_source_map_module_filename_template(compilation.options.experiments.runtime_mode);
    let module_filename_template = self
      .module_filename_template
      .as_ref()
      .unwrap_or(default_module_filename_template);

    let results: Vec<Result<Option<TaskAndSourceNames>>> = match module_filename_template {
      ModuleFilenameTemplate::String(template) => rspack_parallel::scope::<
        _,
        Result<Option<TaskAndSourceNames>>,
      >(|token| {
        for (asset_filename, asset) in compilation_assets {
          let source = match asset.get_source() {
            Some(s) => s.clone(),
            None => continue,
          };

          let cached_sources = cached_entries
            .get::<str>(&asset_filename)
            .map(|entry| entry.input_sources.clone());
          let map_options = map_options.clone();
          let s = unsafe { token.used((self, compilation, file_to_chunk, output_path, template)) };
          s.spawn(
            |(plugin, compilation, file_to_chunk, output_path, template)| async move {
              let (source_map, input_sources) = if let Some(sources) = cached_sources {
                (None, sources)
              } else {
                let Some(map) = source.map(&map_options) else {
                  return Ok(None);
                };
                let sources = map
                  .sources()
                  .iter()
                  .map(ToString::to_string)
                  .collect::<Vec<_>>();
                (Some(map), sources.into())
              };
              let source_references = compute_source_references(compilation, &input_sources);

              let asset_filename: Arc<str> = Arc::from(asset_filename);
              let unresolved_source_map_path = plugin
                .get_unresolved_source_map_path(compilation, output_path, &asset_filename)
                .await?;

              let chunk = file_to_chunk.get(asset_filename.as_ref());
              let path_data = match chunk {
                Some(chunk) => PathData::default()
                  .chunk(chunk.ukey(), compilation)
                  .chunk_id_optional(chunk.id().map(|id| id.as_str()))
                  .chunk_name_optional(chunk.name())
                  .chunk_hash_optional(chunk.rendered_hash(
                    &compilation.chunk_hashes_artifact,
                    compilation.options.output.hash_digest_length,
                  )),
                None => PathData::default(),
              };

              let filename = Filename::from(plugin.namespace.clone());
              let namespace = compilation.get_path(&filename, path_data).await?;

              let mut source_name_entries = Vec::with_capacity(source_references.len());
              for source_reference in &source_references {
                if let SourceReference::Source(sn) = source_reference
                  && is_schema_source(sn.as_ref())
                {
                  source_name_entries.push((
                    source_reference.clone(),
                    SourceNameWithBaseUrl::new(sn.to_string(), unresolved_source_map_path.clone()),
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
                  SourceNameWithBaseUrl::new(source_name, unresolved_source_map_path.clone()),
                ));
              }

              let task = SourceMapTask {
                asset_filename,
                source,
                source_map,
                input_sources,
                unresolved_source_map_path,
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
      .collect::<Vec<_>>(),
      ModuleFilenameTemplate::Fn(f) => rspack_parallel::scope::<
        _,
        Result<Option<TaskAndSourceNames>>,
      >(|token| {
        for (asset_filename, asset) in compilation_assets {
          let source = match asset.get_source() {
            Some(s) => s.clone(),
            None => continue,
          };

          let asset_filename: Arc<str> = Arc::from(asset_filename);
          let cached_sources = cached_entries
            .get::<str>(&asset_filename)
            .map(|entry| entry.input_sources.clone());
          let map_options = map_options.clone();
          let s =
            unsafe { token.used((self, compilation, output_path, f, source, asset_filename)) };
          s.spawn(
            |(plugin, compilation, output_path, f, source, asset_filename)| async move {
              let (source_map, input_sources) = if let Some(sources) = cached_sources {
                (None, sources)
              } else {
                let Some(map) = source.map(&map_options) else {
                  return Ok(None);
                };
                let sources = map
                  .sources()
                  .iter()
                  .map(ToString::to_string)
                  .collect::<Vec<_>>();
                (Some(map), sources.into())
              };
              let source_references = compute_source_references(compilation, &input_sources);
              let unresolved_source_map_path = plugin
                .get_unresolved_source_map_path(compilation, output_path, &asset_filename)
                .await?;

              let mut source_name_entries = Vec::with_capacity(source_references.len());
              for source_reference in &source_references {
                if let SourceReference::Source(sn) = source_reference
                  && is_schema_source(sn.as_ref())
                {
                  source_name_entries.push((
                    source_reference.clone(),
                    SourceNameWithBaseUrl::new(sn.to_string(), unresolved_source_map_path.clone()),
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
                  SourceNameWithBaseUrl::new(source_name, unresolved_source_map_path.clone()),
                ));
              }

              let task = SourceMapTask {
                asset_filename,
                source,
                source_map,
                input_sources,
                unresolved_source_map_path,
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
      .collect::<Vec<_>>(),
    };

    let mut tasks: Vec<SourceMapTask> = Vec::with_capacity(results.len());
    let mut reference_to_source_name_mapping = ReferenceToSourceNameMapping::default();

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
    reference_to_source_name_mapping: &mut ReferenceToSourceNameMapping,
  ) -> Result<()> {
    let output_options = &compilation.options.output;
    let mut used_names_set = HashSet::<SourceName>::with_capacity_and_hasher(
      reference_to_source_name_mapping.len(),
      Default::default(),
    );
    let default_fallback_module_filename_template =
      default_source_map_fallback_module_filename_template(
        compilation.options.experiments.runtime_mode,
      );
    let fallback_module_filename_template = self
      .fallback_module_filename_template
      .as_ref()
      .unwrap_or(default_fallback_module_filename_template);

    // Sort source references by identifier length so the shorter canonical resource wins first.
    // A CSS file can appear twice in the same source map: once as the extracted CSS module and
    // once as a longer loader-chain virtual module like
    // `cssExtractLoader.js!css-loader!...!style.css`. Both can resolve to the same default source
    // name, so the shorter entry should claim it first and force the longer virtual module to use
    // the fallback template instead.
    let sorted_iter =
      reference_to_source_name_mapping
        .iter_mut()
        .sorted_by(|(key_a, _), (key_b, _)| {
          let ident_a = match key_a {
            SourceReference::Module(identifier) => identifier.as_str(),
            SourceReference::Source(source) => source.as_ref(),
          };
          let ident_b = match key_b {
            SourceReference::Module(identifier) => identifier.as_str(),
            SourceReference::Source(source) => source.as_ref(),
          };
          ident_a.len().cmp(&ident_b.len())
        });

    for (source_reference, source_name_entry) in sorted_iter {
      if used_names_set.insert(source_name_entry.source_name.clone()) {
        continue;
      }

      let unresolved_source_map_path = &source_name_entry.unresolved_source_map_path;
      let new_source_name = match fallback_module_filename_template {
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
      let mut new_source_name = SourceName::new(
        new_source_name,
        unresolved_source_map_path.as_ref().map(|p| p.as_path()),
      );

      if used_names_set.insert(new_source_name.clone()) {
        source_name_entry.source_name = new_source_name;
        continue;
      }

      // Otherwise, append stars until we have a valid name
      while used_names_set.contains(&new_source_name) {
        new_source_name.push('*');
      }
      used_names_set.insert(new_source_name.clone());
      source_name_entry.source_name = new_source_name;
    }

    Ok(())
  }

  /// Update source_maps with deduplicated names and compute MappedAssets.
  async fn compute_mapped_assets(
    &self,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    reference_to_source_name_mapping: &ReferenceToSourceNameMapping,
    tasks: Vec<SourceMapTask>,
  ) -> Result<Vec<MappedAsset>> {
    let mapped_assets = rspack_parallel::scope::<_, Result<_>>(|token| {
      tasks.into_iter().for_each(
        |SourceMapTask {
           asset_filename,
           source,
           source_map,
           unresolved_source_map_path,
           source_references,
           input_sources: _,
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
              unresolved_source_map_path,
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
              unresolved_source_map_path,
              source_references,
            )| async move {
              let source_map = source_map
                .or_else(|| source.map(&MapOptions::new(plugin.columns)))
                .expect("cached map sources must still have a map");
              Self::create_mapped_asset(
                plugin,
                compilation,
                file_to_chunk,
                reference_to_source_name_mapping,
                asset_filename,
                source,
                source_map,
                unresolved_source_map_path,
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

  fn update_source_map(
    &self,
    source_map: &mut SourceMap,
    reference_to_source_name_mapping: &ReferenceToSourceNameMapping,
    asset_filename: &str,
    source_map_path: Option<&Utf8Path>,
    source_references: &[SourceReference],
    debug_id: Option<&str>,
  ) {
    // Update source_map with deduplicated source names
    source_map.set_file(Some(asset_filename.to_owned().into()));
    source_map.set_sources(source_references.iter().map(|source_reference| {
      reference_to_source_name_mapping
        .get(source_reference)
        .unwrap_or_else(|| {
          panic!(
            "SourceMapDevToolPlugin: missing source name for reference '{source_reference:?}' in asset '{asset_filename}'."
          )
        })
        .render_for_source_map(source_map_path)
        .into_owned()
    }));

    if let Some(asset_conditions) = &self.ignore_list {
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
      source_map.set_ignore_list(Some(Cow::Owned(ignore_list)));
    }

    if let Some(source_root) = &self.source_root {
      source_map.set_source_root(Some(source_root.to_string().into()));
    }

    if let Some(debug_id) = debug_id {
      source_map.set_debug_id(Some(debug_id.to_owned().into()));
    }

    if self.no_sources {
      source_map.set_sources_content(vec![]);
      if self.source_map_filename.is_some() {
        // External maps retain their metadata in MappingSource. Inline maps
        // are serialized and dropped immediately, so they need no compaction.
        source_map.shrink_to_fit();
      }
    }
  }

  /// Create a single MappedAsset: update the source map and emit asset + optional source map file.
  #[allow(clippy::too_many_arguments)]
  async fn create_mapped_asset(
    plugin: &SourceMapDevToolPlugin,
    compilation: &Compilation,
    file_to_chunk: &HashMap<&str, &Chunk>,
    reference_to_source_name_mapping: &ReferenceToSourceNameMapping,
    asset_filename: Arc<str>,
    source: BoxSource,
    mut source_map: SourceMap,
    unresolved_source_map_path: Option<Utf8PathBuf>,
    source_references: Vec<SourceReference>,
  ) -> Result<MappedAsset> {
    // Generate debug ID and serialize source map
    let debug_id = plugin
      .debug_ids
      .then(|| generate_debug_id(&asset_filename, &source.buffer()));
    let asset_filename_ref = asset_filename.as_ref();

    plugin.update_source_map(
      &mut source_map,
      reference_to_source_name_mapping,
      asset_filename_ref,
      unresolved_source_map_path.as_ref().map(|p| p.as_path()),
      &source_references,
      debug_id.as_deref(),
    );

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

    let source = if plugin.should_detach_source(&asset.info) {
      RawStringSource::from(source.source().into_string_lossy().into_owned()).boxed()
    } else {
      source
    };

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
      let dev_tool_map_source = DevToolMapSource::new(source_map);
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
        if chunk.is_some() && source_map_filename_config.has_content_hash_placeholder() {
          let mut hasher = RspackHasher::from(&compilation.options.output);
          update_content_hash(&dev_tool_map_source, &mut hasher);
          let digest = hasher.digest(&compilation.options.output.hash_digest);
          Some(digest)
        } else {
          None
        };

      let data = PathData::default().filename(&filename);
      let data = match chunk {
        Some(chunk) => data
          .chunk(chunk.ukey(), compilation)
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

      let asset_append = if let Some(current_source_mapping_url_comment) =
        current_source_mapping_url_comment
      {
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

        let mut suffix = debug_id
          .map(|id| format!("\n//# debugId={id}"))
          .unwrap_or_default();
        suffix.push_str(&current_source_mapping_url_comment);
        let suffix = RawStringSource::from(suffix).boxed();
        let asset_append = vec![suffix.clone()];
        asset.source = Some(DevToolCodeSource::new(source.clone(), suffix).boxed());
        asset.info.related.source_map = Some(source_map_filename.clone());
        asset_append
      } else {
        asset.source = Some(
          DevToolCodeSource::new(source.clone(), RawStringSource::from_static("").boxed()).boxed(),
        );
        Vec::new()
      };
      let mut source_map_asset_info = AssetInfo::default().with_development(Some(true));
      if let Some(asset) = compilation.assets().get(asset_filename.as_ref()) {
        // set source map asset version to be the same as the target asset
        source_map_asset_info.version = asset.info.version.clone();
      }
      let source_map_asset =
        CompilationAsset::new(Some(dev_tool_map_source.boxed()), source_map_asset_info);
      Ok(MappedAsset {
        asset: (asset_filename, asset),
        source_map: Some((source_map_filename.clone(), source_map_asset)),
        asset_append,
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
      let source_map_json = source_map.to_json();
      let base64 = base64::encode_to_string(source_map_json.as_bytes());
      let append = current_source_mapping_url_comment
        .cow_replace(
          "[url]",
          &format!("data:application/json;charset=utf-8;base64,{base64}"),
        )
        .into_owned();
      let suffix = RawStringSource::from(append).boxed();
      let asset_append = vec![suffix.clone()];
      asset.source = Some(DevToolCodeSource::new(source.clone(), suffix).boxed());
      Ok(MappedAsset {
        asset: (asset_filename, asset),
        asset_append,
        source_map: None,
      })
    }
  }
}

#[plugin_hook(CompilerCompilation for SourceMapDevToolPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  if !compilation.options.experiments.new_cache.devtool {
    compilation.use_source_map_dev_tool_plugin_cache = true;
  }
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for SourceMapDevToolPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_DEV_TOOLING)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger(PLUGIN_NAME);

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

  let new_cache = compilation
    .options
    .experiments
    .new_cache
    .devtool
    .then(|| compilation.get_cache(PLUGIN_NAME));
  let mut legacy_cache = if new_cache.is_some() {
    None
  } else {
    compilation.source_map_dev_tool_plugin_cache.take()
  };
  let cache_counter = legacy_cache
    .is_some()
    .then(|| logger.cache("source map persistent cache"));

  let start = logger.time("collect source maps");
  let condition_object = (self.test.is_some() || self.include.is_some() || self.exclude.is_some())
    .then(|| AssetConditionsObject {
      test: self.test.as_ref(),
      include: self.include.as_ref(),
      exclude: self.exclude.as_ref(),
    });
  let compilation_assets = compilation
    .assets()
    .iter()
    .filter(|(_filename, asset)| asset.info.related.source_map.is_none())
    .filter(|(filename, _asset)| {
      condition_object
        .as_ref()
        .is_none_or(|condition| match_object(condition, filename))
    })
    .map(|(filename, asset)| (filename.clone(), asset))
    .collect::<Vec<_>>();
  let use_cache = new_cache.is_some() || legacy_cache.is_some();
  let mut cached_entries = HashMap::default();
  if use_cache {
    for (filename, asset) in &compilation_assets {
      if asset.info.version.is_empty() || asset.get_source().is_none() {
        continue;
      }
      // Match the asset-version key used by the render cache and main's devtool cache.
      let cached = if let Some(cache) = &new_cache {
        cache
          .peek::<CachedSourceMapDevToolPluginEntry>(
            filename,
            Some(Etag::from(asset.info.version.clone())),
          )
          .map(|entry| Cow::Owned((*entry).clone()))
      } else {
        legacy_cache
          .as_ref()
          .and_then(|cache| cache.get(filename, &asset.info.version))
          .map(Cow::Borrowed)
      };
      if let Some(cached) = cached {
        cached_entries.insert(filename.clone(), cached);
      }
    }
  }
  let context_version = if use_cache {
    let mut hasher = RspackHasher::from(&compilation.options.output);
    output_path.hash(&mut hasher);
    compilation.options.context.hash(&mut hasher);
    compilation.get_hash().hash(&mut hasher);
    hasher
      .digest(&compilation.options.output.hash_digest)
      .encoded()
      .to_string()
  } else {
    String::new()
  };
  let legacy_version = legacy_cache
    .as_ref()
    .map(|cache| cache.compilation_version().to_string());
  let prepared = self
    .prepare_source_maps(
      compilation,
      &file_to_chunk,
      &output_path,
      &compilation_assets,
      &cached_entries,
      legacy_version.as_deref(),
      use_cache,
      context_version,
    )
    .await?;
  let mapped_assets = if prepared.reuse_all && new_cache.is_none() {
    let mut mapped_assets = Vec::with_capacity(cached_entries.len());
    for (filename, asset) in compilation_assets {
      let cached = cached_entries
        .get(&filename)
        .and_then(|entry| entry.restore(asset, self.should_detach_source(&asset.info)));
      if let Some((source_asset, source_map, asset_append)) = cached {
        if let Some(counter) = &cache_counter {
          counter.hit();
        }
        mapped_assets.push(MappedAsset {
          asset: (Arc::from(filename), source_asset),
          asset_append,
          source_map,
        });
      } else if let Some(counter) = &cache_counter {
        counter.miss();
      }
    }
    drop(cached_entries);
    if let Some(cache) = &mut legacy_cache {
      cache.reuse_unchanged();
    }
    mapped_assets
  } else {
    drop(cached_entries);
    if let Some(cache) = &new_cache {
      self
        .use_new_cache(
          compilation,
          &file_to_chunk,
          compilation_assets,
          prepared,
          cache,
        )
        .await?
    } else {
      self
        .use_legacy_cache(
          compilation,
          &file_to_chunk,
          compilation_assets,
          prepared,
          legacy_cache.as_mut(),
          cache_counter.as_ref(),
        )
        .await?
    }
  };
  logger.time_end(start);

  if let Some(cache) = legacy_cache {
    compilation.source_map_dev_tool_plugin_cache = Some(cache);
  }
  if let Some(counter) = cache_counter {
    logger.cache_end(counter);
  }

  let start = logger.time("emit source map assets");
  for mapped_asset in mapped_assets {
    let MappedAsset {
      asset: (source_filename, mut source_asset),
      source_map,
      ..
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
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
