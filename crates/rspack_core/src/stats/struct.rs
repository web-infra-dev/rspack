use std::{borrow::Cow, fmt::Debug};

use rspack_paths::Utf8Path;
use rspack_sources::BoxSource;
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;

use crate::{ChunkGroupOrderKey, ModuleId, ModuleIdentifier, ModuleType, RuntimeSpec, SourceType};

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

impl Default for ExtendedStatsOptions {
  fn default() -> Self {
    Self {
      chunks: true,
      chunk_modules: true,
      errors: true,
      warnings: true,
      assets: true,
      hash: true,

      cached_modules: false,
      chunk_group_auxiliary: false,
      chunk_group_children: false,
      chunk_groups: false,
      chunk_relations: false,
      depth: false,
      entrypoints: EntrypointsStatsOption::Bool(false),
      ids: false,
      modules: false,
      module_assets: false,
      nested_modules: false,
      optimization_bailout: false,
      provided_exports: false,
      reasons: false,
      source: false,
      used_exports: false,
    }
  }
}

#[derive(Debug)]
pub struct StatsError<'a> {
  pub name: Option<String>,
  pub message: String,
  pub code: Option<String>,
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<Cow<'a, str>>,
  pub module_id: Option<ModuleId>,
  pub loc: Option<String>,
  pub file: Option<&'a Utf8Path>,

  pub chunk_name: Option<&'a str>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub chunk_id: Option<&'a str>,
  pub details: Option<String>,
  pub stack: Option<String>,
  pub module_trace: Vec<StatsModuleTrace<'a>>,
}

#[derive(Debug)]
pub struct StatsModuleTrace<'a> {
  pub origin: StatsErrorModuleTraceModule<'a>,
  pub module: StatsErrorModuleTraceModule<'a>,
  pub dependencies: Vec<StatsErrorModuleTraceDependency>,
}

#[derive(Debug)]
pub struct StatsErrorModuleTraceModule<'a> {
  pub identifier: ModuleIdentifier,
  pub name: Cow<'a, str>,
  pub id: Option<ModuleId>,
}

#[derive(Debug)]
pub struct StatsErrorModuleTraceDependency {
  pub loc: String,
}

#[derive(Debug)]
pub struct StatsAsset<'a> {
  pub r#type: &'static str,
  pub name: &'a str,
  pub size: f64,
  pub chunks: Vec<Option<&'a str>>,
  pub chunk_names: Vec<&'a str>,
  pub chunk_id_hints: Vec<&'a str>,
  pub info: StatsAssetInfo<'a>,
  pub emitted: bool,
  pub auxiliary_chunk_names: Vec<&'a str>,
  pub auxiliary_chunk_id_hints: Vec<&'a str>,
  pub auxiliary_chunks: Vec<Option<&'a str>>,
}

#[derive(Debug)]
pub struct StatsAssetsByChunkName<'a> {
  pub name: &'a str,
  pub files: Vec<&'a str>,
}

#[derive(Debug)]
pub struct StatsAssetInfo<'a> {
  pub minimized: Option<bool>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub source_filename: Option<&'a str>,
  pub copied: Option<bool>,
  pub immutable: Option<bool>,
  pub javascript_module: Option<bool>,
  pub chunk_hash: Vec<&'a str>,
  pub content_hash: Vec<&'a str>,
  pub full_hash: Vec<&'a str>,
  pub related: Vec<StatsAssetInfoRelated<'a>>,
  pub is_over_size_limit: Option<bool>,
}

#[derive(Debug)]
pub struct StatsAssetInfoRelated<'a> {
  pub name: &'a str,
  pub value: Vec<&'a str>,
}

#[derive(Debug)]
pub struct StatsModule<'a> {
  pub r#type: &'static str,
  pub module_type: ModuleType,
  pub layer: Option<Cow<'a, str>>,
  pub identifier: Option<ModuleIdentifier>,
  pub name: Option<Cow<'a, str>>,
  pub name_for_condition: Option<String>,
  pub id: Option<ModuleId>,
  pub chunks: Option<Vec<&'a str>>, // has id after the call of chunkIds hook
  pub size: f64,
  pub sizes: Vec<StatsSourceTypeSize>,
  pub dependent: Option<bool>,
  pub issuer: Option<ModuleIdentifier>,
  pub issuer_name: Option<Cow<'a, str>>,
  pub issuer_id: Option<ModuleId>,
  pub issuer_path: Option<Vec<StatsModuleIssuer<'a>>>,
  pub reasons: Option<Vec<StatsModuleReason<'a>>>,
  pub assets: Option<Vec<&'a str>>,
  pub modules: Option<Vec<StatsModule<'a>>>,
  pub source: Option<&'a BoxSource>,
  pub profile: Option<StatsModuleProfile>,
  pub orphan: Option<bool>,
  pub provided_exports: Option<Vec<Atom>>,
  pub used_exports: Option<StatsUsedExports>,
  pub optimization_bailout: Option<&'a [String]>,
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
  pub factory: u64,
  pub building: u64,
}

#[derive(Debug)]
pub struct StatsOriginRecord<'a> {
  pub module: Option<ModuleIdentifier>,
  pub module_id: Option<ModuleId>,
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Cow<'a, str>,
  pub loc: String,
  pub request: &'a str,
}

#[derive(Debug)]
pub struct StatsChunk<'a> {
  pub r#type: &'static str,
  pub files: Vec<&'a str>,
  pub auxiliary_files: Vec<&'a str>,
  pub id: Option<&'a str>,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<&'a str>,
  pub size: f64,
  pub modules: Option<Vec<StatsModule<'a>>>,
  pub parents: Option<Vec<&'a str>>,
  pub children: Option<Vec<&'a str>>,
  pub siblings: Option<Vec<&'a str>>,
  pub children_by_order: HashMap<ChunkGroupOrderKey, Vec<String>>,
  pub runtime: &'a RuntimeSpec,
  pub sizes: HashMap<SourceType, f64>,
  pub reason: Option<&'a str>,
  pub rendered: bool,
  pub origins: Vec<StatsOriginRecord<'a>>,
  pub id_hints: Vec<&'a str>,
  pub hash: Option<&'a str>,
}

#[derive(Debug)]
pub struct StatsChunkGroupAsset<'a> {
  pub name: &'a str,
  pub size: usize,
}

#[derive(Debug)]
pub struct StatsChunkGroup<'a> {
  pub name: &'a str,
  pub chunks: Vec<&'a str>,
  pub assets: Vec<StatsChunkGroupAsset<'a>>,
  pub assets_size: usize,
  pub auxiliary_assets: Option<Vec<StatsChunkGroupAsset<'a>>>,
  pub auxiliary_assets_size: Option<usize>,
  pub children: Option<StatsChunkGroupChildren<'a>>,
  pub is_over_size_limit: Option<bool>,
  pub child_assets: Option<StatschunkGroupChildAssets<'a>>,
}

#[derive(Debug)]
pub struct StatsChunkGroupChildren<'a> {
  pub preload: Vec<StatsChunkGroup<'a>>,
  pub prefetch: Vec<StatsChunkGroup<'a>>,
}

#[derive(Debug)]
pub struct StatschunkGroupChildAssets<'a> {
  pub preload: Vec<&'a str>,
  pub prefetch: Vec<&'a str>,
}

#[derive(Debug)]
pub struct StatsModuleIssuer<'s> {
  pub identifier: ModuleIdentifier,
  pub name: Cow<'s, str>,
  pub id: Option<ModuleId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatsModuleReason<'s> {
  pub module_identifier: Option<ModuleIdentifier>,
  pub module_name: Option<Cow<'s, str>>,
  pub module_id: Option<ModuleId>,
  pub module_chunks: Option<u32>,
  pub resolved_module_identifier: Option<ModuleIdentifier>,
  pub resolved_module_name: Option<Cow<'s, str>>,
  pub resolved_module_id: Option<ModuleId>,

  pub r#type: Option<&'static str>,
  pub user_request: Option<&'s str>,
  pub explanation: Option<&'static str>,
  pub active: bool,
  pub loc: Option<String>,
}

#[derive(Debug)]
pub struct StatsSourceTypeSize {
  pub source_type: SourceType,
  pub size: f64,
}
