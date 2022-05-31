#[derive(Default, Debug)]
pub struct RuntimeOptions {
  pub hmr: bool,
  pub module: bool,
}

impl RuntimeOptions {
  pub fn default() -> RuntimeOptions {
    RuntimeOptions {
      hmr: false,
      module: true,
    }
  }
}

pub fn rspack_runtime(options: &RuntimeOptions) -> String {
  let mut runtime = "".to_string();
  if options.hmr {
    runtime += include_str!("hmr.js");
  }
  if options.module {
    runtime += include_str!("module.js");
  }
  runtime
}
