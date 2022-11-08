use std::fmt::Debug;

use napi::bindgen_prelude::*;

#[napi(object)]
pub struct PluginCallbacks {
  pub done_callback: JsFunction,
  pub process_assets_callback: JsFunction,
  pub compilation_callback: JsFunction,
  pub this_compilation_callback: JsFunction,
}

impl Debug for PluginCallbacks {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PluginCallbacks")
      .field("done_callback", &"done_adapter")
      .field("procss_assets_callback", &"process_assets_adapter")
      .field("compilation_callback", &"compilation_adapter")
      .field("this_compilation_callback", &"this_compilation_adapter")
      .finish()
  }
}
