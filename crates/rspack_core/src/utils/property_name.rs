use std::borrow::Cow;

use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashSet as HashSet;

pub fn is_safe_identifier(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }
  
  let mut chars = s.chars();
  let first = chars.next().unwrap();
  
  // First character must be _, letter, or $
  if !matches!(first, '_' | 'a'..='z' | 'A'..='Z' | '$') {
    return false;
  }
  
  // Remaining characters must be _, letter, digit, or $
  chars.all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '$'))
}

pub static RESERVED_IDENTIFIER: std::sync::LazyLock<HashSet<&str>> = std::sync::LazyLock::new(|| {
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
  if is_safe_identifier(prop) && !RESERVED_IDENTIFIER.contains(prop) {
    Ok(Cow::from(prop))
  } else {
    serde_json::to_string(prop)
      .to_rspack_result()
      .map(Cow::from)
  }
}
