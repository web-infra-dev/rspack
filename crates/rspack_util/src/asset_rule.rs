use rspack_regex::RspackRegex;

#[derive(Debug, Clone, Hash)]
pub enum AssetRule {
  String(String),
  Regexp(RspackRegex),
}

impl AssetRule {
  pub fn try_match(&self, data: &str) -> bool {
    match self {
      Self::String(s) => data.starts_with(s),
      Self::Regexp(r) => r.test(data),
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum AssetRules {
  Single(AssetRule),
  Multiple(Vec<AssetRule>),
}

impl AssetRules {
  pub fn try_match(&self, data: &str) -> bool {
    match self {
      Self::Single(r) => r.try_match(data),
      Self::Multiple(l) => l.into_iter().any(|r| r.try_match(data)),
    }
  }
}
