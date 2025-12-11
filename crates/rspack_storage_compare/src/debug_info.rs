use itertools::Itertools;

/// CompareTrace tracks the hierarchical context during comparison
#[derive(Debug, Clone, Default)]
pub struct DebugInfo(Vec<String>);

impl DebugInfo {
  /// Create a new DebugInfo with an additional field
  pub fn with_field(&self, field_name: &str, value: &str) -> Self {
    let mut new_info = self.clone();
    new_info.0.push(format!("{}: {}", field_name, value));
    new_info
  }

  pub fn to_string(&self) -> String {
    let body = self.0.iter().map(|item| format!("  {}", item)).join("\n");
    format!("DebugInfo: \n{}", body)
  }
}
