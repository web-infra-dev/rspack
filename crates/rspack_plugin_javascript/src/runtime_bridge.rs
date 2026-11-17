use std::fmt::Write;

use rspack_core::{RuntimeCodeTemplate, RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeVariable};

/// Establish the legacy names in the runtime's scope. Keep the callable separate:
/// webpack's require.r marks namespace objects; the native context's r is its loader.
pub(crate) fn render_runtime_bridge(runtime_template: &RuntimeCodeTemplate) -> String {
  let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
  let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
  let cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
  let mut code = format!(
    "// webpack runtime module bridge\nvar __webpack_require__ = {require}, __webpack_modules__ = {modules}, __webpack_module_cache__ = {cache};\n"
  );
  if runtime_template.render_mode() == RuntimeGlobalsRenderMode::RspackExport {
    writeln!(code, "var __rspack_context = {{}};\nObject.defineProperty({require}, '__rspack_compat_context__', {{value: __rspack_context}});").expect("write to string");
  }
  code.push_str(include_str!("runtime/webpack_runtime_bridge.js"));
  code
}

/// Install accessors immediately before the provider executes. An earlier plugin
/// may enumerate the loader, wrap it, or replace it altogether.
pub(crate) fn render_runtime_bridge_fields(
  fields: RuntimeGlobals,
  template: &RuntimeCodeTemplate,
) -> String {
  let mut code = String::new();
  for (_, global) in fields.renderable_require_scope().iter_names() {
    let (Some(key), Some(native_key), Some(lexical)) = (
      global.property_name(),
      global.rspack_context_property_name(),
      global.to_lexical_name(),
    ) else {
      continue;
    };
    let binding = if global == RuntimeGlobals::MODULE_FACTORIES {
      template.render_runtime_variable(&RuntimeVariable::Modules)
    } else if global == RuntimeGlobals::MODULE_CACHE {
      template.render_runtime_variable(&RuntimeVariable::ModuleCache)
    } else {
      lexical.to_owned()
    };
    writeln!(code, "__rspack_bind_runtime__({}, {}, function() {{ return {binding}; }}, function(value) {{ {binding} = value; }});", rspack_util::json_stringify(key), rspack_util::json_stringify(native_key)).expect("write to string");
  }
  code
}

pub fn render_runtime_bridge_aliases(template: &RuntimeCodeTemplate) -> String {
  let context = template.render_runtime_argument();
  if template.render_mode() == RuntimeGlobalsRenderMode::RspackExport {
    format!(
      "var __rspack_context = {context}.__rspack_compat_context__, __webpack_require__ = __rspack_context.r, __webpack_modules__ = __webpack_require__.m, __webpack_module_cache__ = __webpack_require__.c;\n"
    )
  } else {
    format!(
      "var __webpack_require__ = {context}.r, __webpack_modules__ = __webpack_require__.m, __webpack_module_cache__ = __webpack_require__.c;\n"
    )
  }
}
