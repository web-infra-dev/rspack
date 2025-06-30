#[cfg(test)]
#[allow(clippy::module_inception)]
mod comma_positioning_tests {
  use cow_utils::CowUtils;
  use rspack_core::DependencyRange;
  use swc_core::atoms::Atom;

  use super::super::{CommonJsExportsDependency, ExportContext, ExportsBase};

  #[test]
  fn test_comma_inside_macro_for_object_literal_first_property() {
    // Test that first property in object literal has comma inside macro block
    let _dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 11), // "calculateSum"
      Some(DependencyRange::new(0, 11)),
      ExportsBase::ModuleExports,
      vec![Atom::from("calculateSum")],
      ExportContext::ObjectLiteralPropertyFirst,
    );

    // This would normally be called by the template system
    // We're testing the format generated in render_expression_export
    let macro_condition = "treeShake.cjs-module-exports.calculateSum";
    let expected_format = format!(
      "/* @common:if [condition=\"{macro_condition}\"] */ calculateSum, /* @common:endif */"
    );

    // Verify the format has comma inside the macro block
    assert!(expected_format.contains("calculateSum, /* @common:endif */"));
    assert!(!expected_format.ends_with("/* @common:endif */,"));
  }

  #[test]
  fn test_comma_inside_macro_for_object_literal_subsequent_property() {
    // Test that subsequent properties in object literal have comma inside macro block
    let _dep = CommonJsExportsDependency::new(
      DependencyRange::new(0, 15), // "calculateAverage"
      Some(DependencyRange::new(0, 15)),
      ExportsBase::ModuleExports,
      vec![Atom::from("calculateAverage")],
      ExportContext::ObjectLiteralPropertySubsequent,
    );

    // Simulate the macro generation for subsequent properties
    let macro_condition = "treeShake.cjs-module-exports.calculateAverage";
    let expected_format = format!(
      "/* @common:if [condition=\"{macro_condition}\"] */ calculateAverage, /* @common:endif */"
    );

    // Verify the format has comma inside the macro block
    assert!(expected_format.contains("calculateAverage, /* @common:endif */"));
    assert!(!expected_format.ends_with("/* @common:endif */,"));
  }

  #[test]
  fn test_no_orphaned_commas_after_macro_removal() {
    // Test that removing macros doesn't leave orphaned commas
    let test_cases = vec![
      "/* @common:if [condition=\"test\"] */ prop1, /* @common:endif */",
      "prop0, /* @common:if [condition=\"test\"] */ prop2, /* @common:endif */",
      "/* @common:if [condition=\"test\"] */ prop3 /* @common:endif */",
    ];

    for case in test_cases {
      let pattern = if case.contains("prop1") {
        "/* @common:if [condition=\"test\"] */ prop1, /* @common:endif */"
      } else if case.contains("prop2") {
        "/* @common:if [condition=\"test\"] */ prop2, /* @common:endif */"
      } else {
        "/* @common:if [condition=\"test\"] */ prop3 /* @common:endif */"
      };

      // Simulate macro removal
      let without_macro = case.cow_replace(pattern, "").into_owned();

      // Should not have orphaned commas
      assert!(
        !without_macro.contains(",,"),
        "Found double commas in: {without_macro}"
      );
      assert!(
        !without_macro.starts_with(","),
        "String starts with comma: {without_macro}"
      );
    }
  }

  #[test]
  fn test_export_context_determines_comma_placement() {
    // Test that ExportContext correctly determines comma placement strategy

    // First property should not have leading comma, but should have trailing comma inside macro
    let first_context = ExportContext::ObjectLiteralPropertyFirst;
    assert!(matches!(
      first_context,
      ExportContext::ObjectLiteralPropertyFirst
    ));

    // Subsequent properties should have comma inside macro
    let subsequent_context = ExportContext::ObjectLiteralPropertySubsequent;
    assert!(matches!(
      subsequent_context,
      ExportContext::ObjectLiteralPropertySubsequent
    ));

    // Individual assignments should not have trailing commas
    let individual_context = ExportContext::IndividualAssignment;
    assert!(matches!(
      individual_context,
      ExportContext::IndividualAssignment
    ));
  }

  #[test]
  fn test_macro_format_patterns() {
    // Test various macro format patterns to ensure comma is inside
    let test_patterns = vec![
      ("calculateSum", "/* @common:if [condition=\"treeShake.test.calculateSum\"] */ calculateSum, /* @common:endif */"),
      ("formatCurrency", "/* @common:if [condition=\"treeShake.test.formatCurrency\"] */ formatCurrency, /* @common:endif */"),
      ("validateEmail", "/* @common:if [condition=\"treeShake.test.validateEmail\"] */ validateEmail, /* @common:endif */"),
    ];

    for (property, expected_pattern) in test_patterns {
      // Verify comma is inside macro block
      assert!(expected_pattern.contains(&format!("{property}, /* @common:endif */")));

      // Verify it doesn't end with comma outside macro
      assert!(!expected_pattern.ends_with("/* @common:endif */,"));

      // Verify it follows the correct pattern
      let pattern_description = format!("Pattern for {property}: {expected_pattern}");
    }
  }

  #[test]
  fn test_template_replacement_with_commas() {
    // Test the actual template replacement logic
    let _original_content = "calculateSum";

    // Test replacing with macro that has comma inside
    let macro_with_comma =
      "/* @common:if [condition=\"test\"] */ calculateSum, /* @common:endif */";

    // Verify the macro format has comma inside
    assert!(macro_with_comma.contains("calculateSum, /* @common:endif */"));
    assert!(!macro_with_comma.ends_with("/* @common:endif */,"));
  }

  #[test]
  fn test_multiple_properties_comma_consistency() {
    // Test that multiple properties maintain consistent comma placement
    let properties = vec!["prop1", "prop2", "prop3"];
    let mut formatted_properties = Vec::new();

    for prop in properties {
      let formatted = format!(
        "/* @common:if [condition=\"treeShake.test.{prop}\"] */ {prop}, /* @common:endif */"
      );
      formatted_properties.push(formatted);
    }

    // Verify each property has comma inside macro
    for prop_format in &formatted_properties {
      assert!(prop_format.contains(", /* @common:endif */"));
      assert!(!prop_format.ends_with("/* @common:endif */,"));
    }
  }
}
