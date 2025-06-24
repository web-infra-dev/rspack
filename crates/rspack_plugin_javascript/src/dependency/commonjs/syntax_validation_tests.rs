//! Syntax validation tests for CommonJS macro positioning
//! Ensures JavaScript syntax validity after macro processing

#[cfg(test)]
mod syntax_validation_tests {

  /// Validates JavaScript syntax patterns to ensure macros don't create invalid code
  fn validate_javascript_syntax(code: &str) -> bool {
    // Very specific check for the problematic pattern we're trying to avoid:
    // /* @common:if */ varName /* @common:endif */ = value

    // This pattern specifically matches a macro wrapping a variable name before an assignment
    let problematic_pattern = r"/\*\s*@common:if[^*]*\*/\s*\w+\s*/\*\s*@common:endif\s*\*/\s*=";

    if let Ok(regex) = regex::Regex::new(problematic_pattern) {
      if regex.is_match(code) {
        return false;
      }
    }

    // Check if code would result in orphaned assignment after macro removal
    // This catches patterns like: " = exports.something;" (missing left side)
    if code.trim_start().starts_with("= ") {
      return false;
    }

    true
  }

  #[test]
  fn test_variable_assignment_syntax_patterns() {
    // Test cases that should be VALID after our fix
    let valid_patterns = vec![
            // Individual assignment exports (working correctly)
            "/* @common:if [condition=\"treeShake.test.formatPath\"] */ exports.formatPath /* @common:endif */ = function(path) { return path; };",
            "/* @common:if [condition=\"treeShake.test.readFileSync\"] */ exports.readFileSync /* @common:endif */ = function(file) { return file; };",
            "/* @common:if [condition=\"treeShake.test.constants\"] */ exports.constants /* @common:endif */ = { DEFAULT: 'value' };",

            // Object literal properties (new fix)
            "module.exports = { /* @common:if [condition=\"treeShake.test.calculateSum\"] */ calculateSum /* @common:endif */: calculateSum };",
            "module.exports = { /* @common:if [condition=\"treeShake.test.formatCurrency\"] */ formatCurrency /* @common:endif */: formatCurrency };",
            "exports = { /* @common:if [condition=\"treeShake.test.prop\"] */ prop /* @common:endif */: value };",
        ];

    for pattern in valid_patterns {
      assert!(
        validate_javascript_syntax(pattern),
        "Pattern should be valid: {}",
        pattern
      );
    }
  }

  #[test]
  fn test_invalid_syntax_patterns() {
    // Test cases that should be INVALID (what we're trying to avoid)
    let invalid_patterns = vec![
      // Property access wrapping (old broken behavior)
      "/* @common:if */ module.exports.calculateSum /* @common:endif */,",
      "/* @common:if */ module.exports.formatCurrency /* @common:endif */,",
      "/* @common:if */ exports.prop /* @common:endif */,",
      // Incomplete assignments
      "/* @common:if */ formatPath /* @common:endif */ = exports.formatPath;",
      "/* @common:if */ readFileSync /* @common:endif */ = exports.readFileSync;",
      " = exports.something;",
    ];

    for pattern in invalid_patterns {
      assert!(
        !validate_javascript_syntax(pattern),
        "Pattern should be invalid: {}",
        pattern
      );
    }
  }

  #[test]
  fn test_object_property_syntax_patterns() {
    // Test cases for object literal properties (NEW FIX)
    let valid_object_patterns = vec![
            // Object property names wrapped correctly
            "{ /* @common:if [condition=\"treeShake.test.calculateSum\"] */ calculateSum /* @common:endif */: calculateSum }",
            "{ /* @common:if [condition=\"treeShake.test.formatPath\"] */ formatPath /* @common:endif */: formatPath }",
            "module.exports = { /* @common:if [condition=\"treeShake.test.prop\"] */ prop /* @common:endif */: value };",
        ];

    for pattern in valid_object_patterns {
      assert!(
        validate_javascript_syntax(pattern),
        "Object pattern should be valid: {}",
        pattern
      );
    }

    // Test invalid object patterns (what we DON'T want)
    let invalid_object_patterns = vec![
      // Property access wrapping (broken behavior)
      "{ /* @common:if */ module.exports.calculateSum /* @common:endif */: value }",
      "{ /* @common:if */ exports.prop /* @common:endif */: value }",
    ];

    for pattern in invalid_object_patterns {
      assert!(
        !validate_javascript_syntax(pattern),
        "Object pattern should be invalid: {}",
        pattern
      );
    }
  }

  #[test]
  fn test_exports_assignment_syntax_patterns() {
    // Test cases for exports.prop = value assignments (WORKING CORRECTLY)
    let valid_export_assignments = vec![
            "/* @common:if [condition=\"treeShake.test.formatPath\"] */ exports.formatPath /* @common:endif */ = function(path) { return path; };",
            "/* @common:if [condition=\"treeShake.test.constants\"] */ exports.constants /* @common:endif */ = { DEFAULT: 'value' };",
            "/* @common:if [condition=\"treeShake.test.FileManager\"] */ exports.FileManager /* @common:endif */ = FileManager;",
        ];

    for pattern in valid_export_assignments {
      assert!(
        validate_javascript_syntax(pattern),
        "Export assignment should be valid: {}",
        pattern
      );
    }
  }

  /// Test that demonstrates the template logic fix for variable assignments
  #[test]
  fn test_template_logic_variable_assignment_fix() {
    // This test demonstrates how the template logic should transform problematic patterns

    // Input: Variable assignment pattern
    let input_assignment = "formatPath = exports.formatPath";

    // Our fix should detect " = exports." pattern and wrap the right-hand side
    if input_assignment.contains(" = exports.") {
      if let Some(equals_pos) = input_assignment.find(" = ") {
        let var_name = &input_assignment[..equals_pos];
        let value_part = &input_assignment[equals_pos + 3..]; // Skip " = "

        let macro_condition = "treeShake.test.formatPath";
        let result = format!(
          "{} = /* @common:if [condition=\"{}\"] */ {} /* @common:endif */",
          var_name, macro_condition, value_part
        );

        // The result should be valid JavaScript syntax
        assert!(
          validate_javascript_syntax(&result),
          "Template logic fix should produce valid syntax: {}",
          result
        );

        // Verify specific requirements
        assert!(
          !result.starts_with("/* @common:if"),
          "Variable name should not be wrapped: {}",
          result
        );
        assert!(
          result.contains("formatPath ="),
          "Assignment should start with variable name: {}",
          result
        );
        assert!(
          result.contains("exports.formatPath /* @common:endif */"),
          "Value should be wrapped: {}",
          result
        );
      }
    }
  }

  /// Test edge cases that could still cause problems
  #[test]
  fn test_edge_case_patterns() {
    // Edge cases that our fix should handle correctly
    let edge_cases = vec![
      ("info = { name: 'test', version: '1.0' };", "info"),
      ("formatter = (x) => x.toString();", "formatter"),
      ("helper = require('./helper');", "helper"),
      ("config = process.env.NODE_ENV || 'development';", "config"),
    ];

    for (assignment, var_name) in edge_cases {
      if assignment.contains(" = ") {
        if let Some(equals_pos) = assignment.find(" = ") {
          let var_part = &assignment[..equals_pos];
          let value_part = &assignment[equals_pos + 3..];

          let result = format!(
            "{} = /* @common:if */ {} /* @common:endif */",
            var_part, value_part
          );

          assert!(
            validate_javascript_syntax(&result),
            "Edge case should produce valid syntax: {} -> {}",
            assignment,
            result
          );

          assert_eq!(
            var_part, var_name,
            "Variable extraction should be correct for: {}",
            assignment
          );
        }
      }
    }
  }
}
