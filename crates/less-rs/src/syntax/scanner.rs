//! Port of `parser/parser-input.js`: a byte scanner with save/restore
//! backtracking, comment absorption and furthest-error tracking.

use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PendingComment<'a> {
  pub index: usize,
  pub text: &'a str,
  pub is_line_comment: bool,
}

pub struct EndInfo {
  pub is_finished: bool,
  pub furthest: usize,
  pub furthest_possible_error_message: Option<String>,
  pub furthest_reached_end: bool,
  pub furthest_char: Option<u8>,
}

/// Result of `$parseUntil`.
pub enum ParseUntil<'a> {
  /// The matched groups: plain text and quoted strings `(quote, text)`.
  Groups(Vec<UntilGroup<'a>>, Option<usize>),
  /// An unbalanced / unexpected character.
  Unexpected(String),
}

pub enum UntilGroup<'a> {
  Text(&'a str),
  Quoted(char, &'a str),
}

pub struct Scanner<'a> {
  pub input: &'a str,
  pub bytes: &'a [u8],
  pub i: usize,
  save_stack: Vec<usize>,
  furthest: usize,
  furthest_msg: Option<String>,
  pub comment_store: VecDeque<PendingComment<'a>>,
  /// Comments ever stored (backtracking re-stores them): part of the
  /// parser's step budget.
  pub comments_stored: u64,
  pub auto_comment_absorb: bool,
  pub finished: bool,
  /// Set when `restore` ran on an empty save stack.
  pub fault: bool,
}

#[inline]
fn is_ws(b: u8) -> bool {
  b == b' ' || b == b'\n' || b == b'\t' || b == b'\r'
}

impl<'a> Scanner<'a> {
  pub fn new(input: &'a str) -> Scanner<'a> {
    let mut s = Scanner {
      input,
      bytes: input.as_bytes(),
      i: 0,
      save_stack: Vec::new(),
      furthest: 0,
      furthest_msg: None,
      comment_store: VecDeque::new(),
      comments_stored: 0,
      auto_comment_absorb: true,
      finished: false,
      fault: false,
    };
    s.skip_whitespace(0);
    s
  }

  #[inline]
  pub fn rest(&self) -> &'a str {
    if self.i >= self.input.len() {
      ""
    } else {
      &self.input[self.i..]
    }
  }

  #[inline]
  pub fn rest_bytes(&self) -> &'a [u8] {
    if self.i >= self.bytes.len() {
      &[]
    } else {
      &self.bytes[self.i..]
    }
  }

  /// Advances by `length` and skips whitespace (and comments when
  /// `auto_comment_absorb` is on). Returns whether anything moved.
  #[inline]
  pub fn skip_whitespace(&mut self, length: usize) -> bool {
    // fast path: no whitespace or comment follows
    let i = self.i + length;
    if let Some(&c) = self.bytes.get(i)
      && !is_ws(c)
      && c != b'/'
    {
      self.i = i;
      return length != 0;
    }
    self.skip_whitespace_slow(length)
  }

  fn skip_whitespace_slow(&mut self, length: usize) -> bool {
    let oldi = self.i;
    let end = self.bytes.len();
    self.i += length;
    while self.i < end {
      let c = self.bytes[self.i];
      if self.auto_comment_absorb && c == b'/' {
        let next = self.bytes.get(self.i + 1).copied();
        if next == Some(b'/') {
          let start = self.i;
          let nl = memchr(b'\n', &self.bytes[self.i + 2..])
            .map(|p| p + self.i + 2)
            .unwrap_or(end);
          self.i = nl;
          self.comments_stored += 1;
          self.comment_store.push_back(PendingComment {
            index: start,
            text: &self.input[start..nl],
            is_line_comment: true,
          });
          continue;
        } else if next == Some(b'*')
          && let Some(p) = find_str(&self.bytes[self.i + 2..], b"*/")
        {
          let start = self.i;
          let stop = self.i + 2 + p + 2;
          self.comments_stored += 1;
          self.comment_store.push_back(PendingComment {
            index: start,
            text: &self.input[start..stop],
            is_line_comment: false,
          });
          self.i = stop;
          continue;
        }
        break;
      }
      if !is_ws(c) {
        break;
      }
      self.i += 1;
    }
    if self.i >= end {
      self.finished = true;
    }
    oldi != self.i
  }

  pub fn save(&mut self) {
    self.save_stack.push(self.i);
  }

  pub fn restore(&mut self) {
    self.restore_msg(None);
  }

  pub fn restore_msg(&mut self, msg: Option<&str>) {
    if self.i > self.furthest
      || (self.i == self.furthest && msg.is_some() && self.furthest_msg.is_none())
    {
      self.furthest = self.i;
      self.furthest_msg = msg.map(|s| s.to_string());
    }
    match self.save_stack.pop() {
      // Note: like less.js, `finished` is sticky and not reset here.
      Some(i) => self.i = i,
      // less.js: `saveStack.pop().current` on an empty stack throws a
      // TypeError; the parser reports it at the next check.
      None => self.fault = true,
    }
  }

  pub fn forget(&mut self) {
    self.save_stack.pop();
  }

  pub fn is_whitespace(&self, offset: isize) -> bool {
    let p = self.i as isize + offset;
    if p < 0 {
      return false;
    }
    self
      .bytes
      .get(p as usize)
      .map(|&b| is_ws(b))
      .unwrap_or(false)
  }

  /// `$re`-style consumption of `len` bytes that a matcher accepted.
  #[inline]
  pub fn consume(&mut self, len: usize) -> &'a str {
    let s = &self.input[self.i..self.i + len];
    self.skip_whitespace(len);
    s
  }

  /// Runs a matcher on the remaining input and consumes the match.
  #[inline]
  pub fn re(&mut self, m: impl FnOnce(&'a [u8]) -> Option<usize>) -> Option<&'a str> {
    let rest = self.rest_bytes();
    let len = m(rest)?;
    Some(self.consume(len))
  }

  #[inline]
  pub fn char(&mut self, c: u8) -> bool {
    if self.bytes.get(self.i) != Some(&c) {
      return false;
    }
    self.skip_whitespace(1);
    true
  }

  #[inline]
  pub fn peek_char(&self, c: u8) -> bool {
    self.bytes.get(self.i) == Some(&c)
  }

  pub fn str(&mut self, tok: &str) -> bool {
    if !self.rest_bytes().starts_with(tok.as_bytes()) {
      return false;
    }
    self.skip_whitespace(tok.len());
    true
  }

  /// `currentChar()`; `0` stands for the empty string at the end.
  #[inline]
  pub fn cur(&self) -> u8 {
    self.bytes.get(self.i).copied().unwrap_or(0)
  }

  #[inline]
  pub fn prev(&self) -> u8 {
    if self.i == 0 {
      0
    } else {
      self.bytes.get(self.i - 1).copied().unwrap_or(0)
    }
  }

  pub fn cur_str(&self) -> &'a str {
    if self.i >= self.input.len() {
      return "";
    }
    let mut end = self.i + 1;
    while end < self.input.len() && !self.input.is_char_boundary(end) {
      end += 1;
    }
    &self.input[self.i..end]
  }

  /// `$quoted(loc)`: returns `(quote char, full quoted text)`.
  pub fn quoted_at(&self, pos: usize) -> Option<(u8, &'a str)> {
    let start = *self.bytes.get(pos)?;
    if start != b'\'' && start != b'"' {
      return None;
    }
    let len = self.bytes.len();
    let mut i = 1;
    while i + pos < len {
      let c = self.bytes[i + pos];
      match c {
        b'\\' => {
          i += 2;
          continue;
        }
        b'\r' | b'\n' => {}
        _ if c == start => {
          return Some((start, &self.input[pos..pos + i + 1]));
        }
        _ => {}
      }
      i += 1;
    }
    None
  }

  /// `$quoted()` at the current position, consuming it.
  pub fn quoted(&mut self) -> Option<&'a str> {
    let (_, s) = self.quoted_at(self.i)?;
    self.skip_whitespace(s.len());
    Some(s)
  }

  pub fn peek_not_numeric(&self) -> bool {
    let c = self.cur();
    if self.i >= self.bytes.len() {
      // charCodeAt past the end is NaN: both comparisons are false.
      return false;
    }
    !(b'+'..=b'9').contains(&c) || c == b'/' || c == b','
  }

  /// `$parseUntil(tok, detectBareVar)`.
  pub fn parse_until(
    &mut self,
    tok: impl Fn(u8) -> bool,
    detect_bare_var: bool,
  ) -> Option<ParseUntil<'a>> {
    let mut in_comment = false;
    let mut block_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut bare_var_index: Option<usize> = None;
    let mut block_stack: Vec<u8> = Vec::new();
    let mut groups: Vec<UntilGroup<'a>> = Vec::new();
    let length = self.bytes.len();
    let start_pos = self.i;
    let mut last_pos = self.i;
    let mut i = self.i;
    loop {
      let next = self.bytes.get(i).copied();
      let next_c = next.unwrap_or(0);
      if block_depth == 0 && next.is_some() && tok(next_c) {
        let v = &self.input[last_pos..i];
        if !v.is_empty() {
          groups.push(UntilGroup::Text(v));
        } else {
          groups.push(UntilGroup::Text(" "));
        }
        self.skip_whitespace(i - start_pos);
        return Some(ParseUntil::Groups(groups, bare_var_index));
      }
      if in_comment {
        if next_c == b'*' && self.bytes.get(i + 1) == Some(&b'/') {
          i += 1;
          block_depth -= 1;
          in_comment = false;
        }
        i += 1;
        // Divergence: less.js loops forever on an unterminated `/*`
        // here; stop at the end of input instead.
        if i > length {
          return None;
        }
        continue;
      }
      if detect_bare_var
        && bare_var_index.is_none()
        && next_c == b'@'
        && paren_depth == 0
        && let Some(&a) = self.bytes.get(i + 1)
        && (a == b'-' || a == b'_' || a.is_ascii_alphanumeric())
      {
        bare_var_index = Some(i);
      }
      match next_c {
        b'\\' => {
          i += 1;
          let end = (i + 1).min(length);
          let end = ceil_boundary(self.input, end);
          groups.push(UntilGroup::Text(&self.input[last_pos..end]));
          last_pos = end;
          i = end - 1;
        }
        b'/' if self.bytes.get(i + 1) == Some(&b'*') => {
          i += 1;
          in_comment = true;
          block_depth += 1;
        }
        b'\'' | b'"' => {
          if let Some((q, s)) = self.quoted_at(i) {
            groups.push(UntilGroup::Text(&self.input[last_pos..i]));
            groups.push(UntilGroup::Quoted(q as char, s));
            i += s.len() - 1;
            last_pos = i + 1;
          } else {
            self.skip_whitespace(i - start_pos);
            return Some(ParseUntil::Unexpected((next_c as char).to_string()));
          }
        }
        b'{' => {
          block_stack.push(b'}');
          block_depth += 1;
        }
        b'(' => {
          block_stack.push(b')');
          block_depth += 1;
          paren_depth += 1;
        }
        b'[' => {
          block_stack.push(b']');
          block_depth += 1;
        }
        b'}' | b')' | b']' => {
          let expected = block_stack.pop();
          if Some(next_c) == expected {
            block_depth -= 1;
            if next_c == b')' && paren_depth > 0 {
              paren_depth -= 1;
            }
          } else {
            self.skip_whitespace(i - start_pos);
            return expected.map(|e| ParseUntil::Unexpected((e as char).to_string()));
          }
        }
        _ => {}
      }
      i += 1;
      if i > length {
        return None;
      }
    }
  }

  pub fn end(&mut self) -> EndInfo {
    let is_finished = self.i >= self.bytes.len();
    let mut message = None;
    if self.i < self.furthest {
      message = self.furthest_msg.clone();
      self.i = self.furthest;
    }
    EndInfo {
      is_finished,
      furthest: self.i,
      furthest_possible_error_message: message,
      furthest_reached_end: self.i + 1 >= self.bytes.len(),
      furthest_char: self.bytes.get(self.i).copied(),
    }
  }
}

fn ceil_boundary(s: &str, mut i: usize) -> usize {
  while i < s.len() && !s.is_char_boundary(i) {
    i += 1;
  }
  i.min(s.len())
}

#[inline]
pub fn memchr(needle: u8, hay: &[u8]) -> Option<usize> {
  hay.iter().position(|&b| b == needle)
}

pub fn find_str(hay: &[u8], needle: &[u8]) -> Option<usize> {
  if needle.is_empty() {
    return Some(0);
  }
  let first = needle[0];
  let mut start = 0;
  while let Some(p) = memchr(first, &hay[start..]) {
    let at = start + p;
    if hay[at..].starts_with(needle) {
      return Some(at);
    }
    start = at + 1;
  }
  None
}
