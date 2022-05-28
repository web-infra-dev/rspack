#[derive(Default, Debug)]
pub struct RuntimeOptions {
  pub hmr: bool,
  pub module: bool,
}
impl RuntimeOptions {
  pub fn default() -> RuntimeOptions {
    RuntimeOptions {
      hmr: true,
      module: true,
    }
  }
}
pub fn rspack_runtime(options: &RuntimeOptions) -> String {
  let module = include_str!("module.js");
  let hmr = include_str!("hmr.js");
  let mut runtime = "".to_string();
  if options.module {
    runtime += module
  }
  if options.hmr {
    runtime += hmr
  }
  runtime
}
