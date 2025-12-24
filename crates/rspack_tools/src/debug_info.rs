use itertools::Itertools;

/// DebugInfo tracks the hierarchical context during comparison
#[derive(Debug, Clone, Default)]
pub struct DebugInfo(Vec<String>);

impl std::fmt::Display for DebugInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let body = self.0.iter().map(|item| format!("  {}", item)).join("\n");
    write!(f, "DebugInfo: \n{}", body)
  }
}

impl DebugInfo {
  /// Create a new DebugInfo with an additional field
  pub fn with_field(&self, field_name: &str, value: &str) -> Self {
    let mut new_info = self.clone();
    new_info.0.push(format!("{}: {}", field_name, value));
    new_info
  }
}
