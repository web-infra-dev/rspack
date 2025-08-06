use itertools::Itertools;
use rspack_error::miette::Diagnostic;
use rspack_regex::RspackRegex;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::{
  SourceFile, Span,
  comments::{Comment, CommentKind, Comments},
};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebpackComment {
  ChunkName,
  Prefetch,
  Preload,
  Ignore,
  FetchPriority,
  IncludeRegexp,
  IncludeFlags,
  ExcludeRegexp,
  ExcludeFlags,
  Mode,
  Exports,
}

#[derive(Debug)]
pub struct WebpackCommentMap(FxHashMap<WebpackComment, String>);

impl WebpackCommentMap {
  fn new() -> Self {
    Self(Default::default())
  }

  fn insert(&mut self, key: WebpackComment, value: String) {
    self.0.insert(key, value);
  }

  pub fn get_webpack_mode(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Mode)
  }

  pub fn get_webpack_chunk_name(&self) -> Option<&String> {
    self.0.get(&WebpackComment::ChunkName)
  }

  pub fn get_webpack_prefetch(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Prefetch)
  }

  pub fn get_webpack_preload(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Preload)
  }

  pub fn get_webpack_ignore(&self) -> Option<bool> {
    self.0.get(&WebpackComment::Ignore).and_then(|item| {
      if item == "true" {
        Some(true)
      } else if item == "false" {
        Some(false)
      } else {
        None
      }
    })
  }

  pub fn get_fetch_priority(&self) -> Option<&String> {
    self.0.get(&WebpackComment::FetchPriority)
  }

  pub fn get_webpack_include(&self) -> Option<RspackRegex> {
    self.0.get(&WebpackComment::IncludeRegexp).map(|expr| {
      let flags = self
        .0
        .get(&WebpackComment::IncludeFlags)
        .map(|x| x.as_str())
        .unwrap_or_default();

      RspackRegex::with_flags(expr, flags).unwrap_or_else(|_| {
        // test when capture
        unreachable!();
      })
    })
  }

  pub fn get_webpack_exclude(&self) -> Option<RspackRegex> {
    self.0.get(&WebpackComment::ExcludeRegexp).map(|expr| {
      let flags = self
        .0
        .get(&WebpackComment::ExcludeFlags)
        .map(|x| x.as_str())
        .unwrap_or_default();

      RspackRegex::with_flags(expr, flags).unwrap_or_else(|_| {
        // test when capture
        unreachable!();
      })
    })
  }

  pub fn get_webpack_exports(&self) -> Option<Vec<String>> {
    self.0.get(&WebpackComment::Exports).map(|expr| {
      expr
        .split(',')
        .filter_map(|x| {
          if x.is_empty() {
            None
          } else {
            Some(x.to_owned())
          }
        })
        .collect_vec()
    })
  }
}


// Simple manual parsers for webpack magic comments (cleaner than regex)
#[derive(Debug, Clone)]
struct WebpackMagicComment {
  name: String,
  value: String,
}

// Parse quoted string like "value", 'value', or `value`
fn parse_quoted_string(input: &str) -> Option<(&str, &str)> {
  let first_char = input.chars().next()?;
  if !matches!(first_char, '"' | '\'' | '`') {
    return None;
  }
  
  let remaining = &input[1..];
  if let Some(end_pos) = remaining.find(first_char) {
    let content = &remaining[..end_pos];
    let after = &remaining[end_pos + 1..];
    Some((content, after))
  } else {
    None
  }
}

// Parse number like 123 or 123.456 or -123.456
fn parse_number(input: &str) -> Option<(&str, &str)> {
  let mut end_pos = 0;
  let mut chars = input.chars();
  
  // Handle optional minus sign
  if let Some('-') = chars.next() {
    end_pos += 1;
  } else {
    // Reset if we consumed the first char but it wasn't minus
    chars = input.chars();
  }
  
  // Parse digits
  let mut found_digit = false;
  for ch in chars {
    if ch.is_ascii_digit() {
      end_pos += 1;
      found_digit = true;
    } else if ch == '.' && found_digit {
      end_pos += 1;
      // Parse fractional part
      for frac_ch in input[end_pos..].chars() {
        if frac_ch.is_ascii_digit() {
          end_pos += 1;
        } else {
          break;
        }
      }
      break;
    } else {
      break;
    }
  }
  
  if found_digit && end_pos > 0 {
    Some((&input[..end_pos], &input[end_pos..]))
  } else {
    None
  }
}

// Parse boolean like true or false
fn parse_boolean(input: &str) -> Option<(&str, &str)> {
  if input.starts_with("true") {
    Some(("true", &input[4..]))
  } else if input.starts_with("false") {
    Some(("false", &input[5..]))
  } else {
    None
  }
}

// Parse webpack comment name like webpackChunkName
fn parse_webpack_name(input: &str) -> Option<(&str, &str)> {
  let name_end = input.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-')).unwrap_or(input.len());
  if name_end > 0 {
    let name = &input[..name_end];
    if name.starts_with("webpack") {
      Some((name, &input[name_end..]))
    } else {
      None
    }
  } else {
    None
  }
}

// Parse a complete webpack magic comment like webpackChunkName: "value"
fn parse_magic_comment(input: &str) -> Option<(WebpackMagicComment, &str)> {
  let (name, remaining) = parse_webpack_name(input)?;
  
  // Skip whitespace
  let remaining = remaining.trim_start();
  
  // Expect colon
  if !remaining.starts_with(':') {
    return None;
  }
  let remaining = &remaining[1..];
  
  // Skip whitespace
  let remaining = remaining.trim_start();
  
  let (value, remaining) = if let Some((v, r)) = parse_quoted_string(remaining) {
    (v, r)
  } else if let Some((v, r)) = parse_number(remaining) {
    (v, r)
  } else if let Some((v, r)) = parse_boolean(remaining) {
    (v, r)
  } else {
    return None;
  };
  
  Some((
    WebpackMagicComment {
      name: name.to_string(),
      value: value.to_string(),
    },
    remaining,
  ))
}

// Extract webpack magic comments from a comment text
fn extract_webpack_comments(comment_text: &str) -> Vec<WebpackMagicComment> {
  let mut results = Vec::new();
  let mut remaining = comment_text;
  
  while !remaining.is_empty() {
    remaining = remaining.trim_start();
    if let Some((comment, new_remaining)) = parse_magic_comment(remaining) {
      results.push(comment);
      remaining = new_remaining.trim_start();
      // Skip optional comma
      if remaining.starts_with(',') {
        remaining = &remaining[1..];
      }
    } else {
      // Skip one character and try again
      if !remaining.is_empty() {
        remaining = &remaining[1..];
      }
    }
  }
  
  results
}



pub fn try_extract_webpack_magic_comment(
  source_file: &SourceFile,
  comments: &Option<&dyn Comments>,
  error_span: Span,
  span: Span,
  warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
) -> WebpackCommentMap {
  let mut result = WebpackCommentMap::new();
  comments.with_leading(span.lo, |comments| {
    analyze_comments(
      source_file,
      comments,
      error_span,
      warning_diagnostics,
      &mut result,
    )
  });
  comments.with_trailing(span.hi, |comments| {
    analyze_comments(
      source_file,
      comments,
      error_span,
      warning_diagnostics,
      &mut result,
    )
  });
  result
}



fn analyze_comments(
  _source_file: &SourceFile,
  comments: &[Comment],
  _error_span: Span,
  _warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  result: &mut WebpackCommentMap,
) {
  // TODO: remove this, parser.comments contains two same block comment
  let mut parsed_comment = FxHashSet::<Span>::default();
  for comment in comments
    .iter()
    .rev()
    .filter(|c| matches!(c.kind, CommentKind::Block))
  {
    if parsed_comment.contains(&comment.span) {
      continue;
    }
    parsed_comment.insert(comment.span);
    
    let webpack_comments = extract_webpack_comments(&comment.text);
    
    for webpack_comment in webpack_comments {
      let item_name = &webpack_comment.name;
      let item_value = &webpack_comment.value;
      
      match item_name.as_str() {
        "webpackChunkName" => {
          result.insert(WebpackComment::ChunkName, item_value.clone());
        }
        "webpackPrefetch" => {
          result.insert(WebpackComment::Prefetch, item_value.clone());
        }
        "webpackPreload" => {
          result.insert(WebpackComment::Preload, item_value.clone());
        }
        "webpackIgnore" => {
          result.insert(WebpackComment::Ignore, item_value.clone());
        }
        "webpackMode" => {
          result.insert(WebpackComment::Mode, item_value.clone());
        }
        "webpackFetchPriority" => {
          if item_value == "low" || item_value == "high" || item_value == "auto" {
            result.insert(WebpackComment::FetchPriority, item_value.clone());
          }
        }
        "webpackExports" => {
          result.insert(WebpackComment::Exports, item_value.clone());
        }
        _ => {}
      }
    }
  }
}

#[cfg(test)]
mod tests_extract_regex {
  use super::*;

  #[test]
  fn test_webpack_comment_parser() {
    // Test basic string parsing
    let comments = extract_webpack_comments(r#"webpackChunkName: "test""#);
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].name, "webpackChunkName");
    assert_eq!(comments[0].value, "test");
    
    // Test number parsing
    let comments = extract_webpack_comments("webpackPrefetch: 123");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].name, "webpackPrefetch");
    assert_eq!(comments[0].value, "123");
    
    // Test boolean parsing
    let comments = extract_webpack_comments("webpackIgnore: true");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].name, "webpackIgnore");
    assert_eq!(comments[0].value, "true");
    
    // Test multiple comments
    let comments = extract_webpack_comments(r#"webpackChunkName: "test", webpackPrefetch: true"#);
    assert_eq!(comments.len(), 2);
  }
}
