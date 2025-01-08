use rspack_collections::Identifier;
use rspack_core::DependencyType;
use rustc_hash::FxHashSet as HashSet;

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

pub type ModuleUkey = usize;
pub type DependencyUkey = usize;
pub type ChunkUkey = usize;
pub type AssetUkey = usize;
pub type EntrypointUkey = usize;
pub type ModuleGraphModuleUkey = usize;
pub type ExportInfoUkey = usize;
pub type VariableUkey = usize;
pub type SideEffectUkey = usize;

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
  pub modules: HashSet<ModuleUkey>,
  pub chunks: HashSet<ChunkUkey>,
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
pub struct RsdoctorChunk {
  pub ukey: ChunkUkey,
  pub name: String,
  pub initial: bool,
  pub entry: bool,
  pub assets: HashSet<AssetUkey>,
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
  pub chunks: HashSet<ChunkUkey>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleSource {
  pub module: ModuleUkey,
  pub source_size: usize,
  pub transform_size: usize,
  pub source: Option<String>,
  pub source_map: Option<String>,
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
  pub line: Option<usize>,
  pub column: Option<usize>,
  pub index: Option<usize>,
}

#[derive(Debug, Default)]
pub struct RsdoctorModuleGraph {
  pub modules: Vec<RsdoctorModule>,
  pub dependencies: Vec<RsdoctorDependency>,
}

#[derive(Debug, Default)]
pub struct RsdoctorChunkGraph {
  pub chunks: Vec<RsdoctorChunk>,
  pub entrypoints: Vec<RsdoctorEntrypoint>,
}
