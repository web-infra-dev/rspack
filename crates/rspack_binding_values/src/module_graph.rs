use napi_derive::napi;
use rspack_core::ModuleIdentifier;

use crate::JsCompilation;

#[napi(js_name = "__module_graph_inner_is_async")]
pub fn is_async(js_module_identifier: String, compilation: &JsCompilation) -> Option<bool> {
  let compilation = &compilation.0;
  let module_graph = compilation.get_module_graph();
  module_graph.is_async(&ModuleIdentifier::from(js_module_identifier))
}
