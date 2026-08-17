use std::{collections::VecDeque, str};

use bitflags::bitflags;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::{
  Range,
  css_syntax::{
    MAX_CSS_KEYWORD_LEN, decode_css_keyword, is_css_modules_pure_magic_comment,
    lowercase_ascii_keyword, strip_vendor_prefix,
  },
  dependencies::{PropertyKind, special_value_is_candidate},
};

const fn build_plain_ascii_name_byte_table() -> [bool; 256] {
  let mut table = [false; 256];
  let mut byte = 0usize;
  while byte < table.len() {
    table[byte] = (byte >= b'0' as usize && byte <= b'9' as usize)
      || (byte >= b'A' as usize && byte <= b'Z' as usize)
      || (byte >= b'a' as usize && byte <= b'z' as usize)
      || byte == b'_' as usize
      || byte == b'-' as usize;
    byte += 1;
  }
  table
}

const PLAIN_ASCII_NAME_BYTE: [bool; 256] = build_plain_ascii_name_byte_table();

pub const C_LINE_FEED: u8 = b'\n';
pub const C_CARRIAGE_RETURN: u8 = b'\r';
pub const C_FORM_FEED: u8 = b'\x0c';

pub const C_TAB: u8 = b'\t';
pub const C_SPACE: u8 = b' ';

pub const C_SOLIDUS: u8 = b'/';
pub const C_REVERSE_SOLIDUS: u8 = b'\\';
pub const C_ASTERISK: u8 = b'*';

pub const C_LEFT_PARENTHESIS: u8 = b'(';
pub const C_RIGHT_PARENTHESIS: u8 = b')';
pub const C_LEFT_CURLY: u8 = b'{';
pub const C_RIGHT_CURLY: u8 = b'}';
pub const C_LEFT_SQUARE: u8 = b'[';
pub const C_RIGHT_SQUARE: u8 = b']';

pub const C_QUOTATION_MARK: u8 = b'"';
pub const C_APOSTROPHE: u8 = b'\'';

pub const C_FULL_STOP: u8 = b'.';
pub const C_COLON: u8 = b':';
pub const C_SEMICOLON: u8 = b';';
pub const C_COMMA: u8 = b',';
pub const C_PERCENTAGE: u8 = b'%';
pub const C_AT_SIGN: u8 = b'@';

pub const C_LOW_LINE: u8 = b'_';
pub const C_LOWER_E: u8 = b'e';
pub const C_UPPER_E: u8 = b'E';

pub const C_NUMBER_SIGN: u8 = b'#';
pub const C_PLUS_SIGN: u8 = b'+';
pub const C_HYPHEN_MINUS: u8 = b'-';

pub type Pos = u32;

/// The lexical kind of a CSS token.
///
/// Tokens retain only byte ranges into the original source.  Comments are
/// emitted deliberately so that CSS Modules magic comments can be interpreted
/// without a second scan of the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
  Eof,
  Ident,
  AtKeyword,
  Hash,
  IdHash,
  QuotedString,
  Url,
  Function,
  Number,
  Percentage,
  Dimension,
  WhiteSpace,
  Comment,
  BadComment,
  BadString,
  BadUrl,
  Delim,
  Colon,
  Semicolon,
  Comma,
  LeftParenthesis,
  RightParenthesis,
  LeftSquareBracket,
  RightSquareBracket,
  LeftCurlyBracket,
  RightCurlyBracket,
  IncludeMatch,
  DashMatch,
  PrefixMatch,
  SuffixMatch,
  SubstringMatch,
}

impl TokenKind {
  #[inline]
  pub fn is_trivia(self) -> bool {
    matches!(self, Self::WhiteSpace | Self::Comment)
  }

  #[inline]
  pub fn is_scan_error(self) -> bool {
    matches!(self, Self::BadComment | Self::BadString | Self::BadUrl)
  }
}

/// A CSS token represented by ranges into the input string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
  pub kind: TokenKind,
  pub range: Range,
  pub value_range: Range,
  pub flags: TokenFlags,
}

impl Token {
  #[inline]
  pub const fn new(kind: TokenKind, range: Range, value_range: Range) -> Self {
    Self::with_flags(kind, range, value_range, TokenFlags::ascii())
  }

  #[inline]
  pub const fn with_flags(
    kind: TokenKind,
    range: Range,
    value_range: Range,
    flags: TokenFlags,
  ) -> Self {
    Self {
      kind,
      range,
      value_range,
      flags,
    }
  }
}

bitflags! {
    /// Properties collected while scanning a token. They let dependency parsing
    /// skip escape/null checks for the overwhelmingly common plain-ASCII path.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TokenFlags: u8 {
        const HAS_ESCAPE = 1 << 0;
        const HAS_NULL = 1 << 1;
        const IS_ASCII = 1 << 2;
    }
}

impl TokenFlags {
  #[inline]
  pub const fn ascii() -> Self {
    Self::IS_ASCII
  }

  #[inline]
  pub const fn has_escape(self) -> bool {
    self.contains(Self::HAS_ESCAPE)
  }

  #[inline]
  pub const fn has_null(self) -> bool {
    self.contains(Self::HAS_NULL)
  }

  #[inline]
  pub const fn is_ascii(self) -> bool {
    self.contains(Self::IS_ASCII)
  }

  #[inline]
  fn mark_escape(&mut self) {
    self.insert(Self::HAS_ESCAPE);
  }

  #[inline]
  fn mark_null(&mut self) {
    self.insert(Self::HAS_NULL);
  }

  #[inline]
  fn mark_non_ascii(&mut self) {
    self.remove(Self::IS_ASCII);
  }

  #[inline]
  fn merge(&mut self, other: Self) {
    let is_ascii = self.is_ascii() && other.is_ascii();
    self.insert(other);
    if !is_ascii {
      self.mark_non_ascii();
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trivia {
  pub range: Range,
  pub end: Pos,
  pub first_comment_start: Option<Pos>,
  pub has_white_space: bool,
}

impl Trivia {
  #[inline]
  pub fn has_whitespace(self) -> bool {
    self.has_white_space
  }
}

/// A significant token together with all immediately preceding whitespace and
/// comments.  The leading trivia is consumed once and never rescanned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenWithTrivia {
  pub token: Token,
  pub leading: Trivia,
}

pub trait LexerVisitor {
  fn visit_ident(&mut self, name: &str, range: Range);
}

impl LexerVisitor for () {
  #[inline(always)]
  fn visit_ident(&mut self, _name: &str, _range: Range) {}
}

#[derive(Debug)]
pub struct Lexer<'s, V: LexerVisitor = ()> {
  value: &'s [u8],
  scan_pos: Pos,
  visitor: V,
}

impl<'s, V: LexerVisitor> Lexer<'s, V> {
  pub fn new(value: &'s str, visitor: V) -> Self {
    assert!(value.len() <= Pos::MAX as usize, "CSS input is too large");
    Self {
      value: value.as_bytes(),
      scan_pos: 0,
      visitor,
    }
  }

  #[inline]
  pub(crate) fn visitor_mut(&mut self) -> &mut V {
    &mut self.visitor
  }

  #[inline]
  fn visit_ident(&mut self, kind: TokenKind, range: Range) {
    if !matches!(kind, TokenKind::Ident | TokenKind::Function) {
      return;
    }
    let value = &self.value[range.start as usize..range.end as usize];
    // SAFETY: token value ranges always lie on UTF-8 boundaries.
    let value = unsafe { str::from_utf8_unchecked(value) };
    self.visitor.visit_ident(value, range);
  }

  #[inline]
  pub(crate) fn source_end(&self) -> Pos {
    self.value.len() as Pos
  }

  #[inline(always)]
  pub(crate) fn byte_at(&self, position: Pos) -> Option<u8> {
    self.value.get(position as usize).copied()
  }

  #[inline(always)]
  pub(crate) fn could_start_ident_at(&self, position: Pos) -> bool {
    self.byte_at(position).is_some_and(|byte| {
      matches!(byte, C_HYPHEN_MINUS | C_REVERSE_SOLIDUS) || is_name_start_byte(byte)
    })
  }

  /// Skip value text that cannot produce a dependency or change the block
  /// structure. Delimiters of ordinary functions are tracked without
  /// manufacturing tokens, and the scanner stops before dependency-bearing
  /// functions, ICSS symbols, selector-independent magic comments, or a
  /// declaration boundary.
  fn fast_forward_generic_value<F, C>(
    &mut self,
    state: &mut GenericValueScanState,
    options: GenericValueScanOptions,
    stop_at_top_level_left_curly: bool,
    mut is_candidate: F,
    mut is_comment_candidate: C,
    icss_symbols: Option<&FxHashSet<&str>>,
  ) -> Option<PrescannedIdent>
  where
    F: FnMut(&str) -> bool,
    C: FnMut(&str) -> bool,
  {
    let mut position = self.scan_pos as usize;
    let scan_start = position;
    let mut candidate_token = None;
    while position < self.value.len() {
      let byte = self.value[position];
      if is_white_space(byte) {
        position += 1;
        continue;
      }

      if byte == C_SOLIDUS && self.value.get(position + 1) == Some(&C_ASTERISK) {
        let (kind, end, _, _, _) = self.scan_comment(position);
        if options.keep_comments && kind == TokenKind::Comment {
          let content = &self.value[position + 2..end.saturating_sub(2)];
          let mut first = 0usize;
          while content.get(first).is_some_and(|byte| is_white_space(*byte)) {
            first += 1;
          }
          if content.get(first) == Some(&b'c') {
            let content = unsafe { str::from_utf8_unchecked(content) };
            if is_comment_candidate(content) {
              break;
            }
          }
        }
        position = end;
        continue;
      }

      if matches!(byte, C_QUOTATION_MARK | C_APOSTROPHE) {
        if options.preserve_strings {
          break;
        }
        position = self.scan_string(position, byte).1;
        continue;
      }

      let starts_ident = match byte {
        C_HYPHEN_MINUS | C_REVERSE_SOLIDUS => self.starts_ident_at(position),
        _ => is_name_start_byte(byte),
      };
      if starts_ident {
        let end = self.scan_plain_ascii_name(position);
        if end == position || self.raw_name_needs_tokenizer(end) {
          break;
        }
        let flags = TokenFlags::ascii();
        let name = unsafe { str::from_utf8_unchecked(&self.value[position..end]) };
        let is_function = self.value.get(end) == Some(&C_LEFT_PARENTHESIS);
        let is_candidate = !flags.has_escape()
          && (is_candidate(name)
            || options
              .property
              .is_some_and(|property| special_value_is_candidate(property, name, icss_symbols)));
        if flags.has_escape()
          || is_candidate
          || (is_function && is_dependency_value_function(name))
          || (is_function && options.preserve_delimiters)
        {
          if is_candidate && !is_function && flags.is_ascii() && !flags.has_null() {
            candidate_token = Some(PrescannedIdent {
              start: position as Pos,
              end: end as Pos,
              flags,
            });
            position = end;
          }
          break;
        }
        self
          .visitor
          .visit_ident(name, Range::new(position as Pos, end as Pos));
        if is_function {
          state.parentheses += 1;
          position = end + 1;
        } else {
          position = end;
        }
        continue;
      }

      let starts_number = match byte {
        b'0'..=b'9' => true,
        C_PLUS_SIGN | C_HYPHEN_MINUS | C_FULL_STOP => self.starts_number_at(position),
        _ => false,
      };
      if starts_number {
        let Some(end) = self.scan_raw_numeric_end(position) else {
          break;
        };
        position = end;
        continue;
      }

      match byte {
        C_NUMBER_SIGN => {
          let name_start = position + 1;
          if self.starts_ident_at(name_start)
            || self
              .value
              .get(name_start)
              .is_some_and(|byte| is_digit(*byte) || *byte == C_HYPHEN_MINUS)
          {
            let end = self.scan_plain_ascii_name(name_start);
            if end == name_start || self.raw_name_needs_tokenizer(end) {
              break;
            }
            position = end;
          } else {
            position = name_start;
          }
        }
        C_LEFT_PARENTHESIS => {
          if options.preserve_delimiters {
            break;
          }
          state.parentheses += 1;
          position += 1;
        }
        C_RIGHT_PARENTHESIS => {
          if options.preserve_delimiters {
            break;
          }
          state.parentheses = state.parentheses.saturating_sub(1);
          position += 1;
        }
        C_LEFT_SQUARE => {
          if options.preserve_delimiters {
            break;
          }
          state.squares += 1;
          position += 1;
        }
        C_RIGHT_SQUARE => {
          if options.preserve_delimiters {
            break;
          }
          state.squares = state.squares.saturating_sub(1);
          position += 1;
        }
        C_LEFT_CURLY if stop_at_top_level_left_curly && !state.is_nested() => break,
        C_LEFT_CURLY => {
          if options.preserve_delimiters {
            break;
          }
          state.curlies += 1;
          position += 1;
        }
        C_RIGHT_CURLY if state.curlies > 0 && !options.preserve_delimiters => {
          state.curlies -= 1;
          position += 1;
        }
        C_RIGHT_CURLY => break,
        C_SEMICOLON if !state.is_nested() => break,
        _ => position += 1,
      }
    }
    self.scan_pos = position as Pos;
    debug_assert!(
      self.scan_pos >= scan_start as Pos,
      "fast_forward moved scan_pos backward from {scan_start} to {position}"
    );
    candidate_token
  }

  /// Skip selector text that cannot produce a CSS Modules dependency.
  /// Attribute selectors remain opaque across calls through `square_depth`;
  /// the scanner stops before dependency candidates and structural tokens so
  /// the normal token stream remains the sole owner of parser state changes.
  pub(crate) fn fast_forward_selector<C>(
    &mut self,
    square_depth: &mut u32,
    keep_comments: bool,
    has_mode: bool,
    mut is_comment_candidate: C,
  ) -> bool
  where
    C: FnMut(&str) -> bool,
  {
    let mut position = self.scan_pos as usize;
    let scan_start = position;
    let mut invalidates_composes = false;
    let mut trivia_start = None;

    while position < self.value.len() {
      let byte = self.value[position];

      if is_white_space(byte) {
        trivia_start.get_or_insert(position);
        position += 1;
        continue;
      }

      if byte == C_SOLIDUS && self.value.get(position + 1) == Some(&C_ASTERISK) {
        let (kind, end, _, _, _) = self.scan_comment(position);
        if keep_comments && kind == TokenKind::Comment {
          let content = &self.value[position + 2..end.saturating_sub(2)];
          // SAFETY: comments are slices of the original UTF-8 input.
          let content = unsafe { str::from_utf8_unchecked(content) };
          if is_comment_candidate(content) {
            break;
          }
        }
        trivia_start.get_or_insert(position);
        position = end;
        continue;
      }

      if *square_depth > 0 {
        if byte == C_REVERSE_SOLIDUS && self.is_valid_escape_at(position) {
          position = self.scan_escape(position);
          trivia_start = None;
          continue;
        }
        match byte {
          C_QUOTATION_MARK | C_APOSTROPHE => {
            position = self.scan_string(position, byte).1;
          }
          C_LEFT_SQUARE => {
            *square_depth += 1;
            position += 1;
          }
          C_RIGHT_SQUARE => {
            *square_depth -= 1;
            position += 1;
          }
          _ => position += 1,
        }
        trivia_start = None;
        continue;
      }

      let starts_ident = match byte {
        C_HYPHEN_MINUS | C_REVERSE_SOLIDUS => self.starts_ident_at(position),
        _ => is_name_start_byte(byte),
      };
      if starts_ident {
        let end = self.scan_name(position);
        if end == position {
          break;
        }
        // Functions own balanced-stack state in the dependency
        // scanner. In particular, `url()` and `image-set()` can
        // produce dependencies even when invalid CSS made them
        // appear while recovering from a selector. Leave the name
        // to the regular tokenizer so it can emit `Function`.
        if self.value.get(end) == Some(&C_LEFT_PARENTHESIS) {
          break;
        }
        let name = &self.value[position..end];
        // SAFETY: scanned names start and end on UTF-8 boundaries.
        let name = unsafe { str::from_utf8_unchecked(name) };
        self
          .visitor
          .visit_ident(name, Range::new(position as Pos, end as Pos));
        invalidates_composes = true;
        trivia_start = None;
        position = end;
        continue;
      }

      let starts_number = match byte {
        b'0'..=b'9' => true,
        C_PLUS_SIGN | C_HYPHEN_MINUS | C_FULL_STOP => self.starts_number_at(position),
        _ => false,
      };
      if starts_number {
        invalidates_composes = true;
        trivia_start = None;
        position = self.scan_numeric(position).1;
        continue;
      }

      match byte {
        C_LEFT_SQUARE => {
          invalidates_composes = true;
          *square_depth = 1;
          trivia_start = None;
          position += 1;
        }
        // Outside an attribute selector a string may belong to
        // `url()` or `image-set()`, whose balanced-stack kind decides
        // whether it creates a dependency.
        C_QUOTATION_MARK | C_APOSTROPHE => break,
        C_FULL_STOP | C_NUMBER_SIGN | C_COLON if has_mode => break,
        C_NUMBER_SIGN => {
          let name_start = position + 1;
          position = if self.starts_ident_at(name_start)
            || self
              .value
              .get(name_start)
              .is_some_and(|byte| is_digit(*byte) || *byte == C_HYPHEN_MINUS)
          {
            self.scan_name(name_start)
          } else {
            name_start
          };
          invalidates_composes = true;
          trivia_start = None;
        }
        C_RIGHT_PARENTHESIS => {
          position = trivia_start.unwrap_or(position);
          break;
        }
        C_LEFT_PARENTHESIS | C_LEFT_CURLY | C_RIGHT_CURLY | C_COMMA | C_SEMICOLON | C_AT_SIGN => {
          break;
        }
        _ => {
          invalidates_composes = true;
          trivia_start = None;
          position += 1;
        }
      }
    }

    self.scan_pos = position as Pos;
    debug_assert!(
      self.scan_pos >= scan_start as Pos,
      "selector fast-forward moved scan_pos backward from {scan_start} to {position}"
    );
    invalidates_composes
  }

  /// Skip a balanced delimiter run whose opening token has already been
  /// consumed. Only `RightParenthesis`, `RightSquareBracket`, and
  /// `RightCurlyBracket` are accepted as `end`.
  ///
  /// Returns the range from the opening token through the matching closing
  /// token on success. The caller is responsible for passing the position
  /// just past the opening token.
  ///
  /// Nested delimiters are tracked on a local stack. Strings, comments,
  /// escapes, and non-ASCII code points are skipped without tokenizing so
  /// their content can never close the run. The scan is transactional: on
  /// failure (EOF or an unbalanced closing token) `scan_pos` is left
  /// unchanged and `None` is returned so the caller can fall back to the
  /// regular tokenizer, which remains the sole owner of parser state
  /// changes.
  pub(crate) fn fast_forward(&mut self, end: TokenKind) -> Option<Range> {
    match end {
      TokenKind::RightParenthesis
      | TokenKind::RightSquareBracket
      | TokenKind::RightCurlyBracket => {}
      _ => unreachable!("fast_forward accepts only closing bracket kinds"),
    }
    let open_start = self.scan_pos;
    let mut stack: SmallVec<[TokenKind; 8]> = SmallVec::new();
    stack.push(end);
    let mut position = open_start as usize;
    let bytes = self.value;
    while position < bytes.len() {
      let byte = bytes[position];
      match byte {
        C_QUOTATION_MARK | C_APOSTROPHE => {
          let (kind, string_end, _, _, _) = self.scan_string(position, byte);
          if kind == TokenKind::BadString {
            return None;
          }
          position = string_end;
        }
        C_SOLIDUS if bytes.get(position + 1) == Some(&C_ASTERISK) => {
          let (kind, comment_end, _, _, _) = self.scan_comment(position);
          if kind == TokenKind::BadComment {
            return None;
          }
          position = comment_end;
        }
        C_REVERSE_SOLIDUS if self.is_valid_escape_at(position) => {
          position = self.scan_escape(position);
        }
        C_LEFT_PARENTHESIS => {
          stack.push(TokenKind::RightParenthesis);
          position += 1;
        }
        C_LEFT_SQUARE => {
          stack.push(TokenKind::RightSquareBracket);
          position += 1;
        }
        C_LEFT_CURLY => {
          stack.push(TokenKind::RightCurlyBracket);
          position += 1;
        }
        C_RIGHT_PARENTHESIS | C_RIGHT_SQUARE | C_RIGHT_CURLY => {
          if *stack.last().expect("delimiter stack never empties") != kind_to_right(byte) {
            return None;
          }
          stack.pop();
          position += 1;
          if stack.is_empty() {
            self.scan_pos = position as Pos;
            debug_assert!(
              self.scan_pos >= open_start,
              "fast_forward moved scan_pos backward from {open_start} to {position}"
            );
            return Some(Range::new(open_start, position as Pos));
          }
        }
        // A semicolon ends an at-rule prelude regardless of nesting
        // depth. Bail so the regular tokenizer resumes and the
        // dependency scanner completes the at-rule at the semicolon.
        C_SEMICOLON => return None,
        _ if byte >= 0x80 => {
          position += self.utf8_width_at(position);
        }
        _ => position += 1,
      }
    }
    None
  }

  pub(crate) fn slice(&self, start: Pos, end: Pos) -> Option<&'s str> {
    let range = Range::new(start, end);
    let start = start as usize;
    let end = end as usize;
    if start > end || end > self.value.len() {
      return None;
    }
    // SAFETY: Lexer-generated positions always delimit complete tokens or
    // source fragments and therefore stay on UTF-8 code point boundaries.
    Some(unsafe { slice_unchecked(self.value, &range) })
  }

  #[inline(always)]
  pub(crate) fn slice_trusted(&self, start: Pos, end: Pos) -> &'s str {
    debug_assert!(start <= end && end <= self.value.len() as Pos);
    // SAFETY: Token ranges and parser positions originate from this
    // lexer, so they delimit valid UTF-8 source boundaries.
    unsafe { slice_unchecked(self.value, &Range::new(start, end)) }
  }
}

impl Lexer<'_, ()> {
  pub fn slice_range<'a>(input: &'a str, range: &Range) -> Option<&'a str> {
    let start = range.start as usize;
    let end = range.end as usize;
    if start > end
      || end > input.len()
      || !input.is_char_boundary(start)
      || !input.is_char_boundary(end)
    {
      return None;
    }
    // SAFETY: The range was checked against the original `str` above.
    Some(unsafe { slice_unchecked(input.as_bytes(), range) })
  }
}

unsafe fn slice_unchecked<'a>(input: &'a [u8], range: &Range) -> &'a str {
  unsafe {
    let value = input.get_unchecked(range.start as usize..range.end as usize);
    str::from_utf8_unchecked(value)
  }
}

impl<V: LexerVisitor> Lexer<'_, V> {
  #[inline]
  pub(crate) fn scan_pos(&self) -> Pos {
    self.scan_pos
  }
}

impl<'s, V: LexerVisitor> Lexer<'s, V> {
  /// Return the next CSS token without allocating or decoding its value.
  ///
  /// `TokenKind::Eof` is returned after the input is exhausted. Unterminated
  /// comments, strings, and URLs are represented by the corresponding
  /// `Bad*` token kinds, so EOF and tokenizer errors are never conflated.
  #[inline]
  pub fn next_token(&mut self) -> Token {
    let start = self.scan_pos as usize;
    let len = self.value.len();
    if start >= len {
      self.scan_pos = len as Pos;
      return Token::new(
        TokenKind::Eof,
        Range::new(len as Pos, len as Pos),
        Range::new(len as Pos, len as Pos),
      );
    }

    let byte = self.value[start];
    let (kind, end, value_start, value_end, flags) = match byte {
      C_SPACE | C_TAB | C_LINE_FEED | C_CARRIAGE_RETURN | C_FORM_FEED => {
        self.scan_whitespace(start)
      }
      C_QUOTATION_MARK => self.scan_string(start, C_QUOTATION_MARK),
      C_APOSTROPHE => self.scan_string(start, C_APOSTROPHE),
      C_NUMBER_SIGN => {
        let value_start = start + 1;
        if self.starts_ident_at(value_start) {
          let (end, flags) = self.scan_name_with_flags(value_start);
          (TokenKind::IdHash, end, value_start, end, flags)
        } else if self
          .value
          .get(value_start)
          .is_some_and(|byte| is_digit(*byte) || *byte == C_HYPHEN_MINUS)
        {
          let (end, flags) = self.scan_name_with_flags(value_start);
          (TokenKind::Hash, end, value_start, end, flags)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_LEFT_PARENTHESIS => (
        TokenKind::LeftParenthesis,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_RIGHT_PARENTHESIS => (
        TokenKind::RightParenthesis,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_LEFT_SQUARE => (
        TokenKind::LeftSquareBracket,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_RIGHT_SQUARE => (
        TokenKind::RightSquareBracket,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_LEFT_CURLY => (
        TokenKind::LeftCurlyBracket,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_RIGHT_CURLY => (
        TokenKind::RightCurlyBracket,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_COLON => (
        TokenKind::Colon,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_SEMICOLON => (
        TokenKind::Semicolon,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_COMMA => (
        TokenKind::Comma,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
      C_PLUS_SIGN => {
        if self.starts_number_at(start) {
          self.scan_numeric(start)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_HYPHEN_MINUS => {
        if self.starts_number_at(start) {
          self.scan_numeric(start)
        } else if self.starts_ident_at(start) {
          self.scan_ident_like(start)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_FULL_STOP => {
        if self.starts_number_at(start) {
          self.scan_numeric(start)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_SOLIDUS => {
        if self.value.get(start + 1) == Some(&C_ASTERISK) {
          self.scan_comment(start)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_AT_SIGN => {
        let value_start = start + 1;
        if self.starts_ident_at(value_start) {
          let (end, flags) = self.scan_name_with_flags(value_start);
          (TokenKind::AtKeyword, end, value_start, end, flags)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      C_REVERSE_SOLIDUS => {
        if self.is_valid_escape_at(start) {
          self.scan_ident_like(start)
        } else {
          (
            TokenKind::Delim,
            start + 1,
            start,
            start + 1,
            TokenFlags::ascii(),
          )
        }
      }
      b'0'..=b'9' => self.scan_numeric(start),
      b'$' if self.value[start..].starts_with(b"$=") => (
        TokenKind::SuffixMatch,
        start + 2,
        start,
        start + 2,
        TokenFlags::ascii(),
      ),
      b'^' if self.value[start..].starts_with(b"^=") => (
        TokenKind::PrefixMatch,
        start + 2,
        start,
        start + 2,
        TokenFlags::ascii(),
      ),
      b'|' if self.value[start..].starts_with(b"|=") => (
        TokenKind::DashMatch,
        start + 2,
        start,
        start + 2,
        TokenFlags::ascii(),
      ),
      b'~' if self.value[start..].starts_with(b"~=") => (
        TokenKind::IncludeMatch,
        start + 2,
        start,
        start + 2,
        TokenFlags::ascii(),
      ),
      b'*' if self.value[start..].starts_with(b"*=") => (
        TokenKind::SubstringMatch,
        start + 2,
        start,
        start + 2,
        TokenFlags::ascii(),
      ),
      _ if self.starts_ident_at(start) => self.scan_ident_like(start),
      _ => (
        TokenKind::Delim,
        start + 1,
        start,
        start + 1,
        TokenFlags::ascii(),
      ),
    };

    self.scan_pos = end as Pos;
    debug_assert!(
      self.scan_pos >= start as Pos,
      "scan_pos moved backward from {start} to {}",
      self.scan_pos
    );
    let token = Token::with_flags(
      kind,
      Range::new(start as Pos, end as Pos),
      Range::new(value_start as Pos, value_end as Pos),
      flags,
    );
    self.visit_ident(token.kind, token.value_range);
    token
  }

  #[inline]
  fn scan_whitespace(&self, start: usize) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let bytes = self.value;
    let mut end = start;
    while end < bytes.len() && is_white_space(bytes[end]) {
      end += 1;
    }
    (TokenKind::WhiteSpace, end, start, end, TokenFlags::ascii())
  }

  fn scan_comment(&self, start: usize) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let bytes = self.value;
    let mut end = start + 2;
    let mut flags = TokenFlags::ascii();
    while end < bytes.len() {
      if bytes[end] == C_ASTERISK && bytes.get(end + 1) == Some(&C_SOLIDUS) {
        return (TokenKind::Comment, end + 2, start + 2, end, flags);
      }
      if bytes[end] == 0 {
        flags.mark_null();
      } else if !bytes[end].is_ascii() {
        flags.mark_non_ascii();
      }
      end += if bytes[end] < 0x80 {
        1
      } else {
        self.utf8_width_at(end)
      };
    }
    (
      TokenKind::BadComment,
      bytes.len(),
      start + 2,
      bytes.len(),
      flags,
    )
  }

  fn scan_string(&self, start: usize, quote: u8) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let mut end = start + 1;
    let mut flags = TokenFlags::ascii();
    while end < self.value.len() {
      let byte = self.value[end];
      if byte == quote {
        return (TokenKind::QuotedString, end + 1, start + 1, end, flags);
      }
      if is_new_line(byte) {
        return (TokenKind::BadString, end, start + 1, end, flags);
      }
      if byte == 0 {
        flags.mark_null();
      } else if !byte.is_ascii() {
        flags.mark_non_ascii();
      }
      if byte == C_REVERSE_SOLIDUS {
        flags.mark_escape();
        if self.value.get(end + 1).is_some_and(|next| !next.is_ascii()) {
          flags.mark_non_ascii();
        }
        if self
          .value
          .get(end + 1)
          .is_some_and(|next| is_new_line(*next))
        {
          end += 1;
          if self.value.get(end) == Some(&C_CARRIAGE_RETURN)
            && self.value.get(end + 1) == Some(&C_LINE_FEED)
          {
            end += 1;
          }
          end += 1;
        } else if self.is_valid_escape_at(end) {
          end = self.scan_escape(end);
        } else {
          end += 1;
        }
      } else {
        end += self.utf8_width_at(end);
      }
    }
    (TokenKind::BadString, end, start + 1, end, flags)
  }

  fn scan_ident_like(&self, start: usize) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let (name_end, name_flags) = self.scan_name_with_flags(start);
    if self.value.get(name_end) != Some(&C_LEFT_PARENTHESIS) {
      return (TokenKind::Ident, name_end, start, name_end, name_flags);
    }

    let open_end = name_end + 1;
    let name = &self.value[start..name_end];
    // SAFETY: a scanned name starts and ends on UTF-8 boundaries.
    let name = unsafe { str::from_utf8_unchecked(name) };
    let mut normalized = [0; MAX_CSS_KEYWORD_LEN];
    let normalized = if name_flags.has_escape() {
      decode_css_keyword(name, &mut normalized)
    } else {
      lowercase_ascii_keyword(name, &mut normalized)
    };
    let is_url = normalized == Some("url");
    if is_url {
      let content_start = self.skip_white_space(open_end);
      if !matches!(
        self.value.get(content_start),
        Some(&C_QUOTATION_MARK) | Some(&C_APOSTROPHE)
      ) {
        let (kind, end, value_start, value_end, url_flags) = self.scan_url(start, content_start);
        let mut flags = name_flags;
        flags.merge(url_flags);
        return (kind, end, value_start, value_end, flags);
      }
    }
    (TokenKind::Function, open_end, start, name_end, name_flags)
  }

  fn scan_url(
    &self,
    start: usize,
    content_start: usize,
  ) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let mut end = content_start;
    let mut flags = TokenFlags::ascii();
    while end < self.value.len() {
      let byte = self.value[end];
      if byte == C_RIGHT_PARENTHESIS {
        return (TokenKind::Url, end + 1, content_start, end, flags);
      }
      if is_white_space(byte) {
        let content_end = end;
        let close = self.skip_white_space(end);
        if self.value.get(close) == Some(&C_RIGHT_PARENTHESIS) {
          return (TokenKind::Url, close + 1, content_start, content_end, flags);
        }
        return self.scan_bad_url(start, close, content_start, flags);
      }
      if byte == 0 {
        flags.mark_null();
      } else if !byte.is_ascii() {
        flags.mark_non_ascii();
      }
      if byte == C_QUOTATION_MARK
        || byte == C_APOSTROPHE
        || byte == C_LEFT_PARENTHESIS
        || is_non_printable(byte)
      {
        return self.scan_bad_url(start, end, content_start, flags);
      }
      if byte == C_REVERSE_SOLIDUS {
        flags.mark_escape();
        if self.value.get(end + 1).is_some_and(|next| !next.is_ascii()) {
          flags.mark_non_ascii();
        }
        if self.is_valid_escape_at(end) {
          end = self.scan_escape(end);
        } else if self
          .value
          .get(end + 1)
          .is_some_and(|next| is_new_line(*next))
        {
          end += 2;
          if self.value.get(end - 1) == Some(&C_CARRIAGE_RETURN)
            && self.value.get(end) == Some(&C_LINE_FEED)
          {
            end += 1;
          }
        } else {
          return self.scan_bad_url(start, end, content_start, flags);
        }
      } else {
        end += self.utf8_width_at(end);
      }
    }
    (
      TokenKind::BadUrl,
      self.value.len(),
      content_start,
      self.value.len(),
      flags,
    )
  }

  fn scan_bad_url(
    &self,
    _start: usize,
    mut end: usize,
    content_start: usize,
    mut flags: TokenFlags,
  ) -> (TokenKind, usize, usize, usize, TokenFlags) {
    while end < self.value.len() {
      let byte = self.value[end];
      if byte == 0 {
        flags.mark_null();
      } else if !byte.is_ascii() {
        flags.mark_non_ascii();
      }
      if byte == C_RIGHT_PARENTHESIS {
        end += 1;
        break;
      }
      if byte == C_REVERSE_SOLIDUS {
        flags.mark_escape();
        if self.value.get(end + 1).is_some_and(|next| !next.is_ascii()) {
          flags.mark_non_ascii();
        }
        if self.is_valid_escape_at(end) {
          end = self.scan_escape(end);
        } else if self
          .value
          .get(end + 1)
          .is_some_and(|next| is_new_line(*next))
        {
          end += 2;
          if self.value.get(end - 1) == Some(&C_CARRIAGE_RETURN)
            && self.value.get(end) == Some(&C_LINE_FEED)
          {
            end += 1;
          }
        } else {
          end += 1;
        }
      } else {
        end += self.utf8_width_at(end);
      }
    }
    (TokenKind::BadUrl, end, content_start, end, flags)
  }

  fn scan_numeric(&self, start: usize) -> (TokenKind, usize, usize, usize, TokenFlags) {
    let end = self.scan_number_end(start);
    if self.value.get(end) == Some(&C_PERCENTAGE) {
      return (
        TokenKind::Percentage,
        end + 1,
        start,
        end + 1,
        TokenFlags::ascii(),
      );
    }
    if self.starts_ident_at(end) {
      let (end, flags) = self.scan_name_with_flags(end);
      return (TokenKind::Dimension, end, start, end, flags);
    }
    (TokenKind::Number, end, start, end, TokenFlags::ascii())
  }

  #[inline]
  fn scan_number_end(&self, start: usize) -> usize {
    let mut end = start;
    if matches!(
      self.value.get(end),
      Some(&C_PLUS_SIGN) | Some(&C_HYPHEN_MINUS)
    ) {
      end += 1;
    }
    while self.value.get(end).is_some_and(|byte| is_digit(*byte)) {
      end += 1;
    }
    if self.value.get(end) == Some(&C_FULL_STOP)
      && self.value.get(end + 1).is_some_and(|byte| is_digit(*byte))
    {
      end += 1;
      while self.value.get(end).is_some_and(|byte| is_digit(*byte)) {
        end += 1;
      }
    }
    if matches!(self.value.get(end), Some(&C_LOWER_E) | Some(&C_UPPER_E)) {
      let exponent_start = end;
      let mut exponent_end = end + 1;
      if matches!(
        self.value.get(exponent_end),
        Some(&C_PLUS_SIGN) | Some(&C_HYPHEN_MINUS)
      ) {
        exponent_end += 1;
      }
      if self
        .value
        .get(exponent_end)
        .is_some_and(|byte| is_digit(*byte))
      {
        end = exponent_end + 1;
        while self.value.get(end).is_some_and(|byte| is_digit(*byte)) {
          end += 1;
        }
      } else {
        end = exponent_start;
      }
    }
    end
  }

  #[inline]
  fn skip_white_space(&self, mut position: usize) -> usize {
    while self
      .value
      .get(position)
      .is_some_and(|byte| is_white_space(*byte))
    {
      position += 1;
    }
    position
  }

  #[inline]
  fn scan_plain_ascii_name(&self, mut position: usize) -> usize {
    while self
      .value
      .get(position)
      .is_some_and(|byte| PLAIN_ASCII_NAME_BYTE[*byte as usize])
    {
      position += 1;
    }
    position
  }

  #[inline]
  fn raw_name_needs_tokenizer(&self, position: usize) -> bool {
    self
      .value
      .get(position)
      .is_some_and(|byte| *byte == 0 || *byte == C_REVERSE_SOLIDUS || !byte.is_ascii())
  }

  #[inline]
  fn scan_raw_numeric_end(&self, start: usize) -> Option<usize> {
    let end = self.scan_number_end(start);
    if self.value.get(end) == Some(&C_PERCENTAGE) {
      return Some(end + 1);
    }
    if self.starts_ident_at(end) {
      let name_end = self.scan_plain_ascii_name(end);
      if name_end == end || self.raw_name_needs_tokenizer(name_end) {
        return None;
      }
      return Some(name_end);
    }
    if self.raw_name_needs_tokenizer(end) {
      return None;
    }
    Some(end)
  }

  #[inline]
  fn scan_name(&self, position: usize) -> usize {
    self.scan_name_impl(position, false).0
  }

  #[inline]
  fn scan_name_with_flags(&self, position: usize) -> (usize, TokenFlags) {
    self.scan_name_impl(position, true)
  }

  #[inline]
  fn scan_name_impl(&self, mut position: usize, track_flags: bool) -> (usize, TokenFlags) {
    let bytes = self.value;
    let mut flags = TokenFlags::ascii();
    while position < bytes.len() {
      while position < bytes.len() {
        let byte = bytes[position];
        if PLAIN_ASCII_NAME_BYTE[byte as usize] {
          position += 1;
        } else if byte == 0 {
          if track_flags {
            flags.mark_null();
          }
          position += 1;
        } else {
          break;
        }
      }
      if position == bytes.len() {
        break;
      }

      let byte = bytes[position];
      if byte == C_REVERSE_SOLIDUS {
        if self.is_valid_escape_at(position) {
          if track_flags {
            flags.mark_escape();
            if bytes.get(position + 1).is_some_and(|next| !next.is_ascii()) {
              flags.mark_non_ascii();
            }
          }
          position = self.scan_escape(position);
        } else {
          break;
        }
      } else if byte.is_ascii() {
        break;
      } else {
        if track_flags {
          flags.mark_non_ascii();
        }
        position += self.utf8_width_at(position);
      }
    }
    (position, flags)
  }

  #[inline]
  fn starts_ident_at(&self, position: usize) -> bool {
    let Some(byte) = self.value.get(position).copied() else {
      return false;
    };
    match byte {
      C_HYPHEN_MINUS => match self.value.get(position + 1).copied() {
        Some(next) => {
          is_name_start_byte(next)
            || next == C_HYPHEN_MINUS
            || self.is_valid_escape_at(position + 1)
        }
        None => false,
      },
      C_REVERSE_SOLIDUS => self.is_valid_escape_at(position),
      _ => is_name_start_byte(byte),
    }
  }

  #[inline]
  fn starts_number_at(&self, position: usize) -> bool {
    let Some(first) = self.value.get(position).copied() else {
      return false;
    };
    let second = self.value.get(position + 1).copied();
    let third = self.value.get(position + 2).copied();
    match first {
      C_PLUS_SIGN | C_HYPHEN_MINUS => {
        second.is_some_and(is_digit) || (second == Some(C_FULL_STOP) && third.is_some_and(is_digit))
      }
      C_FULL_STOP => second.is_some_and(is_digit),
      _ => is_digit(first),
    }
  }

  #[inline]
  fn is_valid_escape_at(&self, position: usize) -> bool {
    self.value.get(position) == Some(&C_REVERSE_SOLIDUS)
      && self
        .value
        .get(position + 1)
        .is_some_and(|byte| !is_new_line(*byte))
  }

  #[inline]
  fn scan_escape(&self, position: usize) -> usize {
    debug_assert!(self.is_valid_escape_at(position));
    let mut end = position + 1;
    if self.value[end].is_ascii_hexdigit() {
      let mut digits = 0;
      while end < self.value.len() && digits < 6 && self.value[end].is_ascii_hexdigit() {
        end += 1;
        digits += 1;
      }
      if self
        .value
        .get(end)
        .is_some_and(|byte| is_white_space(*byte))
      {
        end += 1;
        if self.value.get(end - 1) == Some(&C_CARRIAGE_RETURN)
          && self.value.get(end) == Some(&C_LINE_FEED)
        {
          end += 1;
        }
      }
      return end;
    }
    end + self.utf8_width_at(end)
  }

  #[inline]
  fn utf8_width_at(&self, position: usize) -> usize {
    let byte = self.value[position];
    if byte < 0x80 {
      1
    } else if byte < 0xE0 {
      2
    } else if byte < 0xF0 {
      3
    } else {
      4
    }
  }
}

/// The single forward stream used by dependency extraction.
///
/// The stream owns the main scanner state and gives the dependency scanner one
/// token of lookahead without cloning the lexer for every decision. Special
/// parsers use the same stream through `next_parser_token`, so consuming a
/// subgrammar never creates a second scanner over the source.
///
/// The cursor is strictly one-directional:
///
/// ```text
/// Lexer::scan_pos         farthest tokenized position, only advances
/// TokenStream::consumed   farthest semantically consumed position, only advances
/// buffered lookahead      tokens scanned but not yet consumed
/// ```
///
/// `next` consumes from the buffer first and only calls `lexer.next_token`
/// when the buffer is empty; it never moves the scanner backwards, so every
/// source range is tokenized at most once and every token is consumed at most
/// once.
pub(crate) struct TokenStream<'a, 's, V: LexerVisitor = ()> {
  lexer: &'a mut Lexer<'s, V>,
  consumed: Pos,
  buffered: VecDeque<TokenWithTrivia>,
  generic_value_state: GenericValueScanState,
  at_rule_state: GenericValueScanState,
  special_value_state: GenericValueScanState,
}

#[derive(Debug, Default)]
pub(crate) struct GenericValueScanState {
  parentheses: u32,
  squares: u32,
  curlies: u32,
}

#[derive(Debug, Clone, Copy)]
struct GenericValueScanOptions {
  keep_comments: bool,
  preserve_strings: bool,
  preserve_delimiters: bool,
  property: Option<PropertyKind>,
}

#[derive(Debug, Clone, Copy)]
struct PrescannedIdent {
  start: Pos,
  end: Pos,
  flags: TokenFlags,
}

impl GenericValueScanState {
  #[inline]
  fn is_nested(&self) -> bool {
    self.parentheses != 0 || self.squares != 0 || self.curlies != 0
  }
}

#[inline]
fn is_dependency_value_function(name: &str) -> bool {
  let mut lowercase = [0; MAX_CSS_KEYWORD_LEN];
  let Some(name) = lowercase_ascii_keyword(name, &mut lowercase) else {
    return name.starts_with("--");
  };
  matches!(name, "url" | "var" | "image-set")
    || strip_vendor_prefix(name) == Some("image-set")
    || name.starts_with("--")
}

#[inline(always)]
fn never_fast_forward_candidate(_: &str) -> bool {
  false
}

impl<'a, 's, V: LexerVisitor> TokenStream<'a, 's, V> {
  pub(crate) fn from_lexer(lexer: &'a mut Lexer<'s, V>) -> Self {
    Self {
      lexer,
      consumed: 0,
      buffered: VecDeque::new(),
      generic_value_state: GenericValueScanState::default(),
      at_rule_state: GenericValueScanState::default(),
      special_value_state: GenericValueScanState::default(),
    }
  }

  #[inline(always)]
  pub(crate) fn next(&mut self, keep_comments: bool) -> TokenWithTrivia {
    let token = if let Some(token) = self.buffered.pop_front() {
      token
    } else {
      self.read_significant(keep_comments)
    };
    debug_assert!(
      token.token.range.start >= self.consumed,
      "token range starts before consumed_pos: {:?} < {}",
      token.token.range,
      self.consumed
    );
    self.consumed = token.token.range.end;
    if matches!(
      token.token.kind,
      TokenKind::Semicolon | TokenKind::RightCurlyBracket
    ) {
      self.generic_value_state = GenericValueScanState::default();
    }
    debug_assert!(
      self.consumed <= self.lexer.scan_pos(),
      "consumed_pos ({}) exceeded scan_pos ({})",
      self.consumed,
      self.lexer.scan_pos()
    );
    token
  }

  /// Consume the next parser token while folding comments into its leading
  /// trivia. This is the equivalent of the old isolated cursor's behavior,
  /// but it advances the main stream and therefore preserves one scanner
  /// ownership for all subgrammars.
  #[inline]
  pub(crate) fn next_parser_token(&mut self) -> TokenWithTrivia {
    let mut item = self.next(true);
    if item.token.kind != TokenKind::Comment {
      return item;
    }

    let start = item.leading.range.start;
    let first_comment_start = item
      .leading
      .first_comment_start
      .or(Some(item.token.range.start));
    let mut has_white_space = item.leading.has_white_space;
    loop {
      item = self.next(true);
      has_white_space |= item.leading.has_white_space;
      if item.token.kind == TokenKind::Comment {
        continue;
      }
      let end = item.leading.end;
      item.leading = Trivia {
        range: Range::new(start, end),
        end,
        first_comment_start,
        has_white_space,
      };
      return item;
    }
  }

  #[inline]
  pub(crate) fn peek(&mut self, keep_comments: bool) -> TokenWithTrivia {
    if self.buffered.is_empty() {
      let item = self.read_significant(keep_comments);
      self.buffered.push_back(item);
    }
    *self.buffered.front().unwrap()
  }

  /// Peek the next parser token (folding comments into its leading trivia)
  /// without consuming it or any preceding comments. The stream state is
  /// unchanged: `next` later returns the same tokens in the same order.
  #[inline]
  pub(crate) fn peek_parser_token(&mut self) -> TokenWithTrivia {
    self.peek(true);
    let mut leading_start = None;
    let mut first_comment_start = None;
    let mut has_white_space = false;
    let mut index = 0usize;
    loop {
      if index == self.buffered.len() {
        let item = self.read_significant(true);
        self.buffered.push_back(item);
      }
      let item = self.buffered[index];
      leading_start.get_or_insert(item.leading.range.start);
      has_white_space |= item.leading.has_whitespace();
      if matches!(item.token.kind, TokenKind::Comment | TokenKind::BadComment) {
        first_comment_start.get_or_insert(item.token.range.start);
        index += 1;
        continue;
      }

      let end = item.leading.end;
      let mut result = item;
      result.leading = Trivia {
        range: Range::new(leading_start.unwrap_or(end), end),
        end,
        first_comment_start: first_comment_start.or(item.leading.first_comment_start),
        has_white_space,
      };
      return result;
    }
  }

  /// Peek at the next significant token, leaving any skipped comments in
  /// the buffer for `next` to consume in source order.
  #[inline]
  pub(crate) fn peek_significant_skipping_comments(
    &mut self,
    keep_comments: bool,
  ) -> TokenWithTrivia {
    self.peek(keep_comments);
    for item in &self.buffered {
      if !matches!(item.token.kind, TokenKind::Comment | TokenKind::BadComment) {
        return *item;
      }
    }
    loop {
      let item = self.read_significant(keep_comments);
      self.buffered.push_back(item);
      if !matches!(item.token.kind, TokenKind::Comment | TokenKind::BadComment) {
        return item;
      }
    }
  }

  /// The position of the next yet-untokenized source byte. The buffer may
  /// still hold tokens; callers must drain `next` before fast-forwarding.
  #[inline]
  pub(crate) fn fast_forward_generic_value_if_buffer_empty<F, C>(
    &mut self,
    keep_comments: bool,
    preserve_strings: bool,
    preserve_delimiters: bool,
    mut is_candidate: F,
    mut is_comment_candidate: C,
  ) where
    F: FnMut(&str) -> bool,
    C: FnMut(&str) -> bool,
  {
    if !self.buffered.is_empty() {
      return;
    }
    let candidate = self.lexer.fast_forward_generic_value(
      &mut self.generic_value_state,
      GenericValueScanOptions {
        keep_comments,
        preserve_strings,
        preserve_delimiters,
        property: None,
      },
      false,
      &mut is_candidate,
      &mut is_comment_candidate,
      None,
    );
    self.finish_value_fast_forward(candidate);
  }

  #[inline]
  pub(crate) fn fast_forward_generic_value_without_candidates_if_buffer_empty(
    &mut self,
    preserve_strings: bool,
    preserve_delimiters: bool,
  ) {
    if !self.buffered.is_empty() {
      return;
    }
    let candidate = self.lexer.fast_forward_generic_value(
      &mut self.generic_value_state,
      GenericValueScanOptions {
        keep_comments: false,
        preserve_strings,
        preserve_delimiters,
        property: None,
      },
      false,
      never_fast_forward_candidate,
      never_fast_forward_candidate,
      None,
    );
    self.finish_value_fast_forward(candidate);
  }

  #[inline]
  pub(crate) fn fast_forward_at_rule_if_buffer_empty<F, C>(
    &mut self,
    keep_comments: bool,
    preserve_strings: bool,
    preserve_delimiters: bool,
    mut is_candidate: F,
    mut is_comment_candidate: C,
  ) where
    F: FnMut(&str) -> bool,
    C: FnMut(&str) -> bool,
  {
    if !self.buffered.is_empty() {
      return;
    }
    let candidate = self.lexer.fast_forward_generic_value(
      &mut self.at_rule_state,
      GenericValueScanOptions {
        keep_comments,
        preserve_strings,
        preserve_delimiters,
        property: None,
      },
      true,
      &mut is_candidate,
      &mut is_comment_candidate,
      None,
    );
    self.finish_value_fast_forward(candidate);
  }

  #[inline]
  pub(crate) fn reset_at_rule_scan_state(&mut self) {
    self.at_rule_state = GenericValueScanState::default();
  }

  #[inline]
  pub(crate) fn fast_forward_special_value_if_buffer_empty(
    &mut self,
    keep_comments: bool,
    preserve_strings: bool,
    preserve_delimiters: bool,
    property: PropertyKind,
    icss_symbols: Option<&FxHashSet<&str>>,
  ) {
    if !self.buffered.is_empty() {
      return;
    }
    let candidate = self.lexer.fast_forward_generic_value(
      &mut self.special_value_state,
      GenericValueScanOptions {
        keep_comments,
        preserve_strings,
        preserve_delimiters,
        property: Some(property),
      },
      false,
      never_fast_forward_candidate,
      is_css_modules_pure_magic_comment,
      icss_symbols,
    );
    self.finish_value_fast_forward(candidate);
  }

  #[inline]
  fn finish_value_fast_forward(&mut self, candidate: Option<PrescannedIdent>) {
    let Some(candidate) = candidate else {
      self.consumed = self.lexer.scan_pos();
      return;
    };
    debug_assert_eq!(candidate.end, self.lexer.scan_pos());
    self.consumed = candidate.start;
    let range = Range::new(candidate.start, candidate.end);
    self.buffered.push_back(TokenWithTrivia {
      token: Token::with_flags(TokenKind::Ident, range, range, candidate.flags),
      leading: Trivia {
        range: Range::new(candidate.start, candidate.start),
        end: candidate.start,
        first_comment_start: None,
        has_white_space: false,
      },
    });
  }

  #[inline]
  pub(crate) fn reset_special_value_scan_state(&mut self) {
    self.special_value_state = GenericValueScanState::default();
  }

  #[inline]
  pub(crate) fn fast_forward_selector_if_buffer_empty<C>(
    &mut self,
    square_depth: &mut u32,
    keep_comments: bool,
    has_mode: bool,
    is_comment_candidate: C,
  ) -> bool
  where
    C: FnMut(&str) -> bool,
  {
    if !self.buffered.is_empty() {
      return false;
    }
    if *square_depth == 0 {
      let scan_pos = self.lexer.scan_pos();
      let Some(next) = self.lexer.byte_at(scan_pos) else {
        return false;
      };
      if matches!(
        next,
        b'"' | b'\'' | b'(' | b')' | b'{' | b'}' | b',' | b';' | b'@'
      ) || (has_mode && matches!(next, b'.' | b'#' | b':'))
      {
        return false;
      }

      // A raw scan has a fixed call/setup cost. Keep short, dense
      // selector fragments on the regular tokenizer path and only
      // enter the scanner when it can skip a useful run. Comments and
      // attributes are exceptions because their contents are opaque.
      let mut probe = scan_pos;
      while probe - scan_pos < 4 {
        let Some(byte) = self.lexer.byte_at(probe) else {
          return false;
        };
        if matches!(byte, b'/' | b'[') {
          break;
        }
        if probe != scan_pos
          && (matches!(
            byte,
            b'"' | b'\'' | b'(' | b')' | b'{' | b'}' | b',' | b';' | b'@'
          ) || (has_mode && matches!(byte, b'.' | b'#' | b':')))
        {
          return false;
        }
        probe += 1;
      }
    }
    let invalidates_composes =
      self
        .lexer
        .fast_forward_selector(square_depth, keep_comments, has_mode, is_comment_candidate);
    self.consumed = self.lexer.scan_pos();
    invalidates_composes
  }

  /// Skip a balanced delimiter run without manufacturing tokens. The opening
  /// token must already be consumed, and no further tokens may be buffered.
  ///
  /// On success `consumed` is advanced to the end of the closing token and
  /// the full range is returned. On failure (`None`) the lexer position is
  /// unchanged so the caller can re-tokenize the region normally.
  pub(crate) fn fast_forward(&mut self, end: TokenKind) -> Option<Range> {
    debug_assert!(
      self.buffered.is_empty(),
      "fast_forward requires an empty lookahead buffer"
    );
    let old_scan_pos = self.lexer.scan_pos();
    let old_consumed = self.consumed;
    let range = self.lexer.fast_forward(end)?;
    self.consumed = range.end;
    debug_assert!(old_scan_pos <= self.lexer.scan_pos());
    debug_assert!(old_consumed <= self.consumed);
    debug_assert!(self.consumed <= self.lexer.scan_pos());
    Some(range)
  }

  /// The farthest position consumed by `next`. This is the current semantic
  /// position of the dependency scanner.
  #[inline]
  pub(crate) fn consumed_pos(&self) -> Pos {
    self.consumed
  }

  #[inline]
  pub(crate) fn lexer(&self) -> &Lexer<'s, V> {
    self.lexer
  }

  #[inline]
  pub(crate) fn lexer_mut(&mut self) -> &mut Lexer<'s, V> {
    self.lexer
  }

  #[inline]
  pub(crate) fn source_end(&self) -> Pos {
    self.lexer.source_end()
  }

  #[inline(always)]
  pub(crate) fn byte_at(&self, position: Pos) -> Option<u8> {
    self.lexer.byte_at(position)
  }

  #[inline]
  pub(crate) fn slice(&self, start: Pos, end: Pos) -> Option<&'s str> {
    self.lexer.slice(start, end)
  }

  #[inline(always)]
  pub(crate) fn slice_trusted(&self, start: Pos, end: Pos) -> &'s str {
    self.lexer.slice_trusted(start, end)
  }

  #[inline(always)]
  fn read_significant(&mut self, keep_comments: bool) -> TokenWithTrivia {
    let start = self.lexer.scan_pos();
    let mut end = start;
    let mut first_comment_start = None;
    let mut has_white_space = false;
    loop {
      let token = self.lexer.next_token();
      debug_assert!(token.range.start >= end);
      if token.kind == TokenKind::WhiteSpace {
        has_white_space = true;
        end = token.range.end;
        continue;
      }
      if !keep_comments && matches!(token.kind, TokenKind::Comment | TokenKind::BadComment) {
        first_comment_start.get_or_insert(token.range.start);
        end = token.range.end;
        continue;
      }
      let leading = Trivia {
        range: Range::new(start, end),
        end,
        first_comment_start,
        has_white_space,
      };
      return TokenWithTrivia { token, leading };
    }
  }
}

pub fn is_new_line(c: u8) -> bool {
  c == C_LINE_FEED || c == C_CARRIAGE_RETURN || c == C_FORM_FEED
}

#[inline]
fn kind_to_right(byte: u8) -> TokenKind {
  match byte {
    C_RIGHT_PARENTHESIS => TokenKind::RightParenthesis,
    C_RIGHT_SQUARE => TokenKind::RightSquareBracket,
    C_RIGHT_CURLY => TokenKind::RightCurlyBracket,
    _ => unreachable!("fast_forward only sees closing bracket bytes"),
  }
}

pub fn is_space(c: u8) -> bool {
  c == C_TAB || c == C_SPACE
}

pub fn is_white_space(c: u8) -> bool {
  is_new_line(c) || is_space(c)
}

pub fn is_digit(c: u8) -> bool {
  c.is_ascii_digit()
}

#[inline]
fn is_name_start_byte(c: u8) -> bool {
  c == 0 || c == C_LOW_LINE || c.is_ascii_alphabetic() || !c.is_ascii()
}

#[inline]
fn is_non_printable(c: u8) -> bool {
  matches!(c, 0x00..=0x08 | 0x0B | 0x0E..=0x1F | 0x7F)
}

#[cfg(test)]
#[path = "../tests/unit/lexer_tests.rs"]
mod tests;
