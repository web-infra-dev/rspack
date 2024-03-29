mod harmony_compatibility_dependency;
mod harmony_export_expression_dependency;
mod harmony_export_header_dependency;
mod harmony_export_imported_specifier_dependency;
mod harmony_export_specifier_dependency;
mod harmony_import_dependency;
mod harmony_import_specifier_dependency;
mod import_dependency;
mod import_eager_dependency;
mod provide_dependency;

use rspack_core::DependencyCategory;

pub use self::harmony_compatibility_dependency::HarmonyCompatibilityDependency;
pub use self::harmony_export_expression_dependency::*;
pub use self::harmony_export_header_dependency::HarmonyExportHeaderDependency;
pub use self::harmony_export_imported_specifier_dependency::HarmonyExportImportedSpecifierDependency;
pub use self::harmony_export_imported_specifier_dependency::GET_MODE_MAP;
pub use self::harmony_export_specifier_dependency::HarmonyExportSpecifierDependency;
pub use self::harmony_import_dependency::get_import_emitted_runtime;
pub use self::harmony_import_dependency::harmony_import_dependency_apply;
pub use self::harmony_import_dependency::{HarmonyImportSideEffectDependency, Specifier};
pub use self::harmony_import_specifier_dependency::HarmonyImportSpecifierDependency;
pub use self::import_dependency::ImportDependency;
pub use self::import_eager_dependency::ImportEagerDependency;
pub use self::provide_dependency::ProvideDependency;

pub fn create_resource_identifier_for_esm_dependency(request: &str) -> String {
  format!("{}|{}", DependencyCategory::Esm, &request)
}
