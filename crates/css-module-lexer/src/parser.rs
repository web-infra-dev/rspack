//! Block-item candidates and their shared error-recovery boundaries.

use smallvec::SmallVec;

use crate::{
  LexerVisitor, Token, TokenKind,
  css_syntax::{MAX_CSS_KEYWORD_LEN, dashed_ident_name_start, decode_css_keyword},
  lexer::{TokenStream, is_white_space},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RecoveryBoundary {
  Semicolon,
  RightCurly,
  Eof,
}

impl RecoveryBoundary {
  pub(crate) fn at(kind: TokenKind, nested: bool) -> Option<Self> {
    match kind {
      TokenKind::Eof => Some(Self::Eof),
      TokenKind::Semicolon if !nested => Some(Self::Semicolon),
      TokenKind::RightCurlyBracket if !nested => Some(Self::RightCurly),
      _ => None,
    }
  }
}

/// Functions and simple blocks are atomic for outer recovery. Mismatched
/// closing tokens remain inside the component; EOF implicitly closes it.
#[derive(Debug, Default)]
pub(crate) struct ComponentValues {
  closing: SmallVec<[TokenKind; 8]>,
}

impl ComponentValues {
  pub(crate) fn is_nested(&self) -> bool {
    !self.closing.is_empty()
  }

  pub(crate) fn expects(&self, kind: TokenKind) -> bool {
    self.closing.last() == Some(&kind)
  }

  pub(crate) fn consume(&mut self, kind: TokenKind) {
    let closing = match kind {
      TokenKind::Function | TokenKind::LeftParenthesis => Some(TokenKind::RightParenthesis),
      TokenKind::LeftSquareBracket => Some(TokenKind::RightSquareBracket),
      TokenKind::LeftCurlyBracket => Some(TokenKind::RightCurlyBracket),
      _ => None,
    };
    if let Some(closing) = closing {
      self.closing.push(closing);
    } else if self.closing.last() == Some(&kind) {
      self.closing.pop();
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BlockItemKind {
  Declaration,
  Rule,
  Invalid,
}

#[derive(Debug)]
enum DeclarationState {
  Colon,
  Value { has_value: bool },
  CurlyValue,
  AfterCurly,
  Important,
  AfterImportant,
  Accepted,
  Rejected,
}

struct DeclarationCandidate {
  state: DeclarationState,
  components: ComponentValues,
}

impl DeclarationCandidate {
  fn advance<V: LexerVisitor>(&mut self, token: Token, stream: &TokenStream<'_, '_, V>) {
    if matches!(
      self.state,
      DeclarationState::Accepted | DeclarationState::Rejected
    ) {
      return;
    }
    if token.kind.is_trivia() || token.kind == TokenKind::BadComment {
      return;
    }
    if self.components.is_nested() {
      if matches!(token.kind, TokenKind::BadString | TokenKind::BadUrl) {
        self.state = DeclarationState::Rejected;
      } else if token.kind == TokenKind::Eof {
        self.state = DeclarationState::Accepted;
      } else {
        self.components.consume(token.kind);
        if !self.components.is_nested() && matches!(self.state, DeclarationState::CurlyValue) {
          self.state = DeclarationState::AfterCurly;
        }
      }
      return;
    }
    self.state = match self.state {
      DeclarationState::Colon if token.kind == TokenKind::Colon => {
        DeclarationState::Value { has_value: false }
      }
      DeclarationState::Value { .. }
      | DeclarationState::AfterCurly
      | DeclarationState::AfterImportant
        if RecoveryBoundary::at(token.kind, false).is_some() =>
      {
        DeclarationState::Accepted
      }
      DeclarationState::Value { has_value: false } if token.kind == TokenKind::LeftCurlyBracket => {
        self.components.consume(token.kind);
        DeclarationState::CurlyValue
      }
      DeclarationState::Value { .. }
        if !matches!(
          token.kind,
          TokenKind::LeftCurlyBracket
            | TokenKind::RightParenthesis
            | TokenKind::RightSquareBracket
            | TokenKind::BadString
            | TokenKind::BadUrl
        ) =>
      {
        self.components.consume(token.kind);
        DeclarationState::Value { has_value: true }
      }
      DeclarationState::AfterCurly
        if token.kind == TokenKind::Delim && stream.byte_at(token.range.start) == Some(b'!') =>
      {
        DeclarationState::Important
      }
      DeclarationState::Important if token.kind == TokenKind::Ident => {
        let mut normalized = [0; MAX_CSS_KEYWORD_LEN];
        if decode_css_keyword(
          stream.slice_trusted(token.range.start, token.range.end),
          &mut normalized,
        ) == Some("important")
        {
          DeclarationState::AfterImportant
        } else {
          DeclarationState::Rejected
        }
      }
      _ => DeclarationState::Rejected,
    };
  }
}

enum RuleCandidate {
  Prelude(ComponentValues),
  Body,
  Rejected,
}

impl RuleCandidate {
  fn advance(&mut self, kind: TokenKind) {
    let Self::Prelude(components) = self else {
      return;
    };
    if RecoveryBoundary::at(kind, components.is_nested()).is_some() {
      *self = Self::Rejected;
    } else if !components.is_nested() && kind == TokenKind::LeftCurlyBracket {
      // The rule body belongs to the existing block parser once selected.
      *self = Self::Body;
    } else {
      components.consume(kind);
    }
  }
}

/// Parse both candidate prefixes from shared, side-effect-free lookahead.
/// Declarations have priority, as in CSS Syntax's consume-block-contents.
/// https://drafts.csswg.org/css-syntax/#consume-block-contents
/// Once a branch is selected, the semantic parser consumes the buffered
/// tokens rather than re-tokenizing the source or undoing dependencies.
pub(crate) fn parse_block_item<V: LexerVisitor>(
  stream: &mut TokenStream<'_, '_, V>,
  keep_comments: bool,
) -> BlockItemKind {
  let first = stream.peek_nth(0, keep_comments).token;
  if first.kind != TokenKind::Ident {
    // Only identifier-led items are ambiguous with declarations. Keep the
    // existing tolerant selector parser for all other prefixes.
    return BlockItemKind::Rule;
  }
  let custom =
    dashed_ident_name_start(stream.slice_trusted(first.range.start, first.range.end)).is_some();
  let mut declaration = DeclarationCandidate {
    state: DeclarationState::Colon,
    components: ComponentValues::default(),
  };
  let mut rule = RuleCandidate::Prelude(ComponentValues::default());
  rule.advance(first.kind);
  let mut index = 1;
  loop {
    let token = stream.peek_nth(index, keep_comments).token;
    index += 1;
    rule.advance(token.kind);
    declaration.advance(token, stream);
    match declaration.state {
      DeclarationState::Accepted => return BlockItemKind::Declaration,
      DeclarationState::Rejected => match rule {
        RuleCandidate::Body => return BlockItemKind::Rule,
        RuleCandidate::Rejected => return BlockItemKind::Invalid,
        RuleCandidate::Prelude(_) => {}
      },
      DeclarationState::Value { has_value: false } if token.kind == TokenKind::Colon => {
        if custom {
          return BlockItemKind::Declaration;
        }
        // Preserve SIMD scanning for ordinary flat declarations. The semantic
        // parser has not consumed their values, so dependency handling is intact.
        if let Some((value, boundary)) = stream.lexer().scan_flat_value(token.range.end) {
          if boundary != TokenKind::LeftCurlyBracket {
            return BlockItemKind::Declaration;
          }
          if stream
            .slice_trusted(value.start, value.end)
            .bytes()
            .any(|byte| !is_white_space(byte))
          {
            return BlockItemKind::Rule;
          }
        }
      }
      _ => {}
    }
  }
}

/// Leave the synchronization token for the outer parser, which owns statement
/// completion and block exit. Invalid text produces no visitor side effects.
pub(crate) fn recover_block_item<V: LexerVisitor>(
  stream: &mut TokenStream<'_, '_, V>,
  keep_comments: bool,
) {
  let mut components = ComponentValues::default();
  loop {
    let kind = stream.peek(keep_comments).token.kind;
    if RecoveryBoundary::at(kind, components.is_nested()).is_some() {
      return;
    }
    components.consume(kind);
    stream.discard(keep_comments);
  }
}
