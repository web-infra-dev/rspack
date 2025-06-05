use rspack_collections::{IdentifierMap, IdentifierSet, UkeyMap};
use rspack_error::Diagnostic;

use crate::{chunk_graph_chunk::ChunkId, ChunkRenderResult, ChunkUkey, ModuleId, RuntimeGlobals};

mod cgm_hash_artifact;
mod cgm_runtime_requirement_artifact;
mod chunk_hashes_artifact;
mod code_generation_results;
mod module_graph_cache_artifact;
mod side_effects_do_optimize_artifact;

pub use cgm_hash_artifact::*;
pub use cgm_runtime_requirement_artifact::*;
pub use chunk_hashes_artifact::*;
pub use code_generation_results::*;
pub use module_graph_cache_artifact::*;
pub use side_effects_do_optimize_artifact::*;

pub type AsyncModulesArtifact = IdentifierSet;
pub type DependenciesDiagnosticsArtifact = IdentifierMap<Vec<Diagnostic>>;
pub type ModuleIdsArtifact = IdentifierMap<ModuleId>;
pub type ChunkIdsArtifact = UkeyMap<ChunkUkey, ChunkId>;
pub type CgcRuntimeRequirementsArtifact = UkeyMap<ChunkUkey, RuntimeGlobals>;
pub type ChunkRenderArtifact = UkeyMap<ChunkUkey, ChunkRenderResult>;
