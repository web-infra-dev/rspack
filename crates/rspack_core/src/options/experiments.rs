use once_cell::sync::OnceCell;

#[derive(Debug, Default)]
pub struct IncrementalRebuild {
  pub make: Option<IncrementalRebuildMakeState>,
  pub emit_asset: bool,
}

#[derive(Debug, Default)]
pub struct IncrementalRebuildMakeState {
  first: OnceCell<()>,
}

impl IncrementalRebuildMakeState {
  pub fn is_first(&self) -> bool {
    self.first.get().is_none()
  }

  pub fn set_is_not_first(&self) {
    self.first.get_or_init(|| ());
  }
}

#[derive(Debug, Default)]
pub struct Experiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: IncrementalRebuild,
  pub async_web_assembly: bool,
  pub new_split_chunks: bool,
  pub css: bool,
}
