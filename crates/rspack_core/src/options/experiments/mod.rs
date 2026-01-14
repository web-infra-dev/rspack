use crate::incremental::IncrementalOptions;

// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<Experiments> for ExperimentsBuilder` should be updated.
#[derive(Debug)]
pub struct Experiments {
  pub incremental: IncrementalOptions,
  pub css: bool,
  pub lazy_barrel: bool,
  pub defer_import: bool,
}
