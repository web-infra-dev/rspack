use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{error, Result};
use rustc_hash::FxHashSet as HashSet;

pub static SAFE_IDENTIFIER: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[_a-zA-Z$][_a-zA-Z$0-9]*$").expect("Invalid regexp"));
pub static RESERVED_IDENTIFIER: Lazy<HashSet<&str>> = Lazy::new(|| {
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

pub fn property_name(prop: &str) -> Result<Cow<str>> {
  if SAFE_IDENTIFIER.is_match(prop) && !RESERVED_IDENTIFIER.contains(prop) {
    Ok(Cow::from(prop))
  } else {
    serde_json::to_string(prop)
      .map_err(|e| error!(e.to_string()))
      .map(Cow::from)
  }
}
