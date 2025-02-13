use std::borrow::Cow;
use std::fmt::Debug;

use rspack_paths::Utf8PathBuf;
use rspack_sources::BoxSource;
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use crate::{ChunkGroupOrderKey, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType};

pub enum EntrypointsStatsOption {
  Bool(bool),
  String(String),
}

pub struct ExtendedStatsOptions {
  pub assets: bool,
  pub cached_modules: bool,
  pub chunks: bool,
  pub chunk_group_auxiliary: bool,
  pub chunk_group_children: bool,
  pub chunk_groups: bool,
  pub chunk_modules: bool,
  pub chunk_relations: bool,
  pub depth: bool,
  pub entrypoints: EntrypointsStatsOption,
  pub errors: bool,
  pub hash: bool,
  pub ids: bool,
  pub modules: bool,
  pub module_assets: bool,
  pub nested_modules: bool,
  pub optimization_bailout: bool,
  pub provided_exports: bool,
  pub reasons: bool,
  pub source: bool,
  pub used_exports: bool,
  pub warnings: bool,
}

#[derive(Debug)]
pub struct StatsError<'s> {
  pub message: String,
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<Cow<'s, str>>,
  pub module_id: Option<&'s str>,
  pub loc: Option<String>,
  pub file: Option<Utf8PathBuf>,

  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<StatsModuleTrace>,
}

#[derive(Debug)]
pub struct StatsWarning<'s> {
  pub message: String,
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<Cow<'s, str>>,
  pub module_id: Option<&'s str>,
  pub loc: Option<String>,
  pub file: Option<Utf8PathBuf>,

  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<StatsModuleTrace>,
}

#[derive(Debug)]
pub struct StatsModuleTrace {
  pub origin: StatsErrorModuleTraceModule,
  pub module: StatsErrorModuleTraceModule,
  pub dependencies: Vec<StatsErrorModuleTraceDependency>,
}

#[derive(Debug)]
pub struct StatsErrorModuleTraceModule {
  pub identifier: ModuleIdentifier,
  pub name: String,
  pub id: Option<String>,
}

#[derive(Debug)]
pub struct StatsErrorModuleTraceDependency {
  pub loc: String,
}

#[derive(Debug)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<Option<String>>,
  pub chunk_names: Vec<String>,
  pub chunk_id_hints: Vec<String>,
  pub info: StatsAssetInfo,
  pub emitted: bool,
  pub auxiliary_chunk_names: Vec<String>,
  pub auxiliary_chunk_id_hints: Vec<String>,
  pub auxiliary_chunks: Vec<Option<String>>,
}

#[derive(Debug)]
pub struct StatsAssetsByChunkName {
  pub name: String,
  pub files: Vec<String>,
}

#[derive(Debug)]
pub struct StatsAssetInfo {
  pub minimized: Option<bool>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub source_filename: Option<String>,
  pub copied: Option<bool>,
  pub immutable: Option<bool>,
  pub javascript_module: Option<bool>,
  pub chunk_hash: Vec<String>,
  pub content_hash: Vec<String>,
  pub full_hash: Vec<String>,
  pub related: Vec<StatsAssetInfoRelated>,
  pub is_over_size_limit: Option<bool>,
}

#[derive(Debug)]
pub struct StatsAssetInfoRelated {
  pub name: String,
  pub value: Vec<String>,
}

#[derive(Debug)]
pub struct StatsModule<'s> {
  pub r#type: &'static str,
  pub module_type: ModuleType,
  pub layer: Option<Cow<'s, str>>,
  pub identifier: Option<ModuleIdentifier>,
  pub name: Option<Cow<'s, str>>,
  pub name_for_condition: Option<String>,
  pub id: Option<&'s str>,
  pub chunks: Option<Vec<String>>, // has id after the call of chunkIds hook
  pub size: f64,
  pub sizes: Vec<StatsSourceTypeSize>,
  pub dependent: Option<bool>,
  pub issuer: Option<ModuleIdentifier>,
  pub issuer_name: Option<Cow<'s, str>>,
  pub issuer_id: Option<&'s str>,
  pub issuer_path: Option<Vec<StatsModuleIssuer<'s>>>,
  pub reasons: Option<Vec<StatsModuleReason<'s>>>,
  pub assets: Option<Vec<String>>,
  pub modules: Option<Vec<StatsModule<'s>>>,
  pub source: Option<&'s BoxSource>,
  pub profile: Option<StatsModuleProfile>,
  pub orphan: Option<bool>,
  pub provided_exports: Option<Vec<Atom>>,
  pub used_exports: Option<StatsUsedExports>,
  pub optimization_bailout: Option<&'s [String]>,
  pub depth: Option<usize>,
  pub pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub built: bool,
  pub code_generated: bool,
  pub build_time_executed: bool,
  pub cached: bool,
  pub cacheable: Option<bool>,
  pub optional: Option<bool>,
  pub failed: Option<bool>,
  pub errors: Option<u32>,
  pub warnings: Option<u32>,
}

#[derive(Debug)]
pub enum StatsUsedExports {
  Vec(Vec<Atom>),
  Bool(bool),
  Null,
}

#[derive(Debug)]
pub struct StatsModuleProfile {
  pub factory: StatsMillisecond,
  pub building: StatsMillisecond,
}

#[derive(Debug)]
pub struct StatsOriginRecord {
  pub module: Option<ModuleIdentifier>,
  pub module_id: String,
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: String,
  pub loc: String,
  pub request: String,
}

#[derive(Debug)]
pub struct StatsChunk<'a> {
  pub r#type: &'static str,
  pub files: Vec<String>,
  pub auxiliary_files: Vec<String>,
  pub id: Option<String>,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
  pub modules: Option<Vec<StatsModule<'a>>>,
  pub parents: Option<Vec<String>>,
  pub children: Option<Vec<String>>,
  pub siblings: Option<Vec<String>>,
  pub children_by_order: HashMap<ChunkGroupOrderKey, Vec<String>>,
  pub runtime: RuntimeSpec,
  pub sizes: HashMap<SourceType, f64>,
  pub reason: Option<String>,
  pub rendered: bool,
  pub origins: Vec<StatsOriginRecord>,
  pub id_hints: Vec<String>,
  pub hash: Option<String>,
}

#[derive(Debug)]
pub struct StatsChunkGroupAsset {
  pub name: String,
  pub size: usize,
}

#[derive(Debug)]
pub struct StatsChunkGroup {
  pub name: String,
  pub chunks: Vec<String>,
  pub assets: Vec<StatsChunkGroupAsset>,
  pub assets_size: usize,
  pub auxiliary_assets: Option<Vec<StatsChunkGroupAsset>>,
  pub auxiliary_assets_size: Option<usize>,
  pub children: Option<StatsChunkGroupChildren>,
  pub is_over_size_limit: Option<bool>,
  pub child_assets: Option<StatschunkGroupChildAssets>,
}

#[derive(Debug)]
pub struct StatsChunkGroupChildren {
  pub preload: Vec<StatsChunkGroup>,
  pub prefetch: Vec<StatsChunkGroup>,
}

#[derive(Debug)]
pub struct StatschunkGroupChildAssets {
  pub preload: Vec<String>,
  pub prefetch: Vec<String>,
}

#[derive(Debug)]
pub struct StatsModuleIssuer<'s> {
  pub identifier: ModuleIdentifier,
  pub name: Cow<'s, str>,
  pub id: Option<&'s str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatsModuleReason<'s> {
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<Cow<'s, str>>,
  pub module_id: Option<&'s str>,
  pub module_chunks: Option<u32>,
  pub resolved_module_identifier: Option<ModuleIdentifier>,
  pub resolved_module_name: Option<Cow<'s, str>>,
  pub resolved_module_id: Option<&'s str>,

  pub r#type: Option<&'static str>,
  pub user_request: Option<&'s str>,
  pub explanation: Option<&'static str>,
  pub active: bool,
  pub loc: Option<String>,
}

#[derive(Debug)]
pub struct StatsMillisecond {
  pub secs: u64,
  pub subsec_millis: u32,
}

impl StatsMillisecond {
  pub fn new(secs: u64, subsec_millis: u32) -> Self {
    Self {
      secs,
      subsec_millis,
    }
  }
}

#[derive(Debug)]
pub struct StatsSourceTypeSize {
  pub source_type: SourceType,
  pub size: f64,
}
