mod dynamic_import_origin_dependency;
mod esm_import_dependency;
mod import_dependency;
mod mock_method_dependency;
mod mock_module_id_dependency;
mod module_path_name_dependency;
mod parser_plugin;
mod plugin;
mod require_resolve_origin_dependency;
mod url_dependency;

pub use plugin::*;

fn render_mock_scope(template: &mut rspack_core::ModuleCodeTemplate) -> String {
  use rspack_core::RuntimeGlobals;
  let requirement = if template.render_mode().is_legacy() {
    RuntimeGlobals::REQUIRE
  } else {
    RuntimeGlobals::REQUIRE_SCOPE
  };
  template.render_runtime_globals(&requirement)
}
