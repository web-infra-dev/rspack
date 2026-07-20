mod drive;

use std::{
  hash::BuildHasherDefault,
  sync::{Arc, LazyLock},
};

use aho_corasick::{AhoCorasick, AhoCorasickKind, MatchKind};
use atomic_refcell::AtomicRefCell;
use derive_more::Debug;
pub use drive::*;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use rspack_core::{
  AssetInfo, Compilation, CompilationId, CompilationProcessAssets, ContentHashDependency, Logger,
  Plugin, RealContentHashArtifact,
  rspack_sources::{BoxSource, ReplaceSource, SourceExt, SourceValue},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHasher;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

type IndexSet<T> = indexmap::IndexSet<T, BuildHasherDefault<FxHasher>>;

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// We should make sure that there's no read-write and write-write conflicts for each hook instance by looking up [RealContentHashPlugin::get_compilation_hooks_mut]
type ArcReadContentHashPluginHooks = Arc<AtomicRefCell<RealContentHashPluginHooks>>;

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, ArcReadContentHashPluginHooks>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug, Default)]
pub struct RealContentHashPlugin;

impl RealContentHashPlugin {
  pub fn get_compilation_hooks(id: CompilationId) -> ArcReadContentHashPluginHooks {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
      .clone()
  }

  pub fn get_compilation_hooks_mut(id: CompilationId) -> ArcReadContentHashPluginHooks {
    COMPILATION_HOOKS_MAP.entry(id).or_default().clone()
  }
}

#[plugin_hook(CompilationProcessAssets for RealContentHashPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  inner_impl(compilation).await
}

impl Plugin for RealContentHashPlugin {
  fn name(&self) -> &'static str {
    "rspack.RealContentHashPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    COMPILATION_HOOKS_MAP.remove(&id);
  }
}

async fn inner_impl(compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.RealContentHashPlugin");
  let start = logger.time("hash to asset names");
  let mut hash_to_asset_names: HashMap<&str, Vec<&str>> = HashMap::default();
  for (name, asset) in compilation
    .assets()
    .iter()
    .filter(|(_, asset)| asset.get_source().is_some())
  {
    // e.g. filename: '[contenthash:8]-[contenthash:6].js'
    for hash in &asset.info.content_hash {
      hash_to_asset_names
        .entry(hash)
        .and_modify(|names| names.push(name))
        .or_insert_with(|| vec![name]);
    }
  }
  logger.time_end(start);
  if hash_to_asset_names.is_empty() {
    return Ok(());
  }
  let start = logger.time("create hash regexp");
  // use LeftmostLongest here:
  // e.g. 4afc|4afcbe match xxx.4afcbe-4afc.js -> xxx.[4afc]be-[4afc].js
  //      4afcbe|4afc match xxx.4afcbe-4afc.js -> xxx.[4afcbe]-[4afc].js
  const DFA_PATTERN_BYTES_CAP: usize = 128 * 1024;
  let total_pattern_bytes: usize = hash_to_asset_names.keys().map(|s| s.len()).sum();
  let hash_patterns = hash_to_asset_names
    .keys()
    .map(|hash| (*hash).to_string())
    .collect::<Vec<_>>();
  let hash_ac = AhoCorasick::builder()
    .match_kind(MatchKind::LeftmostLongest)
    .kind((total_pattern_bytes <= DFA_PATTERN_BYTES_CAP).then_some(AhoCorasickKind::DFA))
    .build(hash_patterns.iter().map(|s| s.as_bytes()))
    .expect("Invalid patterns");
  logger.time_end(start);

  let start = logger.time("create ordered hashes");
  let assets_data: HashMap<&str, AssetData> = compilation
    .assets()
    .par_iter()
    .filter_map(|(name, asset)| {
      asset.get_source().map(|source| {
        let recorded_dependencies = get_recorded_dependencies(
          name,
          asset.get_info(),
          &compilation.real_content_hash_artifact,
          &hash_to_asset_names,
        );
        (
          name.as_str(),
          AssetData::new(
            source.clone(),
            asset.get_info(),
            recorded_dependencies,
            &hash_ac,
            &hash_patterns,
          ),
        )
      })
    })
    .collect();

  let (ordered_hashes, mut hash_dependencies) =
    OrderedHashesBuilder::new(&hash_to_asset_names, &assets_data).build();
  let mut ordered_hashes_iter = ordered_hashes.into_iter();

  logger.time_end(start);

  let start = logger.time("old hash to new hash");
  let mut hash_to_new_hash = HashMap::default();

  let hooks = RealContentHashPlugin::get_compilation_hooks(compilation.id());

  let mut computed_hashes = HashSet::default();
  let mut top_task = ordered_hashes_iter.next();

  while let Some(top) = top_task {
    let mut batch = vec![top];
    top_task = None;

    for hash in ordered_hashes_iter.by_ref() {
      let Some(dependencies) = hash_dependencies.remove(hash.as_str()) else {
        top_task = Some(hash);
        break;
      };
      if dependencies.iter().all(|dep| computed_hashes.contains(dep)) {
        batch.push(hash);
      } else {
        top_task = Some(hash);
        break;
      }
    }

    let batch_source_tasks = batch
      .iter()
      .filter_map(|hash| {
        let assets_names = hash_to_asset_names.get(hash.as_str())?;
        let tasks = assets_names
          .iter()
          .filter_map(|name| {
            let data = assets_data.get(name)?;
            Some((hash.as_str(), *name, data))
          })
          .collect::<Vec<_>>();
        Some(tasks)
      })
      .flatten()
      .collect::<Vec<_>>();

    let batch_sources = batch_source_tasks
      .into_par_iter()
      .map(|(hash, name, data)| {
        let new_source = data.compute_new_source(
          data.own_hashes.contains(hash),
          &hash_to_new_hash,
          &hash_ac,
          &hash_patterns,
        );
        ((hash, name), new_source)
      })
      .collect::<HashMap<_, _>>();

    let new_hashes = rspack_parallel::scope::<_, Result<_>>(|token| {
      batch
        .iter()
        .cloned()
        .filter_map(|old_hash| {
          let asset_names = hash_to_asset_names.remove(old_hash.as_str())?;
          Some((old_hash, asset_names))
        })
        .for_each(|(old_hash, asset_names)| {
          let s =
            unsafe { token.used((&hooks, &compilation, &batch_sources, old_hash, asset_names)) };
          s.spawn(
            |(hooks, compilation, batch_sources, old_hash, mut asset_names)| async move {
              asset_names.sort_unstable();
              let mut asset_contents = asset_names
                .iter()
                .filter_map(|name| batch_sources.get(&(old_hash.as_str(), name)))
                .cloned()
                .collect::<Vec<_>>();
              asset_contents.dedup();
              let updated_hash = hooks
                .borrow()
                .update_hash
                .call(compilation, &asset_contents, &old_hash)
                .await?;

              let new_hash = if let Some(new_hash) = updated_hash {
                new_hash
              } else {
                let mut hasher = RspackHasher::from(&compilation.options.output);
                for asset_content in asset_contents {
                  hasher.write(&asset_content.buffer());
                }
                let new_hash = hasher.digest(&compilation.options.output.hash_digest);

                new_hash.rendered(old_hash.len()).to_string()
              };

              Ok((old_hash.clone(), new_hash))
            },
          );
        });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    for res in new_hashes {
      let (old_hash, new_hash) = res?;
      hash_to_new_hash.insert(old_hash, new_hash);
    }

    computed_hashes.extend(batch);
  }

  logger.time_end(start);

  let start = logger.time("collect hash updates");
  let updates: Vec<_> = assets_data
    .into_par_iter()
    .filter_map(|(name, data)| {
      let new_source = data.compute_new_source(false, &hash_to_new_hash, &hash_ac, &hash_patterns);
      let new_name = if data.has_recorded_dependencies {
        replace_asset_name(name, &data.own_hashes, &hash_to_new_hash)
      } else {
        let mut new_name = String::with_capacity(name.len());
        hash_ac.replace_all_with(name, &mut new_name, |_, hash, dst| {
          let replace_to = hash_to_new_hash
            .get(hash)
            .expect("RealContentHashPlugin: should have new hash");
          dst.push_str(replace_to);
          true
        });
        (name != new_name).then_some(new_name)
      };
      Some((name.to_owned(), new_source, new_name))
    })
    .collect();
  logger.time_end(start);

  let start = logger.time("update assets");
  let mut asset_renames = Vec::with_capacity(updates.len());
  for (name, new_source, new_name) in updates {
    let asset = compilation
      .assets_mut()
      .get_mut(&name)
      .expect("RealContentHashPlugin: asset should exist");
    asset.set_source(Some(new_source));
    let new_hashes = asset
      .get_info()
      .content_hash
      .iter()
      .map(|old_hash| {
        hash_to_new_hash
          .get(old_hash.as_str())
          .expect("should have new hash")
          .to_owned()
      })
      .collect();
    asset.get_info_mut().content_hash = new_hashes;
    if let Some(new_name) = new_name {
      asset_renames.push((name, new_name));
    }
  }

  compilation.par_rename_assets(asset_renames);

  logger.time_end(start);

  Ok(())
}

fn get_recorded_dependencies(
  name: &str,
  info: &AssetInfo,
  artifact: &RealContentHashArtifact,
  hash_to_asset_names: &HashMap<&str, Vec<&str>>,
) -> Option<RecordedContentHashDependencies> {
  let record = artifact.asset_record(name)?;
  if record.own_hashes() != &info.content_hash {
    return None;
  }

  let mut dependencies = HashSet::default();
  let mut may_contain_own_hash = info.related.source_map.is_some();
  for dependency in record.dependencies().iter() {
    match dependency {
      ContentHashDependency::Chunk(chunk, source_type) => {
        if let Some(hashes) = artifact.chunk_hashes(*chunk, *source_type) {
          for hash in hashes {
            if info.content_hash.contains(hash) {
              may_contain_own_hash = true;
            } else if hash_to_asset_names.contains_key(hash.as_str()) {
              dependencies.insert(hash.clone());
            }
          }
        }
      }
      ContentHashDependency::Hash(hash) => {
        if info.content_hash.contains(hash) {
          may_contain_own_hash = true;
        } else if hash_to_asset_names.contains_key(hash.as_str()) {
          dependencies.insert(hash.clone());
        }
      }
    }
  }
  Some(RecordedContentHashDependencies {
    referenced_hashes: dependencies,
    may_contain_own_hash,
  })
}

struct RecordedContentHashDependencies {
  referenced_hashes: HashSet<String>,
  may_contain_own_hash: bool,
}

#[derive(Debug, Clone, Copy)]
struct ContentHashMatch {
  start: u32,
  end: u32,
  pattern: usize,
}

fn replace_asset_name(
  name: &str,
  own_hashes: &HashSet<String>,
  hash_to_new_hash: &HashMap<String, String>,
) -> Option<String> {
  let mut matches = own_hashes
    .iter()
    .flat_map(|hash| {
      name
        .match_indices(hash)
        .map(move |(start, matched)| (start, start + matched.len(), hash))
    })
    .collect::<Vec<_>>();
  if matches.is_empty() {
    return None;
  }
  matches.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

  let mut output = String::with_capacity(name.len());
  let mut last = 0;
  for (start, end, hash) in matches {
    if start < last {
      continue;
    }
    output.push_str(&name[last..start]);
    output.push_str(
      hash_to_new_hash
        .get(hash)
        .expect("RealContentHashPlugin: should have new hash"),
    );
    last = end;
  }
  output.push_str(&name[last..]);
  (name != output).then_some(output)
}

fn find_content_hash_matches(source: &BoxSource, hash_ac: &AhoCorasick) -> Vec<ContentHashMatch> {
  let SourceValue::String(content) = source.source() else {
    return Vec::new();
  };
  hash_ac
    .find_iter(content.as_ref())
    .map(|matched| ContentHashMatch {
      start: matched
        .start()
        .try_into()
        .expect("source size should fit in u32"),
      end: matched
        .end()
        .try_into()
        .expect("source size should fit in u32"),
      pattern: matched.pattern().as_usize(),
    })
    .collect()
}

#[derive(Debug)]
struct AssetData {
  own_hashes: HashSet<String>,
  referenced_hashes: HashSet<String>,
  may_contain_own_hash: bool,
  has_recorded_dependencies: bool,
  #[debug(skip)]
  old_source: BoxSource,
  #[debug(skip)]
  content_hash_matches: OnceCell<Vec<ContentHashMatch>>,
  #[debug(skip)]
  new_source: OnceCell<BoxSource>,
  #[debug(skip)]
  new_source_without_own: OnceCell<BoxSource>,
}

impl AssetData {
  pub fn new(
    source: BoxSource,
    info: &AssetInfo,
    recorded_dependencies: Option<RecordedContentHashDependencies>,
    hash_ac: &AhoCorasick,
    hash_patterns: &[String],
  ) -> Self {
    let own_hashes = info.content_hash.iter().cloned().collect::<HashSet<_>>();
    let content_hash_matches = OnceCell::new();
    let has_recorded_dependencies = recorded_dependencies.is_some();
    let (referenced_hashes, may_contain_own_hash) =
      if let Some(recorded_dependencies) = recorded_dependencies {
        (
          recorded_dependencies.referenced_hashes,
          recorded_dependencies.may_contain_own_hash,
        )
      } else {
        let matches = find_content_hash_matches(&source, hash_ac);
        let may_contain_own_hash = matches
          .iter()
          .any(|matched| own_hashes.contains(&hash_patterns[matched.pattern]));
        let referenced_hashes = matches
          .iter()
          .map(|matched| hash_patterns[matched.pattern].clone())
          .filter(|hash| !own_hashes.contains(hash))
          .collect();
        content_hash_matches
          .set(matches)
          .expect("content hash matches should only be initialized once");
        (referenced_hashes, may_contain_own_hash)
      };

    Self {
      own_hashes,
      referenced_hashes,
      may_contain_own_hash,
      has_recorded_dependencies,
      old_source: source,
      content_hash_matches,
      new_source: OnceCell::new(),
      new_source_without_own: OnceCell::new(),
    }
  }

  pub fn compute_new_source(
    &self,
    without_own: bool,
    hash_to_new_hash: &HashMap<String, String>,
    hash_ac: &AhoCorasick,
    hash_patterns: &[String],
  ) -> BoxSource {
    (if without_own {
      &self.new_source_without_own
    } else {
      &self.new_source
    })
    .get_or_init(|| {
      let own_hash_may_change = self.may_contain_own_hash
        && (without_own
          || self
            .own_hashes
            .iter()
            .any(|hash| matches!(hash_to_new_hash.get(hash), Some(new_hash) if new_hash != hash)));
      let referenced_hash_may_change = self
        .referenced_hashes
        .iter()
        .any(|hash| matches!(hash_to_new_hash.get(hash), Some(new_hash) if new_hash != hash));
      if !own_hash_may_change && !referenced_hash_may_change {
        return self.old_source.clone();
      }

      let matches = self
        .content_hash_matches
        .get_or_init(|| find_content_hash_matches(&self.old_source, hash_ac));
      let mut replace_source = ReplaceSource::new(self.old_source.clone());
      let mut changed = false;

      for matched in matches {
        let old_hash = &hash_patterns[matched.pattern];
        let replacement = if without_own && self.own_hashes.contains(old_hash) {
          Some("")
        } else {
          hash_to_new_hash
            .get(old_hash)
            .filter(|new_hash| *new_hash != old_hash)
            .map(String::as_str)
        };
        if let Some(replacement) = replacement {
          replace_source.replace(matched.start, matched.end, replacement.to_string(), None);
          changed = true;
        }
      }

      if changed {
        replace_source.boxed()
      } else {
        self.old_source.clone()
      }
    })
    .clone()
  }
}

struct OrderedHashesBuilder<'a> {
  hash_to_asset_names: &'a HashMap<&'a str, Vec<&'a str>>,
  assets_data: &'a HashMap<&'a str, AssetData>,
}

impl<'a> OrderedHashesBuilder<'a> {
  pub fn new(
    hash_to_asset_names: &'a HashMap<&'a str, Vec<&'a str>>,
    assets_data: &'a HashMap<&'a str, AssetData>,
  ) -> Self {
    Self {
      hash_to_asset_names,
      assets_data,
    }
  }

  pub fn build(&self) -> (IndexSet<String>, HashMap<String, HashSet<String>>) {
    let mut ordered_hashes = IndexSet::default();
    let mut hash_dependencies = HashMap::default();
    for hash in self.hash_to_asset_names.keys() {
      self.add_to_ordered_hashes(
        hash,
        &mut ordered_hashes,
        &mut HashSet::default(),
        &mut hash_dependencies,
      );
    }
    (
      ordered_hashes,
      hash_dependencies
        .into_iter()
        .map(|(k, v)| {
          (
            k.to_string(),
            v.into_iter().map(|s| s.to_string()).collect(),
          )
        })
        .collect(),
    )
  }
}

impl OrderedHashesBuilder<'_> {
  fn get_hash_dependencies(&self, hash: &str) -> HashSet<&str> {
    let asset_names = self
      .hash_to_asset_names
      .get(hash)
      .expect("RealContentHashPlugin: should have asset_names");
    let mut hashes = HashSet::default();
    for name in asset_names {
      if let Some(asset_hash) = self.assets_data.get(name) {
        if !asset_hash.own_hashes.contains(hash) {
          for hash in &asset_hash.own_hashes {
            hashes.insert(hash.as_str());
          }
        }
        for hash in &asset_hash.referenced_hashes {
          hashes.insert(hash.as_str());
        }
      }
    }
    hashes
  }

  fn add_to_ordered_hashes<'b, 'a: 'b>(
    &'a self,
    hash: &'b str,
    ordered_hashes: &mut IndexSet<String>,
    stack: &mut HashSet<&'b str>,
    hash_dependencies: &mut HashMap<&'b str, HashSet<&'b str>>,
  ) {
    let deps = hash_dependencies
      .entry(hash)
      .or_insert_with(|| self.get_hash_dependencies(hash))
      .clone();
    stack.insert(hash);
    for dep in deps {
      if ordered_hashes.contains(dep) {
        continue;
      }
      if stack.contains(dep) {
        // Safety: all chunk-level hash will be collected in runtime chunk
        // so there shouldn't have circular hash dependency between chunks
        panic!("RealContentHashPlugin: circular hash dependency");
      }
      self.add_to_ordered_hashes(dep, ordered_hashes, stack, hash_dependencies);
    }
    ordered_hashes.insert(hash.to_string());
    stack.remove(hash);
  }
}
