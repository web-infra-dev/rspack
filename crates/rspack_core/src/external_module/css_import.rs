use std::borrow::Cow;

use rspack_util::json_stringify_str;

use crate::{CssLayer, CssModuleRenderCondition};

fn push_all(out: &mut String, parts: &[&str]) {
  for part in parts {
    out.push_str(part);
  }
}

fn eq_ci(a: &str, b: &str) -> bool {
  a.eq_ignore_ascii_case(b)
}

#[derive(Clone)]
struct MediaType {
  decoded: String,
  raw_output: Option<String>,
}

impl MediaType {
  fn from_ident(ident: ParsedIdent<'_>) -> Self {
    let raw_output = match &ident.decoded {
      Cow::Borrowed(_) => None,
      Cow::Owned(decoded) => (!is_plain_ident(decoded)).then(|| ident.raw.to_owned()),
    };
    Self {
      decoded: ident.decoded.into_owned(),
      raw_output,
    }
  }

  fn from_plain(name: &str) -> Self {
    Self {
      decoded: name.to_string(),
      raw_output: None,
    }
  }

  fn output(&self) -> &str {
    self.raw_output.as_deref().unwrap_or(&self.decoded)
  }

  fn eq_ignore_ascii_case(&self, other: &str) -> bool {
    eq_ci(&self.decoded, other)
  }
}

struct ParsedIdent<'a> {
  decoded: Cow<'a, str>,
  raw: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MediaQualifier {
  None,
  Only,
  Not,
}

#[derive(Clone)]
struct ParsedMediaQuery {
  qualifier: MediaQualifier,
  media_type: Option<MediaType>,
  condition: Option<String>,
}

impl ParsedMediaQuery {
  fn is_negated(&self) -> bool {
    self.qualifier == MediaQualifier::Not
  }

  fn is_unconstrained_all(&self) -> bool {
    self.qualifier == MediaQualifier::None
      && self.condition.is_none()
      && self
        .media_type
        .as_ref()
        .is_none_or(|mt| mt.eq_ignore_ascii_case("all"))
  }
}

fn is_ident_start_byte(b: u8) -> bool {
  b.is_ascii_alphabetic() || b == b'_' || b >= 0x80
}

fn is_ident_continue_byte(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b >= 0x80
}

fn is_valid_escape(bytes: &[u8], idx: usize) -> bool {
  bytes.get(idx) == Some(&b'\\') && !matches!(bytes.get(idx + 1), Some(b'\n' | b'\r' | 0x0C))
}

fn is_plain_ident(s: &str) -> bool {
  let bytes = s.as_bytes();
  let start = match bytes.first() {
    Some(&b'-') => match bytes.get(1) {
      Some(&b) if is_ident_start_byte(b) || b == b'-' => 2,
      _ => return false,
    },
    Some(&b) if is_ident_start_byte(b) => 1,
    _ => return false,
  };
  let mut i = start;
  while i < bytes.len() {
    if !is_ident_continue_byte(bytes[i]) {
      return false;
    }
    i += 1;
  }
  true
}

/// Shared cursor scanner for CSS media query lists, comments, escapes,
/// identifiers, and balanced parenthesized conditions.
struct MediaScanner<'a> {
  src: &'a str,
  pos: usize,
}

impl<'a> MediaScanner<'a> {
  fn new(src: &'a str) -> Self {
    Self { src, pos: 0 }
  }

  fn bytes(&self) -> &'a [u8] {
    self.src.as_bytes()
  }

  fn rest(&self) -> &'a str {
    &self.src[self.pos..]
  }

  fn is_empty(&self) -> bool {
    self.pos >= self.src.len()
  }

  fn peek_byte(&self) -> Option<u8> {
    self.bytes().get(self.pos).copied()
  }

  fn starts_with_comment(&self) -> bool {
    self.bytes().get(self.pos..self.pos + 2) == Some(b"/*".as_slice())
  }

  /// Advances past a `/* ... */` comment. Returns `false` if unclosed.
  fn skip_comment(&mut self) -> bool {
    let bytes = self.bytes();
    let mut i = self.pos + 2;
    while i + 1 < bytes.len() {
      if bytes[i] == b'*' && bytes[i + 1] == b'/' {
        self.pos = i + 2;
        return true;
      }
      i += 1;
    }
    self.pos = bytes.len();
    false
  }

  fn skip_ws_and_comments(&mut self) {
    let bytes = self.bytes();
    loop {
      while self.pos < bytes.len() && bytes[self.pos].is_ascii_whitespace() {
        self.pos += 1;
      }
      if !self.starts_with_comment() || !self.skip_comment() {
        return;
      }
    }
  }

  /// Consumes a CSS escape sequence starting at `self.pos` (`\`).
  fn consume_css_escape(&mut self) -> Option<char> {
    let bytes = self.bytes();
    if !is_valid_escape(bytes, self.pos) {
      return None;
    }
    let after_bs = self.pos + 1;
    let Some(&first) = bytes.get(after_bs) else {
      self.pos = after_bs;
      return Some(char::REPLACEMENT_CHARACTER);
    };
    if first.is_ascii_hexdigit() {
      let mut pos = after_bs;
      let mut value = 0u32;
      let end = (after_bs + 6).min(bytes.len());
      while pos < end {
        let byte = bytes[pos];
        let digit = match byte {
          b'0'..=b'9' => byte - b'0',
          b'a'..=b'f' => byte - b'a' + 10,
          b'A'..=b'F' => byte - b'A' + 10,
          _ => break,
        };
        value = value * 16 + u32::from(digit);
        pos += 1;
      }
      if let Some(&ws) = bytes.get(pos)
        && ws.is_ascii_whitespace()
      {
        pos += 1;
        if ws == b'\r' && bytes.get(pos) == Some(&b'\n') {
          pos += 1;
        }
      }
      self.pos = pos;
      return Some(
        char::from_u32(value)
          .filter(|_| value != 0)
          .unwrap_or(char::REPLACEMENT_CHARACTER),
      );
    }
    let ch = self.src[after_bs..].chars().next()?;
    self.pos = after_bs + ch.len_utf8();
    Some(ch)
  }

  /// Consumes a CSS identifier (decoding CSS escapes if present).
  /// Leaves `self.pos` unchanged and returns `None` if no identifier starts here.
  fn consume_ident(&mut self) -> Option<ParsedIdent<'a>> {
    let bytes = self.bytes();
    let start = self.pos;
    let &first = bytes.get(start)?;
    let valid_start = is_ident_start_byte(first)
      || is_valid_escape(bytes, start)
      || (first == b'-'
        && bytes.get(start + 1).is_some_and(|&second| {
          is_ident_start_byte(second) || second == b'-' || is_valid_escape(bytes, start + 1)
        }));
    if !valid_start {
      return None;
    }

    let mut decoded: Option<String> = None;
    while self.pos < bytes.len() {
      let run_start = self.pos;
      while self.pos < bytes.len() && is_ident_continue_byte(bytes[self.pos]) {
        self.pos += 1;
      }
      if self.pos > run_start {
        if let Some(buf) = &mut decoded {
          buf.push_str(&self.src[run_start..self.pos]);
        }
        continue;
      }
      let before_esc = self.pos;
      let Some(escaped_ch) = self.consume_css_escape() else {
        break;
      };
      if decoded.is_none() {
        decoded = Some(self.src[start..before_esc].to_string());
      }
      if let Some(buf) = &mut decoded {
        buf.push(escaped_ch);
      }
    }

    let raw = &self.src[start..self.pos];
    let decoded = match decoded {
      Some(buf) => Cow::Owned(buf),
      None => Cow::Borrowed(raw),
    };
    Some(ParsedIdent { decoded, raw })
  }

  /// Consumes a leading identifier followed by EOF, ASCII whitespace, or `/*`.
  /// Restores `self.pos` if not matched.
  fn take_leading_ident(&mut self) -> Option<ParsedIdent<'a>> {
    let saved = self.pos;
    let ident = self.consume_ident()?;
    if self.is_empty()
      || self.peek_byte().is_some_and(|b| b.is_ascii_whitespace())
      || self.starts_with_comment()
    {
      Some(ident)
    } else {
      self.pos = saved;
      None
    }
  }

  /// Scans tokens skipping quoted strings, comments, and CSS escapes.
  /// - `split_commas == true`: stops after the next top-level `,` (tracking `()`/`[]`).
  /// - `split_commas == false`: expects `self.pos` at `(` and stops after the matching `)`.
  fn scan_balanced(&mut self, split_commas: bool) -> bool {
    let bytes = self.bytes();
    let mut depth = 0usize;
    let mut quote = 0u8;
    let mut escaped = false;
    while self.pos < bytes.len() {
      let b = bytes[self.pos];
      if quote != 0 {
        self.pos += 1;
        if escaped {
          escaped = false;
        } else if b == b'\\' {
          escaped = true;
        } else if b == quote {
          quote = 0;
        }
        continue;
      }
      if b == b'/' && self.starts_with_comment() {
        if !self.skip_comment() {
          return false;
        }
        continue;
      }
      if b == b'\\' && self.consume_css_escape().is_some() {
        continue;
      }
      self.pos += 1;
      match b {
        b'"' | b'\'' => quote = b,
        b'(' => depth += 1,
        b'[' if split_commas => depth += 1,
        b')' => {
          depth = depth.saturating_sub(1);
          if !split_commas && depth == 0 {
            return true;
          }
        }
        b']' if split_commas => depth = depth.saturating_sub(1),
        b',' if split_commas && depth == 0 => return true,
        _ => {}
      }
    }
    false
  }

  /// Consumes `<media-in-parens>` (`(...)` or `ident(...)`).
  fn consume_media_in_parens(&mut self) -> bool {
    let saved = self.pos;
    if self.peek_byte() != Some(b'(')
      && (self.consume_ident().is_none() || self.peek_byte() != Some(b'('))
    {
      self.pos = saved;
      return false;
    }
    if self.scan_balanced(false) {
      true
    } else {
      self.pos = saved;
      false
    }
  }
}

/// Renders an `@import` rule for an external stylesheet.
///
/// Conditions are ordered from the outermost to the innermost context and are
/// folded into the `@import` prelude: nested layers become a single layer path,
/// supports conditions are combined with `and`, and media query lists are
/// intersected.
#[inline(never)]
pub(super) fn render_css_import_source(
  request: &str,
  conditions: &[CssModuleRenderCondition],
) -> String {
  let mut layer: Option<CssLayer> = None;
  let mut supports: Vec<&str> = Vec::new();
  let mut media: Option<String> = None;

  for condition in conditions {
    if let Some(condition_layer) = &condition.layer {
      layer = Some(match (layer.take(), condition_layer.clone()) {
        (None, layer) => layer,
        (Some(outer), inner) => merge_css_layers(outer, inner),
      });
    }
    if let Some(value) = condition
      .supports
      .as_deref()
      .filter(|value| !value.is_empty())
    {
      supports.push(value);
    }
    if let Some(value) = condition.media.as_deref().filter(|value| !value.is_empty()) {
      media = Some(match media {
        Some(outer) => merge_media_query_lists(&outer, value),
        None => value.to_string(),
      });
    }
  }

  let mut source = String::new();
  let quoted_request = json_stringify_str(request);
  push_all(&mut source, &["@import url(", &quoted_request, ")"]);
  match layer {
    Some(CssLayer::Anonymous) => source.push_str(" layer()"),
    Some(CssLayer::Named(layer)) => push_all(&mut source, &[" layer(", &layer, ")"]),
    None => {}
  }
  if let Some((first, rest)) = supports.split_first() {
    source.push_str(" supports(");
    if rest.is_empty() {
      source.push_str(first);
    } else {
      push_all(&mut source, &["(", first, ")"]);
      for value in rest {
        push_all(&mut source, &[" and (", value, ")"]);
      }
    }
    source.push(')');
  }
  if let Some(media) = media {
    push_all(&mut source, &[" ", &media]);
  }
  source.push(';');
  source
}

/// Merges a nested layer into its outer layer.
///
/// Anonymous layers have no addressable name, so they fall back to the closest
/// representable form: an anonymous outer layer stays anonymous, while an
/// anonymous inner layer keeps the named outer layer.
fn merge_css_layers(outer: CssLayer, inner: CssLayer) -> CssLayer {
  match (outer, inner) {
    (CssLayer::Named(outer), CssLayer::Named(inner)) => {
      let mut merged = String::new();
      push_all(&mut merged, &[&outer, ".", &inner]);
      CssLayer::Named(merged.into())
    }
    (CssLayer::Anonymous, _) => CssLayer::Anonymous,
    (CssLayer::Named(outer), CssLayer::Anonymous) => CssLayer::Named(outer),
  }
}

fn push_comma_item(out: &mut String, item: &str) {
  if !out.is_empty() {
    out.push_str(", ");
  }
  out.push_str(item);
}

fn parse_media_query_list(items: &[&str]) -> Vec<ParsedMediaQuery> {
  let mut parsed = Vec::with_capacity(items.len());
  for item in items {
    if let Some(query) = parse_media_query(item) {
      parsed.push(query);
    }
  }
  parsed
}

/// Intersects two media query lists according to CSS Media Queries Level 4
/// syntax, keeping at most one `[not | only]? <media-type>` at the start of
/// each query.
fn merge_media_query_lists(outer: &str, inner: &str) -> String {
  let outer = split_media_query_list(outer);
  let inner = split_media_query_list(inner);
  if outer.is_empty() || inner.is_empty() {
    let mut out = String::new();
    for item in if outer.is_empty() { &inner } else { &outer } {
      push_comma_item(&mut out, item);
    }
    return out;
  }

  let outer_parsed = parse_media_query_list(&outer);
  let inner_parsed = parse_media_query_list(&inner);

  let mut merged = Vec::new();
  let mut intersected = Vec::new();
  for outer_query in &outer_parsed {
    for inner_query in &inner_parsed {
      intersected.clear();
      push_intersected_queries(outer_query, inner_query, &mut intersected);
      for query in &intersected {
        let formatted = format_media_query(query);
        if !merged.contains(&formatted) {
          merged.push(formatted);
        }
      }
    }
  }

  if merged.is_empty() {
    "not all".to_string()
  } else {
    let mut out = String::new();
    for item in &merged {
      push_comma_item(&mut out, item);
    }
    out
  }
}

fn split_media_query_list(media: &str) -> Vec<&str> {
  let mut queries = Vec::new();
  let mut scanner = MediaScanner::new(media);
  loop {
    let start = scanner.pos;
    let found_comma = scanner.scan_balanced(true);
    let end = if found_comma {
      scanner.pos - 1
    } else {
      media.len()
    };
    let query = media[start..end].trim_ascii();
    if !query.is_empty() {
      queries.push(query);
    }
    if !found_comma {
      break;
    }
  }
  queries
}

fn is_valid_media_condition(s: &str, allow_or: bool, allow_not: bool) -> bool {
  let mut scanner = MediaScanner::new(s);
  scanner.skip_ws_and_comments();
  if let Some(ident) = scanner.take_leading_ident() {
    if !allow_not || !eq_ci(&ident.decoded, "not") {
      return false;
    }
    scanner.skip_ws_and_comments();
    if !scanner.consume_media_in_parens() {
      return false;
    }
    scanner.skip_ws_and_comments();
    return scanner.is_empty();
  }

  if !scanner.consume_media_in_parens() {
    return false;
  }
  let mut is_and_chain: Option<bool> = None;
  loop {
    scanner.skip_ws_and_comments();
    if scanner.is_empty() {
      return true;
    }
    let Some(kw) = scanner.take_leading_ident() else {
      return false;
    };
    if eq_ci(&kw.decoded, "and") {
      if is_and_chain == Some(false) {
        return false;
      }
      is_and_chain = Some(true);
    } else if allow_or && eq_ci(&kw.decoded, "or") {
      if is_and_chain == Some(true) {
        return false;
      }
      is_and_chain = Some(false);
    } else {
      return false;
    }
    scanner.skip_ws_and_comments();
    if !scanner.consume_media_in_parens() {
      return false;
    }
  }
}

fn is_valid_media_type(ident: &str) -> bool {
  for kw in ["only", "not", "and", "or", "layer"] {
    if eq_ci(ident, kw) {
      return false;
    }
  }
  true
}

fn parse_media_query_tail(scanner: &mut MediaScanner<'_>) -> Option<Option<String>> {
  scanner.skip_ws_and_comments();
  if scanner.is_empty() {
    return Some(None);
  }
  let kw = scanner.take_leading_ident()?;
  if !eq_ci(&kw.decoded, "and") {
    return None;
  }
  scanner.skip_ws_and_comments();
  let cond = scanner.rest();
  if cond.is_empty() || !is_valid_media_condition(cond, false, true) {
    return None;
  }
  Some(Some(cond.trim_ascii().to_string()))
}

fn parse_media_query(raw: &str) -> Option<ParsedMediaQuery> {
  let mut scanner = MediaScanner::new(raw);
  scanner.skip_ws_and_comments();
  if scanner.is_empty() {
    return None;
  }

  let cond_start = scanner.pos;
  let Some(first) = scanner.take_leading_ident() else {
    let cond = &raw[cond_start..];
    if is_valid_media_condition(cond, true, true) {
      return Some(ParsedMediaQuery {
        qualifier: MediaQualifier::None,
        media_type: None,
        condition: Some(cond.trim_ascii().to_string()),
      });
    }
    return None;
  };

  let qualifier = if eq_ci(&first.decoded, "not") {
    MediaQualifier::Not
  } else if eq_ci(&first.decoded, "only") {
    MediaQualifier::Only
  } else {
    MediaQualifier::None
  };

  let mt = if qualifier != MediaQualifier::None {
    scanner.skip_ws_and_comments();
    if qualifier == MediaQualifier::Not {
      let saved = scanner.pos;
      let is_paren_cond = scanner.consume_media_in_parens();
      scanner.pos = saved;
      if is_paren_cond {
        let cond = &raw[cond_start..];
        if is_valid_media_condition(cond, true, true) {
          return Some(ParsedMediaQuery {
            qualifier: MediaQualifier::None,
            media_type: None,
            condition: Some(cond.trim_ascii().to_string()),
          });
        }
        return None;
      }
    }
    scanner.take_leading_ident()?
  } else {
    first
  };

  if !is_valid_media_type(&mt.decoded) {
    return None;
  }
  let cond = parse_media_query_tail(&mut scanner)?;
  let media_type = MediaType::from_ident(mt);
  if qualifier == MediaQualifier::Not && media_type.eq_ignore_ascii_case("all") {
    let c = cond?;
    return Some(ParsedMediaQuery {
      qualifier: MediaQualifier::None,
      media_type: None,
      condition: Some(negate_condition(&c)),
    });
  }
  Some(ParsedMediaQuery {
    qualifier,
    media_type: Some(media_type),
    condition: cond,
  })
}

fn push_wrapped_for_and(out: &mut String, condition: &str) {
  if !is_valid_media_condition(condition, false, false) {
    push_all(out, &["(", condition, ")"]);
  } else {
    out.push_str(condition);
  }
}

fn negate_condition(condition: &str) -> String {
  let trimmed = condition.trim_ascii();
  let mut scanner = MediaScanner::new(trimmed);
  scanner.skip_ws_and_comments();
  let single = scanner.consume_media_in_parens() && {
    scanner.skip_ws_and_comments();
    scanner.is_empty()
  };
  let mut out = String::new();
  if single {
    push_all(&mut out, &["not ", trimmed]);
  } else {
    push_all(&mut out, &["not (", trimmed, ")"]);
  }
  out
}

fn combine_conditions(a: Option<&str>, b: Option<&str>) -> Option<String> {
  match (a, b) {
    (None, None) => None,
    (Some(c), None) | (None, Some(c)) => Some(c.to_string()),
    (Some(a), Some(b)) if a == b => Some(a.to_string()),
    (Some(a), Some(b)) => {
      let mut out = String::new();
      push_wrapped_for_and(&mut out, a);
      out.push_str(" and ");
      push_wrapped_for_and(&mut out, b);
      Some(out)
    }
  }
}

fn expand_negated_query(media_type: &MediaType, condition: Option<&str>) -> Vec<ParsedMediaQuery> {
  let mut expanded = Vec::with_capacity(4);
  for known in ["screen", "print", "speech"] {
    if !media_type.eq_ignore_ascii_case(known) {
      expanded.push(ParsedMediaQuery {
        qualifier: MediaQualifier::None,
        media_type: Some(MediaType::from_plain(known)),
        condition: None,
      });
    }
  }
  if let Some(cond) = condition {
    expanded.push(ParsedMediaQuery {
      qualifier: MediaQualifier::None,
      media_type: Some(media_type.clone()),
      condition: Some(negate_condition(cond)),
    });
  }
  expanded
}

fn push_intersected_queries(
  a: &ParsedMediaQuery,
  b: &ParsedMediaQuery,
  out: &mut Vec<ParsedMediaQuery>,
) {
  if a.is_unconstrained_all() {
    out.push(b.clone());
    return;
  }
  if b.is_unconstrained_all() {
    out.push(a.clone());
    return;
  }

  if !a.is_negated() && !b.is_negated() {
    let merged_type = match (a.media_type.as_ref(), b.media_type.as_ref()) {
      (None, Some(t)) | (Some(t), None) => Some(t.clone()),
      (None, None) => None,
      (Some(t1), Some(t2)) if t1.eq_ignore_ascii_case("all") => Some(t2.clone()),
      (Some(t1), Some(t2)) if t2.eq_ignore_ascii_case("all") => Some(t1.clone()),
      (Some(t1), Some(t2)) if t1.eq_ignore_ascii_case(&t2.decoded) => Some(t1.clone()),
      (Some(_), Some(_)) => return,
    };
    let qualifier = if (a.qualifier == MediaQualifier::Only || b.qualifier == MediaQualifier::Only)
      && merged_type.is_some()
    {
      MediaQualifier::Only
    } else {
      MediaQualifier::None
    };
    out.push(ParsedMediaQuery {
      qualifier,
      media_type: merged_type,
      condition: combine_conditions(a.condition.as_deref(), b.condition.as_deref()),
    });
    return;
  }

  if a.is_negated()
    && b.is_negated()
    && a.condition.is_none()
    && b.condition.is_none()
    && let (Some(t1), Some(t2)) = (&a.media_type, &b.media_type)
    && t1.eq_ignore_ascii_case(&t2.decoded)
  {
    out.push(a.clone());
    return;
  }

  let (neg, other, negated_first) = if a.is_negated() {
    (a, b, true)
  } else {
    (b, a, false)
  };
  let Some(nt) = &neg.media_type else { return };

  if !other.is_negated()
    && let Some(pt) = &other.media_type
    && !pt.eq_ignore_ascii_case("all")
  {
    if !nt.eq_ignore_ascii_case(&pt.decoded) {
      out.push(other.clone());
    } else if let Some(nc) = &neg.condition {
      out.push(ParsedMediaQuery {
        qualifier: other.qualifier,
        media_type: Some(pt.clone()),
        condition: combine_conditions(Some(&negate_condition(nc)), other.condition.as_deref()),
      });
    }
    return;
  }

  let expanded = expand_negated_query(nt, neg.condition.as_deref());
  for item in &expanded {
    if negated_first {
      push_intersected_queries(item, other, out);
    } else {
      push_intersected_queries(other, item, out);
    }
  }
}

fn format_media_query(query: &ParsedMediaQuery) -> String {
  let mut out = String::new();
  match query.qualifier {
    MediaQualifier::Only => out.push_str("only "),
    MediaQualifier::Not => out.push_str("not "),
    MediaQualifier::None => {}
  }
  match (&query.media_type, &query.condition) {
    (Some(mt), cond) => {
      out.push_str(mt.output());
      if let Some(cond) = cond {
        out.push_str(" and ");
        push_wrapped_for_and(&mut out, cond);
      }
    }
    (None, Some(cond)) => out.push_str(cond),
    (None, None) => out.push_str("all"),
  }
  out
}
