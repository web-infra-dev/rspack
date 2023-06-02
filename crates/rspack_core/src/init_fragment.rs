use bitflags::bitflags;
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Clone, Hash)]
pub struct InitFragment {
  pub content: String,
  pub stage: InitFragmentStage,
  pub end_content: Option<String>,
}

impl InitFragment {
  pub fn new(content: String, stage: InitFragmentStage, end_content: Option<String>) -> Self {
    InitFragment {
      content,
      stage,
      end_content,
    }
  }
}

bitflags! {
  pub struct InitFragmentStage: u8 {
    const STAGE_CONSTANTS = 1 << 0;
    const STAGE_ASYNC_BOUNDARY = 1 << 1;
    const STAGE_HARMONY_EXPORTS = 1 << 2;
    const STAGE_HARMONY_IMPORTS = 1 << 3;
    const STAGE_PROVIDES = 1 << 4;
    const STAGE_ASYNC_DEPENDENCIES = 1 << 5;
    const STAGE_ASYNC_HARMONY_IMPORTS = 1 << 6;
  }
}

pub type ChunkInitFragments = HashMap<String, InitFragment>;
