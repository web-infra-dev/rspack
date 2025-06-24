#[cfg(test)]
mod context_aware_tests {
  use rspack_core::DependencyRange;
  use swc_core::atoms::Atom;

  use super::super::{CommonJsExportsDependency, ExportContext, ExportsBase};

  #[test]
  fn test_export_context_creation() {
    // Test individual assignment context
    let individual_dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::Exports,
      vec![Atom::from("testFunction")],
      ExportContext::IndividualAssignment,
    );

    assert_eq!(
      *individual_dep.get_context(),
      ExportContext::IndividualAssignment
    );
    assert_eq!(*individual_dep.get_base(), ExportsBase::Exports);
    assert_eq!(individual_dep.get_names()[0], "testFunction");

    // Test object literal property context
    let object_dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::ModuleExports,
      vec![Atom::from("calculateSum")],
      ExportContext::ObjectLiteralProperty,
    );

    assert_eq!(
      *object_dep.get_context(),
      ExportContext::ObjectLiteralProperty
    );
    assert_eq!(*object_dep.get_base(), ExportsBase::ModuleExports);

    // Test variable assignment context
    let variable_dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      None, // No value range for variable assignments
      ExportsBase::Exports,
      vec![Atom::from("formatPath")],
      ExportContext::VariableAssignment,
    );

    assert_eq!(
      *variable_dep.get_context(),
      ExportContext::VariableAssignment
    );
    assert!(variable_dep.get_value_range().is_none());

    // Test define property context
    let define_dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::DefinePropertyExports,
      vec![Atom::from("propertyName")],
      ExportContext::DefineProperty,
    );

    assert_eq!(*define_dep.get_context(), ExportContext::DefineProperty);
    assert_eq!(*define_dep.get_base(), ExportsBase::DefinePropertyExports);
  }

  #[test]
  fn test_export_context_enum_values() {
    // Test all context variants exist and are distinct
    let contexts = [
      ExportContext::IndividualAssignment,
      ExportContext::ObjectLiteralProperty,
      ExportContext::VariableAssignment,
      ExportContext::DefineProperty,
    ];

    // Ensure all contexts are unique
    for (i, context1) in contexts.iter().enumerate() {
      for (j, context2) in contexts.iter().enumerate() {
        if i != j {
          assert_ne!(
            std::mem::discriminant(context1),
            std::mem::discriminant(context2)
          );
        }
      }
    }
  }

  #[test]
  fn test_exports_base_detection() {
    // Test exports base type detection helpers

    // Test is_exports
    assert!(ExportsBase::Exports.is_exports());
    assert!(ExportsBase::DefinePropertyExports.is_exports());
    assert!(!ExportsBase::ModuleExports.is_exports());
    assert!(!ExportsBase::This.is_exports());

    // Test is_module_exports
    assert!(ExportsBase::ModuleExports.is_module_exports());
    assert!(ExportsBase::DefinePropertyModuleExports.is_module_exports());
    assert!(!ExportsBase::Exports.is_module_exports());
    assert!(!ExportsBase::This.is_module_exports());

    // Test is_this
    assert!(ExportsBase::This.is_this());
    assert!(ExportsBase::DefinePropertyThis.is_this());
    assert!(!ExportsBase::Exports.is_this());
    assert!(!ExportsBase::ModuleExports.is_this());

    // Test is_define_property
    assert!(ExportsBase::DefinePropertyExports.is_define_property());
    assert!(ExportsBase::DefinePropertyModuleExports.is_define_property());
    assert!(ExportsBase::DefinePropertyThis.is_define_property());
    assert!(!ExportsBase::Exports.is_define_property());
    assert!(!ExportsBase::ModuleExports.is_define_property());
    assert!(!ExportsBase::This.is_define_property());

    // Test is_expression
    assert!(ExportsBase::Exports.is_expression());
    assert!(ExportsBase::ModuleExports.is_expression());
    assert!(ExportsBase::This.is_expression());
    assert!(!ExportsBase::DefinePropertyExports.is_expression());
  }

  #[test]
  fn test_dependency_range_handling() {
    // Test range handling for different contexts
    let range = DependencyRange::new(5, 15);
    let value_range = DependencyRange::new(25, 45);

    // Individual assignment should have both ranges
    let individual_dep = CommonJsExportsDependency::new(
      range,
      Some(value_range),
      ExportsBase::Exports,
      vec![Atom::from("testExport")],
      ExportContext::IndividualAssignment,
    );

    assert_eq!(individual_dep.get_range().start, 5);
    assert_eq!(individual_dep.get_range().end, 15);
    assert!(individual_dep.get_value_range().is_some());
    assert_eq!(individual_dep.get_value_range().unwrap().start, 25);
    assert_eq!(individual_dep.get_value_range().unwrap().end, 45);

    // Variable assignment typically doesn't need value range
    let variable_dep = CommonJsExportsDependency::new(
      DependencyRange::new(5, 15), // Create new range instead of reusing
      None,
      ExportsBase::Exports,
      vec![Atom::from("variableName")],
      ExportContext::VariableAssignment,
    );

    assert_eq!(variable_dep.get_range().start, 5);
    assert_eq!(variable_dep.get_range().end, 15);
    assert!(variable_dep.get_value_range().is_none());
  }

  #[test]
  fn test_multiple_export_names() {
    // Test handling of multiple export names (nested properties)
    let multiple_names = vec![
      Atom::from("nested"),
      Atom::from("property"),
      Atom::from("path"),
    ];

    let nested_dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::ModuleExports,
      multiple_names.clone(),
      ExportContext::IndividualAssignment,
    );

    assert_eq!(nested_dep.get_names().len(), 3);
    assert_eq!(nested_dep.get_names()[0], "nested");
    assert_eq!(nested_dep.get_names()[1], "property");
    assert_eq!(nested_dep.get_names()[2], "path");
  }

  #[test]
  fn test_context_specific_patterns() {
    // Test context patterns match expected use cases

    // Individual assignment: exports.foo = value
    let individual = CommonJsExportsDependency::new(
      DependencyRange::new(0, 11),        // "exports.foo"
      Some(DependencyRange::new(14, 19)), // "value"
      ExportsBase::Exports,
      vec![Atom::from("foo")],
      ExportContext::IndividualAssignment,
    );

    assert!(matches!(
      *individual.get_context(),
      ExportContext::IndividualAssignment
    ));
    assert!(individual.get_value_range().is_some());

    // Object literal property: { foo } in module.exports = { foo }
    let object_prop = CommonJsExportsDependency::new(
      DependencyRange::new(20, 23),       // "foo" property name span
      Some(DependencyRange::new(18, 25)), // "{ foo }" object span
      ExportsBase::ModuleExports,
      vec![Atom::from("foo")],
      ExportContext::ObjectLiteralPropertyFirst,
    );

    assert!(matches!(
      *object_prop.get_context(),
      ExportContext::ObjectLiteralPropertyFirst
    ));
    assert!(object_prop.get_base().is_module_exports());

    // Variable assignment: variable = exports.foo
    let variable = CommonJsExportsDependency::new(
      DependencyRange::new(10, 21), // "exports.foo" span
      None,                         // No value range needed
      ExportsBase::Exports,
      vec![Atom::from("foo")],
      ExportContext::VariableAssignment,
    );

    assert!(matches!(
      *variable.get_context(),
      ExportContext::VariableAssignment
    ));
    assert!(variable.get_value_range().is_none());

    // Define property: Object.defineProperty(exports, "foo", { ... })
    let define_prop = CommonJsExportsDependency::new(
      DependencyRange::new(0, 40),        // Full defineProperty call span
      Some(DependencyRange::new(42, 50)), // Value object span
      ExportsBase::DefinePropertyExports,
      vec![Atom::from("foo")],
      ExportContext::DefineProperty,
    );

    assert!(matches!(
      *define_prop.get_context(),
      ExportContext::DefineProperty
    ));
    assert!(define_prop.get_base().is_define_property());
  }

  #[test]
  fn test_resource_identifier_generation() {
    // Test that dependencies with same base and names generate same resource identifier
    let dep1 = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::Exports,
      vec![Atom::from("testName")],
      ExportContext::IndividualAssignment,
    );

    let dep2 = CommonJsExportsDependency::new(
      DependencyRange::new(50, 60),         // Different range
      Some(DependencyRange::new(70, 80)),   // Different value range
      ExportsBase::Exports,                 // Same base
      vec![Atom::from("testName")],         // Same names
      ExportContext::ObjectLiteralProperty, // Different context (shouldn't affect resource ID)
    );

    // Resource identifiers should be the same for same base + names
    assert_eq!(
      dep1.get_resource_identifier(),
      dep2.get_resource_identifier()
    );

    // Different names should produce different resource identifiers
    let dep3 = CommonJsExportsDependency::new(
      DependencyRange::new(0, 10),
      Some(DependencyRange::new(20, 30)),
      ExportsBase::Exports,
      vec![Atom::from("differentName")],
      ExportContext::IndividualAssignment,
    );

    assert_ne!(
      dep1.get_resource_identifier(),
      dep3.get_resource_identifier()
    );
  }
}
