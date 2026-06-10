// BE CAREFUL:
// Add more fields to this struct should result in adding new fields to options builder.
// `impl From<Experiments> for ExperimentsBuilder` should be updated.
#[derive(Debug)]
pub struct Experiments {
  pub css: bool,
  pub defer_import: bool,
  pub source_import: bool,
  pub pure_functions: bool,
}
