#![feature(let_chains)]

use std::{
  borrow::Cow,
  hash::{BuildHasherDefault, Hash},
};

use derivative::Derivative;
use once_cell::sync::{Lazy, OnceCell};
use rayon::prelude::*;
use regex::{Captures, Regex};
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  AssetInfo, Compilation, Logger, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook, AsyncSeries};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

type IndexSet<T> = indexmap::IndexSet<T, BuildHasherDefault<FxHasher>>;

pub static QUOTE_META: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[-\[\]\\/{}()*+?.^$|]").expect("Invalid regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct RealContentHashPlugin;

#[plugin_hook(AsyncSeries<Compilation> for RealContentHashPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  inner_impl(compilation)
}

impl Plugin for RealContentHashPlugin {
  fn name(&self) -> &'static str {
    "rspack.RealContentHashPlugin"
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

fn inner_impl(compilation: &mut Compilation) -> Result<()> {
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
  let mut hash_list = hash_to_asset_names
    .keys()
    // xx\xx{xx?xx.xx -> xx\\xx\{xx\?xx\.xx escape for Regex::new
    .map(|hash| QUOTE_META.replace_all(hash, "\\$0"))
    .collect::<Vec<Cow<str>>>();
  // long hash should sort before short hash to make sure match long hash first in hash_regexp matching
  // e.g. 4afc|4afcbe match xxx.4afcbe-4afc.js -> xxx.[4afc]be-[4afc].js
  //      4afcbe|4afc match xxx.4afcbe-4afc.js -> xxx.[4afcbe]-[4afc].js
  hash_list.par_sort_by(|a, b| b.len().cmp(&a.len()));
  let hash_regexp = Regex::new(&hash_list.join("|")).expect("Invalid regex");
  logger.time_end(start);

  let start = logger.time("create ordered hashes");
  let assets_data: HashMap<&str, AssetData> = compilation
    .assets()
    .par_iter()
    .filter_map(|(name, asset)| {
      asset.get_source().map(|source| {
        (
          name.as_str(),
          AssetData::new(source.clone(), asset.get_info(), &hash_regexp),
        )
      })
    })
    .collect();

  let ordered_hashes = OrderedHashesBuilder::new(&hash_to_asset_names, &assets_data).build();
  logger.time_end(start);

  let start = logger.time("old hash to new hash");
  let mut hash_to_new_hash = HashMap::default();

  for old_hash in &ordered_hashes {
    if let Some(asset_names) = hash_to_asset_names.get_mut(old_hash.as_str()) {
      asset_names.sort();
      let asset_contents: Vec<_> = asset_names
        .par_iter()
        .filter_map(|name| assets_data.get(name))
        .map(|data| {
          data.compute_new_source(
            data.own_hashes.contains(old_hash),
            &hash_to_new_hash,
            &hash_regexp,
          )
        })
        .collect();
      let mut hasher = RspackHash::from(&compilation.options.output);
      for asset_content in asset_contents {
        asset_content.hash(&mut hasher);
      }
      let new_hash = hasher.digest(&compilation.options.output.hash_digest);
      let new_hash = new_hash.rendered(old_hash.len()).to_string();
      hash_to_new_hash.insert(old_hash, new_hash);
    }
  }
  logger.time_end(start);

  let start = logger.time("collect hash updates");
  let updates: Vec<_> = assets_data
    .into_par_iter()
    .filter_map(|(name, data)| {
      let new_source = data.compute_new_source(false, &hash_to_new_hash, &hash_regexp);
      let new_name = hash_regexp
        .replace_all(name, |c: &Captures| {
          let hash = c
            .get(0)
            .expect("RealContentHashPlugin: should have match")
            .as_str();
          hash_to_new_hash
            .get(hash)
            .expect("RealContentHashPlugin: should have new hash")
        })
        .into_owned();
      let new_name = (name != new_name).then_some(new_name);
      Some((name.to_owned(), new_source.clone(), new_name))
    })
    .collect();
  logger.time_end(start);

  let start = logger.time("update assets");
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
      Ok((new_source.clone(), old_info.with_content_hashes(new_hashes)))
    })?;
    if let Some(new_name) = new_name {
      compilation.rename_asset(&name, new_name);
    }
  }
  logger.time_end(start);

  Ok(())
}

#[derive(Derivative)]
#[derivative(Debug)]
struct AssetData {
  own_hashes: HashSet<String>,
  referenced_hashes: HashSet<String>,
  #[derivative(Debug = "ignore")]
  old_source: BoxSource,
  #[derivative(Debug = "ignore")]
  content: AssetDataContent,
  #[derivative(Debug = "ignore")]
  new_source: OnceCell<BoxSource>,
  #[derivative(Debug = "ignore")]
  new_source_without_own: OnceCell<BoxSource>,
}

#[derive(Debug)]
enum AssetDataContent {
  Buffer,
  String(String),
}

impl AssetData {
  pub fn new(source: BoxSource, info: &AssetInfo, hash_regexp: &Regex) -> Self {
    let mut own_hashes = HashSet::default();
    let mut referenced_hashes = HashSet::default();
    // TODO(ahabhgk): source.is_buffer() instead of String::from_utf8().is_ok()
    let content = if let Ok(content) = String::from_utf8(source.buffer().to_vec()) {
      for hash in hash_regexp.find_iter(&content) {
        if info.content_hash.contains(hash.as_str()) {
          own_hashes.insert(hash.as_str().to_string());
          continue;
        }
        referenced_hashes.insert(hash.as_str().to_string());
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
    hash_to_new_hash: &HashMap<&str, String>,
    hash_regexp: &Regex,
  ) -> &BoxSource {
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
        let new_content = hash_regexp.replace_all(content, |c: &Captures| {
          let hash = c
            .get(0)
            .expect("RealContentHashPlugin: should have matched")
            .as_str();
          if without_own && self.own_hashes.contains(hash) {
            return "";
          }
          hash_to_new_hash
            .get(hash)
            .expect("RealContentHashPlugin: should have new hash")
        });
        return RawSource::from(new_content.into_owned()).boxed();
      }
      self.old_source.clone()
    })
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

  pub fn build(&self) -> IndexSet<String> {
    let mut ordered_hashes = IndexSet::default();
    for hash in self.hash_to_asset_names.keys() {
      self.add_to_ordered_hashes(hash, &mut ordered_hashes, &mut HashSet::default());
    }
    ordered_hashes
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
  ) {
    let deps = self.get_hash_dependencies(hash);
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
      self.add_to_ordered_hashes(dep, ordered_hashes, stack);
    }
    ordered_hashes.insert(hash.to_string());
    stack.remove(hash);
  }
}
