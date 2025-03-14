mod esm_compatibility_dependency;
mod esm_export_expression_dependency;
mod esm_export_header_dependency;
mod esm_export_imported_specifier_dependency;
mod esm_export_specifier_dependency;
mod esm_import_dependency;
mod esm_import_specifier_dependency;
mod external_module_dependency;
mod import_dependency;
mod import_eager_dependency;
mod provide_dependency;

use rspack_core::{DependencyCategory, ImportAttributes};
use rspack_util::json_stringify;

pub use self::{
  esm_compatibility_dependency::ESMCompatibilityDependency,
  esm_export_expression_dependency::*,
  esm_export_header_dependency::ESMExportHeaderDependency,
  esm_export_imported_specifier_dependency::ESMExportImportedSpecifierDependency,
  esm_export_specifier_dependency::ESMExportSpecifierDependency,
  esm_import_dependency::{
    esm_import_dependency_apply, import_emitted_runtime, ESMImportSideEffectDependency,
  },
  esm_import_specifier_dependency::ESMImportSpecifierDependency,
  external_module_dependency::ExternalModuleDependency,
  import_dependency::ImportDependency,
  import_eager_dependency::ImportEagerDependency,
  provide_dependency::ProvideDependency,
};

pub fn create_resource_identifier_for_esm_dependency(
  request: &str,
  attributes: Option<&ImportAttributes>,
) -> String {
  let mut ident = format!("{}|{}", DependencyCategory::Esm, &request);
  if let Some(attributes) = attributes {
    ident += &json_stringify(&attributes);
  }
  ident
}
