use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashSet as HashSet;

pub static SAFE_IDENTIFIER: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[_a-zA-Z$][_a-zA-Z$0-9]*$").expect("Invalid regexp"));
pub static RESERVED_IDENTIFIER: LazyLock<HashSet<&str>> = LazyLock::new(|| {
  HashSet::from_iter([
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "enum",
    // strict mode
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    // module code
    "await",
    // skip future reserved keywords defined under ES1 till ES3
    // additional
    "null",
    "true",
    "false",
  ])
});

pub fn property_name(prop: &str) -> Result<Cow<'_, str>> {
  if SAFE_IDENTIFIER.is_match(prop) && !RESERVED_IDENTIFIER.contains(prop) {
    Ok(Cow::from(prop))
  } else {
    serde_json::to_string(prop)
      .to_rspack_result()
      .map(Cow::from)
  }
}

/// Returns the name quoted for ESM export/import specifiers.
/// Unlike `property_name`, this function allows reserved words like "default"
/// since they are valid in export/import context (e.g., `export { foo as default }`).
/// Only quotes names that are not valid JavaScript identifiers (e.g., "a.b" -> "\"a.b\"").
pub fn export_name(name: &str) -> Result<Cow<'_, str>> {
  if SAFE_IDENTIFIER.is_match(name) {
    Ok(Cow::from(name))
  } else {
    serde_json::to_string(name)
      .to_rspack_result()
      .map(Cow::from)
  }
}
