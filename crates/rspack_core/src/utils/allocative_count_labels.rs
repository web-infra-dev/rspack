// Prefix custom names to distinguish them from built-in module types, including
// an empty custom name. Percent encoding preserves their identity while removing
// folded-path delimiters and whitespace rejected by the report helper.
pub(super) fn custom_module_type_label(name: &str) -> String {
  format!("custom:{}", urlencoding::encode(name))
}
