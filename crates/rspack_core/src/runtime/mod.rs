#[derive(Default, Debug)]
pub struct RuntimeOptions {
  pub hmr: bool,
  pub polyfill: bool,
  pub module: bool,
}
impl RuntimeOptions {
  pub fn default() -> RuntimeOptions {
    RuntimeOptions {
      hmr: true,
      polyfill: true,
      module: true,
    }
  }
}
pub fn rspack_runtime(options: &RuntimeOptions) -> String {
  let polyfill = include_str!("polyfill.js");
  let module = include_str!("module.js");
  let hmr = include_str!("hmr.js");
  let mut runtime = "".to_string();
  if options.polyfill {
    runtime += polyfill;
  }
  if options.module {
    runtime += module
  }
  if options.hmr {
    runtime += hmr
  }
  runtime
}
