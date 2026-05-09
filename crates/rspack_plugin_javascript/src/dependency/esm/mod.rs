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

use std::fmt::Write as _;

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
  let mut ident = String::with_capacity(category.len() + 1 + request.len());
  ident.push_str(category);
  ident.push('|');
  ident.push_str(request);

  let Some(attributes) = attributes else {
    return ident.into();
  };
  let mut iter = attributes.iter();
  let Some(first) = iter.next() else {
    ident.push_str("|attrs=0");
    return ident.into();
  };
  let Some(second) = iter.next() else {
    push_esm_resource_identifier_attributes(&mut ident, std::iter::once(first), 1);
    return ident.into();
  };

  let mut attrs = Vec::with_capacity(iter.size_hint().0 + 2);
  attrs.push(first);
  attrs.push(second);
  attrs.extend(iter);
  attrs.sort_unstable_by(|a, b| a.0.cmp(b.0));
  let len = attrs.len();
  push_esm_resource_identifier_attributes(&mut ident, attrs.into_iter(), len);
  ident.into()
}

fn push_esm_resource_identifier_attributes<'a>(
  ident: &mut String,
  attrs: impl Iterator<Item = (&'a str, &'a str)>,
  len: usize,
) {
  ident.push_str("|attrs=");
  write!(ident, "{len}").expect("write to String should not fail");

  for (key, value) in attrs {
    ident.reserve(key.len() + value.len() + 8);
    push_esm_resource_identifier_attribute_part(ident, key);
    push_esm_resource_identifier_attribute_part(ident, value);
  }
}

fn push_esm_resource_identifier_attribute_part(ident: &mut String, value: &str) {
  ident.push('|');
  write!(ident, "{}:", value.len()).expect("write to String should not fail");
  ident.push_str(value);
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
      "esm|./data.json|attrs=2|9:integrity|6:sha256|4:type|4:json"
    );
  }

  #[test]
  fn creates_resource_identifier_without_attribute_delimiter_collisions() {
    let first_attributes = ImportAttributes::from_iter([("a".to_string(), "b|c=d".to_string())]);
    let second_attributes = ImportAttributes::from_iter([
      ("a".to_string(), "b".to_string()),
      ("c".to_string(), "d".to_string()),
    ]);

    let first =
      create_resource_identifier_for_esm_dependency("./data.json", Some(&first_attributes));
    let second =
      create_resource_identifier_for_esm_dependency("./data.json", Some(&second_attributes));

    assert_ne!(first, second);
  }
}
