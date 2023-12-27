use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_regex::RspackRegex;
use rspack_util::try_any_sync;

#[derive(Debug)]
pub enum Rule {
  String(String),
  Regexp(RspackRegex),
}

impl Rule {
  pub fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

#[derive(Debug)]
pub enum Rules {
  Single(Rule),
  Array(Vec<Rule>),
}

impl Rules {
  #[async_recursion]
  pub async fn try_match(&self, data: &str) -> Result<bool> {
    match self {
      Self::Single(s) => s.try_match(data),
      Self::Array(l) => try_any_sync(l, |i| i.try_match(data)),
    }
  }
}
#[derive(Debug)]
pub struct MatchObject {
  pub test: Option<Rules>,
  pub include: Option<Rules>,
  pub exclude: Option<Rules>,
}

#[async_recursion]
pub async fn match_object(match_object: &MatchObject, text: &str) -> Result<bool> {
  let MatchObject {
    test,
    include,
    exclude,
  } = match_object;

  if let Some(condition) = &test {
    if !condition.try_match(text).await? {
      return Ok(false);
    }
  }
  if let Some(condition) = &include {
    if !condition.try_match(text).await? {
      return Ok(false);
    }
  }
  if let Some(condition) = &exclude {
    if condition.try_match(text).await? {
      return Ok(false);
    }
  }
  Ok(true)
}
