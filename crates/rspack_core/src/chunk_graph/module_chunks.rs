use rspack_collections::SsoHashSet;

use crate::ChunkUkey;

/// A module's chunk memberships, with small sets stored inline.
pub type ModuleChunks = SsoHashSet<ChunkUkey>;
