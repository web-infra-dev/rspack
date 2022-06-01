use crate::{BundleOptions, Platform};

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

pub fn rspack_runtime(options: &RuntimeOptions, bundle_options: &BundleOptions) -> String {
  let mut runtime = "".to_string();
  if options.hmr {
    runtime += include_str!("hmr.js");
  }
  if options.module {
    let module_code = match bundle_options.platform {
      Platform::Browser => include_str!("module_in_browser.js"),
      Platform::Node => include_str!("module_in_node.js"),
    };
    runtime += module_code;
  }
  runtime
}
