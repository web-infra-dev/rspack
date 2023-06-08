#[derive(Debug, Default)]
pub struct IncrementalRebuild {
  pub make: bool,
  pub emit_asset: bool,
}

#[derive(Debug, Default)]
pub struct Experiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: IncrementalRebuild,
  pub async_web_assembly: bool,
  pub new_split_chunks: bool,
  pub css: bool,
}
