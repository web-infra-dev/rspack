#![feature(let_chains)]

mod drive;

use std::{
  hash::{BuildHasherDefault, Hasher},
  sync::LazyLock,
};

use aho_corasick::{AhoCorasick, MatchKind};
use derive_more::Debug;
pub use drive::*;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  AssetInfo, BindingCell, Compilation, CompilationId, CompilationProcessAssets, Logger, Plugin,
  PluginContext,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

type IndexSet<T> = indexmap::IndexSet<T, BuildHasherDefault<FxHasher>>;

pub static QUOTE_META: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Invalid regex"));

static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, Box<RealContentHashPluginHooks>>> =
  LazyLock::new(Default::default);

#[plugin]
#[derive(Debug, Default)]
pub struct RealContentHashPlugin;

impl RealContentHashPlugin {
  pub fn get_compilation_hooks(
    id: CompilationId,
  ) -> dashmap::mapref::one::Ref<'static, CompilationId, Box<RealContentHashPluginHooks>> {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
  }

  pub fn get_compilation_hooks_mut(
    id: CompilationId,
  ) -> dashmap::mapref::one::RefMut<'static, CompilationId, Box<RealContentHashPluginHooks>> {
    COMPILATION_HOOKS_MAP.entry(id).or_default()
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
        .or_insert(vec![name]);
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
  let hash_ac = AhoCorasick::builder()
    .match_kind(MatchKind::LeftmostLongest)
    .build(hash_to_asset_names.keys().map(|s| s.as_bytes()))
    .expect("Invalid patterns");
  logger.time_end(start);

  let start = logger.time("create ordered hashes");
  let assets_data: HashMap<&str, AssetData> = compilation
    .assets()
    .par_iter()
    .filter_map(|(name, asset)| {
      asset.get_source().map(|source| {
        (
          name.as_str(),
          AssetData::new(source.clone(), asset.get_info(), &hash_ac),
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

  loop {
    let Some(top) = top_task else {
      break;
    };
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
        let new_source =
          data.compute_new_source(data.own_hashes.contains(hash), &hash_to_new_hash, &hash_ac);
        ((hash, name), new_source)
      })
      .collect::<HashMap<_, _>>();

    let new_hashes = rspack_futures::scope::<_, Result<_>>(|token| {
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
              asset_names.sort();
              let mut asset_contents = asset_names
                .iter()
                .filter_map(|name| batch_sources.get(&(old_hash.as_str(), name)))
                .cloned()
                .collect::<Vec<_>>();
              asset_contents.dedup();
              let updated_hash = hooks
                .update_hash
                .call(compilation, &asset_contents, &old_hash)
                .await?;

              let new_hash = if let Some(new_hash) = updated_hash {
                new_hash
              } else {
                let mut hasher = RspackHash::from(&compilation.options.output);
                for asset_content in asset_contents {
                  hasher.write(&asset_content.buffer());
                }
                let new_hash = hasher.digest(&compilation.options.output.hash_digest);
                let new_hash = new_hash.rendered(old_hash.len()).to_string();
                new_hash
              };

              Ok((old_hash.to_string(), new_hash))
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
      let new_source = data.compute_new_source(false, &hash_to_new_hash, &hash_ac);
      let mut new_name = String::with_capacity(name.len());
      hash_ac.replace_all_with(name, &mut new_name, |_, hash, dst| {
        let replace_to = hash_to_new_hash
          .get(hash)
          .expect("RealContentHashPlugin: should have new hash");
        dst.push_str(replace_to);
        true
      });
      let new_name = (name != new_name).then_some(new_name);
      Some((name.to_owned(), new_source, new_name))
    })
    .collect();
  logger.time_end(start);

  let start = logger.time("update assets");
  let mut rename_tasks = vec![];
  for (name, new_source, new_name) in updates {
    compilation.update_asset(&name, |_, old_info| {
      let new_hashes: HashSet<_> = old_info
        .content_hash
        .iter()
        .map(|old_hash| {
          hash_to_new_hash
            .get(old_hash.as_str())
            .expect("should have new hash")
            .to_owned()
        })
        .collect();
      let info_update = (*old_info).clone();
      Ok((
        new_source.clone(),
        BindingCell::from(info_update.with_content_hashes(new_hashes)),
      ))
    })?;
    if let Some(new_name) = new_name {
      rename_tasks.push((name, new_name));
    }
  }

  let assets = compilation.assets_mut();
  rename_tasks.retain(|(filename, new_name)| {
    if let Some(asset) = assets.remove(filename) {
      assets.insert(new_name.clone(), asset);
      true
    } else {
      false
    }
  });

  compilation
    .chunk_by_ukey
    .values_mut()
    .par_bridge()
    .for_each(|chunk| {
      for (filename, new_name) in rename_tasks.iter() {
        if chunk.remove_file(filename) {
          chunk.add_file(new_name.clone());
        }

        if chunk.remove_auxiliary_file(filename) {
          chunk.add_auxiliary_file(new_name.clone());
        }
      }
    });

  logger.time_end(start);

  Ok(())
}

#[derive(Debug)]
struct AssetData {
  own_hashes: HashSet<String>,
  referenced_hashes: HashSet<String>,
  #[debug(skip)]
  old_source: BoxSource,
  #[debug(skip)]
  content: AssetDataContent,
  #[debug(skip)]
  new_source: OnceCell<BoxSource>,
  #[debug(skip)]
  new_source_without_own: OnceCell<BoxSource>,
}

#[derive(Debug)]
enum AssetDataContent {
  Buffer,
  String(String),
}

impl AssetData {
  pub fn new(source: BoxSource, info: &AssetInfo, hash_ac: &AhoCorasick) -> Self {
    let mut own_hashes = HashSet::default();
    let mut referenced_hashes = HashSet::default();
    // TODO(ahabhgk): source.is_buffer() instead of String::from_utf8().is_ok()
    let content = if let Ok(content) = String::from_utf8(source.buffer().to_vec()) {
      for hash in hash_ac.find_iter(&content) {
        let hash = &content[hash.range()];
        if info.content_hash.contains(hash) {
          own_hashes.insert(hash.to_string());
          continue;
        }
        referenced_hashes.insert(hash.to_string());
      }
      AssetDataContent::String(content)
    } else {
      AssetDataContent::Buffer
    };

    Self {
      own_hashes,
      referenced_hashes,
      old_source: source,
      content,
      new_source: OnceCell::new(),
      new_source_without_own: OnceCell::new(),
    }
  }

  pub fn compute_new_source(
    &self,
    without_own: bool,
    hash_to_new_hash: &HashMap<String, String>,
    hash_ac: &AhoCorasick,
  ) -> BoxSource {
    (if without_own {
      &self.new_source_without_own
    } else {
      &self.new_source
    })
    .get_or_init(|| {
      if let AssetDataContent::String(content) = &self.content
        && (!self.own_hashes.is_empty()
          || self
            .referenced_hashes
            .iter()
            .any(|hash| matches!(hash_to_new_hash.get(hash.as_str()), Some(h) if h != hash)))
      {
        let mut new_content = String::with_capacity(content.len());
        hash_ac.replace_all_with(content, &mut new_content, |_, hash, dst| {
          let replace_to = if without_own && self.own_hashes.contains(hash) {
            ""
          } else {
            hash_to_new_hash
              .get(hash)
              .expect("RealContentHashPlugin: should have new hash")
          };
          dst.push_str(replace_to);
          true
        });
        return RawStringSource::from(new_content).boxed();
      }
      self.old_source.clone()
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
