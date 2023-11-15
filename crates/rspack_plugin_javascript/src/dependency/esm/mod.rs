mod harmony_compatibility_dependency;
mod harmony_export_expression_dependency;
mod harmony_export_header_dependency;
mod harmony_export_imported_specifier_dependency;
mod harmony_export_specifier_dependency;
mod harmony_import_dependency;
mod harmony_import_specifier_dependency;
mod import_dependency;
mod import_eager_dependency;

pub use harmony_compatibility_dependency::HarmonyCompatibilityDependency;
pub use harmony_export_expression_dependency::HarmonyExportExpressionDependency;
pub use harmony_export_expression_dependency::{AnonymousFunctionRangeInfo, DEFAULT_EXPORT};
pub use harmony_export_header_dependency::HarmonyExportHeaderDependency;
pub use harmony_export_imported_specifier_dependency::HarmonyExportImportedSpecifierDependency;
pub use harmony_export_specifier_dependency::HarmonyExportSpecifierDependency;
pub use harmony_import_dependency::harmony_import_dependency_apply;
pub use harmony_import_dependency::{HarmonyImportSideEffectDependency, Specifier};
pub use harmony_import_specifier_dependency::HarmonyImportSpecifierDependency;
pub use import_dependency::ImportDependency;
pub use import_eager_dependency::ImportEagerDependency;
use rspack_core::DependencyCategory;

pub fn create_resource_identifier_for_esm_dependency(request: &str) -> String {
  format!("{}|{}", DependencyCategory::Esm, &request)
}
