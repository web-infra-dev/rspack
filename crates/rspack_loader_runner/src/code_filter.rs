use aho_corasick::AhoCorasick;
use rspack_cacheable::{
  ContextGuard, Result as CacheableResult, cacheable,
  with::{Custom, CustomConverter},
};
use rspack_regex::RspackRegex;

use crate::Content;

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeFilterPattern {
  String(String),
  RegExp(RspackRegex),
}

#[derive(Debug)]
enum StringMatcher {
  None,
  Always,
  Single(String),
  Multiple(AhoCorasick),
}

#[derive(Debug)]
struct PatternMatcher {
  patterns: Vec<CodeFilterPattern>,
  string_matcher: StringMatcher,
}

impl PatternMatcher {
  fn new(patterns: Vec<CodeFilterPattern>) -> Self {
    let strings = patterns
      .iter()
      .filter_map(|pattern| match pattern {
        CodeFilterPattern::String(pattern) => Some(pattern.as_str()),
        CodeFilterPattern::RegExp(_) => None,
      })
      .collect::<Vec<_>>();
    let string_matcher = if strings.iter().any(|pattern| pattern.is_empty()) {
      StringMatcher::Always
    } else {
      match strings.as_slice() {
        [] => StringMatcher::None,
        [pattern] => StringMatcher::Single((*pattern).to_owned()),
        patterns => StringMatcher::Multiple(
          AhoCorasick::new(patterns).expect("string code filter patterns should be valid"),
        ),
      }
    };
    Self {
      patterns,
      string_matcher,
    }
  }

  #[inline]
  fn matches(&self, code: &str) -> bool {
    let matches_string = match &self.string_matcher {
      StringMatcher::None => false,
      StringMatcher::Always => true,
      StringMatcher::Single(pattern) => {
        memchr::memmem::find(code.as_bytes(), pattern.as_bytes()).is_some()
      }
      StringMatcher::Multiple(matcher) => matcher.is_match(code.as_bytes()),
    };
    matches_string
      || self.patterns.iter().any(|pattern| match pattern {
        CodeFilterPattern::String(_) => false,
        CodeFilterPattern::RegExp(pattern) => pattern.test(code),
      })
  }
}

#[cacheable]
#[doc(hidden)]
pub struct SerializableCodeFilter {
  include: Option<Vec<CodeFilterPattern>>,
  exclude: Vec<CodeFilterPattern>,
}

#[cacheable(with=Custom)]
#[derive(Debug)]
pub struct CodeFilter {
  include: Option<PatternMatcher>,
  exclude: PatternMatcher,
}

impl CodeFilter {
  pub fn new(include: Option<Vec<CodeFilterPattern>>, exclude: Vec<CodeFilterPattern>) -> Self {
    Self {
      include: include.map(PatternMatcher::new),
      exclude: PatternMatcher::new(exclude),
    }
  }

  pub fn include(&self) -> Option<&[CodeFilterPattern]> {
    self
      .include
      .as_ref()
      .map(|matcher| matcher.patterns.as_slice())
  }

  pub fn exclude(&self) -> &[CodeFilterPattern] {
    &self.exclude.patterns
  }

  pub fn matches(&self, content: Option<&Content>) -> bool {
    let Some(content) = content else {
      return false;
    };
    let code = match content {
      Content::String(code) => code.as_str(),
      Content::Buffer(code) => {
        let Ok(code) = simdutf8::basic::from_utf8(code) else {
          return false;
        };
        code
      }
    };

    if self.exclude.matches(code) {
      return false;
    }
    self
      .include
      .as_ref()
      .is_none_or(|include| include.matches(code))
  }
}

impl CustomConverter for CodeFilter {
  type Target = SerializableCodeFilter;

  fn serialize(&self, _guard: &ContextGuard) -> CacheableResult<Self::Target> {
    Ok(SerializableCodeFilter {
      include: self.include().map(ToOwned::to_owned),
      exclude: self.exclude().to_vec(),
    })
  }

  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> CacheableResult<Self> {
    Ok(Self::new(data.include, data.exclude))
  }
}
