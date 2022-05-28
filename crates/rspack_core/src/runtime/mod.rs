mod inject_module;

#[derive(Default, Debug)]
pub struct RuntimeOptions {
  pub hmr: bool,
  pub module: bool,
}

pub struct RuntimeInfo {
  pub external: Vec<String>,
}

impl RuntimeOptions {
  pub fn default() -> RuntimeOptions {
    RuntimeOptions {
      hmr: true,
      module: true,
    }
  }
}

pub fn rspack_runtime(options: &RuntimeOptions, info: RuntimeInfo) -> String {
  let hmr = include_str!("hmr.js");
  let mut runtime = "".to_string();
  if options.module {
    let module = if info.external.is_empty() {
      inject_module::ModuleRuntime::inject()
    } else {
      inject_module::ModuleRuntime::inject_with_external(&info.external)
    };
    runtime += &module;
  }
  if options.hmr {
    runtime += hmr
  }
  runtime
}
