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

use rspack_core::DependencyCategory;
use rspack_core::ImportAttributes;
use rspack_util::json_stringify;

pub use self::esm_compatibility_dependency::ESMCompatibilityDependency;
pub use self::esm_export_expression_dependency::*;
pub use self::esm_export_header_dependency::ESMExportHeaderDependency;
pub use self::esm_export_imported_specifier_dependency::ESMExportImportedSpecifierDependency;
pub use self::esm_export_specifier_dependency::ESMExportSpecifierDependency;
pub use self::esm_import_dependency::esm_import_dependency_apply;
pub use self::esm_import_dependency::import_emitted_runtime;
pub use self::esm_import_dependency::ESMImportSideEffectDependency;
pub use self::esm_import_specifier_dependency::ESMImportSpecifierDependency;
pub use self::external_module_dependency::ExternalModuleDependency;
pub use self::import_dependency::ImportDependency;
pub use self::import_eager_dependency::ImportEagerDependency;
pub use self::provide_dependency::ProvideDependency;

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
