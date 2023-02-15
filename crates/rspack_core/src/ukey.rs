use rspack_database::Ukey;

use crate::{Chunk, ChunkGroup};

pub type ChunkUkey = Ukey<Chunk>;
pub type ChunkGroupUkey = Ukey<ChunkGroup>;
