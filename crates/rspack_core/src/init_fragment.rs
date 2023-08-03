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

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum InitFragmentStage {
  StageConstants,
  StageAsyncBoundary,
  StageHarmonyExportsCompatibility,
  StageHarmonyExports,
  StageHarmonyImports,
  StageProvides,
  StageAsyncDependencies,
  StageAsyncHarmonyImports,
}

pub type ChunkInitFragments = HashMap<String, InitFragment>;
