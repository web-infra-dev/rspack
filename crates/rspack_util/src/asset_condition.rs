use rspack_regex::RspackRegex;

#[derive(Debug, Clone, Hash)]
pub enum AssetCondition {
  String(String),
  Regexp(RspackRegex),
}

impl AssetCondition {
  pub fn try_match(&self, data: &str) -> bool {
    match self {
      Self::String(s) => data.starts_with(s),
      Self::Regexp(r) => r.test(data),
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum AssetConditions {
  Single(AssetCondition),
  Multiple(Vec<AssetCondition>),
}

impl AssetConditions {
  pub fn try_match(&self, data: &str) -> bool {
    match self {
      Self::Single(r) => r.try_match(data),
      Self::Multiple(l) => l.iter().any(|r| r.try_match(data)),
    }
  }
}
