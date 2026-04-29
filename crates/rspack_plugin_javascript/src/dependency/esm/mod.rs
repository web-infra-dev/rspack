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
mod import_weak_dependency;
mod provide_dependency;

use rspack_core::{DependencyCategory, ImportAttributes, ResourceIdentifier};

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
  import_weak_dependency::{ImportWeakDependency, ImportWeakDependencyTemplate},
  provide_dependency::{ProvideDependency, ProvideDependencyTemplate},
};

pub fn create_resource_identifier_for_esm_dependency(
  request: &str,
  attributes: Option<&ImportAttributes>,
) -> ResourceIdentifier {
  let category = DependencyCategory::Esm.as_str();
  let attrs = attributes
    .map(|attributes| {
      let mut attrs = attributes.iter().collect::<Vec<_>>();
      attrs.sort_unstable_by(|a, b| a.0.cmp(b.0));
      attrs
    })
    .unwrap_or_default();
  let attributes_len = attrs
    .iter()
    .map(|(key, value)| 2 + key.len() + value.len())
    .sum::<usize>();

  let mut ident = String::with_capacity(category.len() + 1 + request.len() + attributes_len);
  ident.push_str(category);
  ident.push('|');
  ident.push_str(request);
  if attributes.is_some() {
    for (key, value) in attrs {
      ident.push('|');
      ident.push_str(key);
      ident.push('=');
      ident.push_str(value);
    }
  }
  ident.into()
}

#[cfg(test)]
mod tests {
  use rspack_core::ImportAttributes;

  use super::create_resource_identifier_for_esm_dependency;

  #[test]
  fn creates_resource_identifier_with_sorted_import_attributes() {
    let attributes = ImportAttributes::from_iter([
      ("type".to_string(), "json".to_string()),
      ("integrity".to_string(), "sha256".to_string()),
    ]);

    let ident = create_resource_identifier_for_esm_dependency("./data.json", Some(&attributes));

    assert_eq!(
      ident.to_string(),
      "esm|./data.json|integrity=sha256|type=json"
    );
  }
}
