#[derive(Debug, Default)]
pub struct Experiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: bool,
  pub async_web_assembly: bool,
}
