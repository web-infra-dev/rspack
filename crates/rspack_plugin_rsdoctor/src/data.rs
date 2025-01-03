use rspack_collections::Identifier;
use rspack_core::DependencyType;
use rustc_hash::FxHashSet as HashSet;

pub enum ModuleKind {
  Normal,
  Concatenated,
}

pub type ModuleUkey = usize;
pub type DependencyUkey = usize;
pub type ChunkUkey = usize;
pub type AssetUkey = usize;
pub type EntrypointUkey = usize;

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

pub struct RsdoctorDependency {
  pub ukey: DependencyUkey,
  pub kind: DependencyType,
  pub request: String,
  pub module: ModuleUkey,
  pub dependency: ModuleUkey,
}

pub struct RsdoctorChunk {
  pub ukey: ChunkUkey,
  pub name: String,
  pub initial: bool,
  pub entry: bool,
  pub assets: HashSet<AssetUkey>,
  pub dependencies: HashSet<ChunkUkey>,
  pub imported: HashSet<ChunkUkey>,
}

pub struct RsdoctorEntrypoint {
  pub ukey: EntrypointUkey,
  pub name: String,
  pub chunks: HashSet<ChunkUkey>,
}

pub struct RsdoctorAsset {
  pub ukey: AssetUkey,
  pub path: String,
  pub content: String,
  pub chunks: HashSet<ChunkUkey>,
}

pub struct RsdoctorModuleSource {
  pub source_size: usize,
  pub transform_size: usize,
  pub source: Option<String>,
  pub source_map: Option<String>,
}
