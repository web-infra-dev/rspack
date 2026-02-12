use rspack_collections::Identifier;
use rspack_core::DependencyType;
use rustc_hash::FxHashSet as HashSet;

pub type ConnectionUkey = i32;

#[derive(Debug, Default)]
pub enum ModuleKind {
  #[default]
  Normal,
  Concatenated,
}

impl From<ModuleKind> for String {
  fn from(value: ModuleKind) -> Self {
    match value {
      ModuleKind::Normal => "normal".to_string(),
      ModuleKind::Concatenated => "concatenated".to_string(),
    }
  }
}

pub type ModuleUkey = i32;
pub type DependencyUkey = i32;
pub type ChunkUkey = i32;
pub type AssetUkey = i32;
pub type EntrypointUkey = i32;
pub type ModuleGraphModuleUkey = i32;
pub type ExportInfoUkey = i32;
pub type VariableUkey = i32;
pub type SideEffectUkey = i32;

#[derive(Debug, Default)]
pub struct RsdoctorStatsModuleIssuer {
  pub ukey: Option<ModuleUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorSideEffectLocation {
  pub location: String,
  pub node_type: String,
  pub module: ModuleUkey,
  pub request: String,
}

#[derive(Debug, Default)]
pub struct RsdoctorModule {
  pub ukey: ModuleUkey,
  pub identifier: Identifier,
  pub path: String,
  pub is_entry: bool,
  pub kind: ModuleKind,
  pub layer: Option<String>,
  pub dependencies: HashSet<DependencyUkey>,
  pub imported: HashSet<ModuleUkey>,
  pub chunks: HashSet<ChunkUkey>,
  pub modules: HashSet<ModuleUkey>,
  pub belong_modules: HashSet<ModuleUkey>,
  pub issuer_path: Option<Vec<RsdoctorStatsModuleIssuer>>,
  pub bailout_reason: HashSet<String>,
  pub side_effects: Option<bool>,
  pub side_effects_locations: Vec<RsdoctorSideEffectLocation>,
}

#[derive(Debug, Default)]
pub struct RsdoctorDependency {
  pub ukey: DependencyUkey,
  pub kind: DependencyType,
  pub request: String,
  pub module: ModuleUkey,
  pub dependency: ModuleUkey,
}

#[derive(Debug, Default)]
pub struct RsdoctorConnection {
  pub ukey: ConnectionUkey,
  pub dependency_id: String,
  pub module: ModuleUkey,
  pub origin_module: Option<ModuleUkey>,
  pub resolved_module: ModuleUkey,
  pub dependency_type: String,
  pub user_request: String,
  pub loc: Option<String>,
  pub active: bool,
}

#[derive(Debug, Default)]
pub struct RsdoctorChunk {
  pub ukey: ChunkUkey,
  pub name: String,
  pub initial: bool,
  pub entry: bool,
  pub dependencies: HashSet<ChunkUkey>,
  pub imported: HashSet<ChunkUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorEntrypoint {
  pub ukey: EntrypointUkey,
  pub name: String,
  pub chunks: HashSet<ChunkUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorAsset {
  pub ukey: AssetUkey,
  pub path: String,
  pub size: i32,
  pub chunks: HashSet<ChunkUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorChunkAssets {
  pub chunk: ChunkUkey,
  pub assets: HashSet<AssetUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorEntrypointAssets {
  pub entrypoint: EntrypointUkey,
  pub assets: HashSet<AssetUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleGraphModule {
  pub ukey: ModuleGraphModuleUkey,
  pub module: ModuleUkey,
  pub exports: Vec<ExportInfoUkey>,
  pub side_effects: Vec<SideEffectUkey>,
  pub variables: Vec<VariableUkey>,
  pub dynamic: bool,
}

#[derive(Debug, Default)]
pub struct RsdoctorSideEffect {
  pub ukey: SideEffectUkey,
  pub name: String,
  pub origin_name: Option<String>,
  pub module: ModuleUkey,
  pub identifier: RsdoctorStatement,
  pub is_name_space: bool,
  pub from_dependency: Option<DependencyUkey>,
  pub exports: Vec<ExportInfoUkey>,
  pub variable: Option<VariableUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorVariable {
  pub ukey: VariableUkey,
  pub name: String,
  pub module: ModuleUkey,
  pub used_info: String,
  pub identififer: RsdoctorStatement,
  pub exported: Option<ExportInfoUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorExportInfo {
  pub ukey: ExportInfoUkey,
  pub name: String,
  pub from: Option<ExportInfoUkey>,
  pub variable: Option<VariableUkey>,
  pub identifier: Option<RsdoctorStatement>,
  pub side_effects: Vec<SideEffectUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorStatement {
  pub module: ModuleUkey,
  pub source_position: Option<RsdoctorSourceRange>,
  pub transformed_position: RsdoctorSourceRange,
}

#[derive(Debug, Default)]
pub struct RsdoctorSourceRange {
  pub start: RsdoctorSourcePosition,
  pub end: Option<RsdoctorSourcePosition>,
}

#[derive(Debug, Default)]
pub struct RsdoctorSourcePosition {
  pub line: Option<i32>,
  pub column: Option<i32>,
  pub index: Option<i32>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleGraph {
  pub modules: Vec<RsdoctorModule>,
  pub dependencies: Vec<RsdoctorDependency>,
  pub connections: Vec<RsdoctorConnection>,
  pub chunk_modules: Vec<RsdoctorChunkModules>,
}

#[derive(Debug, Default)]
pub struct RsdoctorAssetPatch {
  pub assets: Vec<RsdoctorAsset>,
  pub chunk_assets: Vec<RsdoctorChunkAssets>,
  pub entrypoint_assets: Vec<RsdoctorEntrypointAssets>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleIdsPatch {
  pub module_ids: Vec<RsdoctorModuleId>,
}

pub type RsdoctorJsonModuleSizes = rustc_hash::FxHashMap<String, i32>;

#[derive(Debug, Default)]
pub struct RsdoctorModuleSourcesPatch {
  pub module_original_sources: Vec<RsdoctorModuleOriginalSource>,
  pub json_module_sizes: RsdoctorJsonModuleSizes,
}

#[derive(Debug, Default)]
pub struct RsdoctorChunkModules {
  pub chunk: ChunkUkey,
  pub modules: Vec<ModuleUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorChunkGraph {
  pub chunks: Vec<RsdoctorChunk>,
  pub entrypoints: Vec<RsdoctorEntrypoint>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleId {
  pub module: ModuleUkey,
  pub render_id: String,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleOriginalSource {
  pub module: ModuleUkey,
  pub source: String,
  pub size: i32,
}
