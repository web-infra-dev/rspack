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
mod import_meta_resolve_dependency;
mod import_meta_resolve_header_dependency;
mod provide_dependency;

use rspack_core::{DependencyCategory, ImportAttributes, ResourceIdentifier};
use rspack_util::json_stringify;

pub use self::{
  esm_compatibility_dependency::{ESMCompatibilityDependency, ESMCompatibilityDependencyTemplate},
  esm_export_expression_dependency::{
    DeclarationId, DeclarationInfo, ESMExportExpressionDependency,
    ESMExportExpressionDependencyTemplate,
  },
  esm_export_header_dependency::{ESMExportHeaderDependency, ESMExportHeaderDependencyTemplate},
  esm_export_imported_specifier_dependency::{
    ESMExportImportedSpecifierDependency, ESMExportImportedSpecifierDependencyTemplate,
  },
  esm_export_specifier_dependency::{
    ESMExportSpecifierDependency, ESMExportSpecifierDependencyTemplate,
  },
  esm_import_dependency::{
    ESMImportSideEffectDependency, ESMImportSideEffectDependencyTemplate,
    esm_import_dependency_apply, import_emitted_runtime,
  },
  esm_import_specifier_dependency::{
    ESMImportSpecifierDependency, ESMImportSpecifierDependencyTemplate,
  },
  external_module_dependency::{ExternalModuleDependency, ExternalModuleDependencyTemplate},
  import_dependency::{ImportDependency, ImportDependencyTemplate},
  import_eager_dependency::{ImportEagerDependency, ImportEagerDependencyTemplate},
  import_meta_resolve_dependency::{
    ImportMetaResolveDependency, ImportMetaResolveDependencyTemplate,
  },
  import_meta_resolve_header_dependency::{
    ImportMetaResolveHeaderDependency, ImportMetaResolveHeaderDependencyTemplate,
  },
  provide_dependency::{ProvideDependency, ProvideDependencyTemplate},
};

pub fn create_resource_identifier_for_esm_dependency(
  request: &str,
  attributes: Option<&ImportAttributes>,
) -> ResourceIdentifier {
  let mut ident = format!("{}|{}", DependencyCategory::Esm, &request);
  if let Some(attributes) = attributes {
    ident += &json_stringify(&attributes);
  }
  ident.into()
}
