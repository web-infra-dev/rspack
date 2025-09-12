use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;
use rustc_hash::FxHashMap;

use crate::{ChunkRenderResult, ChunkUkey, ModuleId, RuntimeGlobals, chunk_graph_chunk::ChunkId};

mod cgm_hash_artifact;
mod cgm_runtime_requirement_artifact;
mod chunk_hashes_artifact;
mod chunk_render_cache_artifact;
mod code_generation_results;
mod module_graph_cache_artifact;
mod module_static_cache_artifact;
mod side_effects_do_optimize_artifact;

pub use cgm_hash_artifact::*;
pub use cgm_runtime_requirement_artifact::*;
pub use chunk_hashes_artifact::*;
pub use chunk_render_cache_artifact::ChunkRenderCacheArtifact;
pub use code_generation_results::*;
pub use module_graph_cache_artifact::*;
pub use module_static_cache_artifact::*;
pub use side_effects_do_optimize_artifact::*;

pub type AsyncModulesArtifact = IdentifierSet;
pub type DependenciesDiagnosticsArtifact = IdentifierMap<Vec<Diagnostic>>;
pub type ModuleIdsArtifact = IdentifierMap<ModuleId>;
pub type ChunkIdsArtifact = FxHashMap<ChunkUkey, ChunkId>;
pub type CgcRuntimeRequirementsArtifact = FxHashMap<ChunkUkey, RuntimeGlobals>;
pub type ChunkRenderArtifact = FxHashMap<ChunkUkey, ChunkRenderResult>;
