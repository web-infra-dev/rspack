//! Structural block-item classification before dependency collection.

use smallvec::SmallVec;

use crate::{
  Pos, Token, TokenKind,
  css_syntax::{MAX_CSS_KEYWORD_LEN, decode_css_keyword},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
  fn advance(&mut self, token: Token, source: &[u8]) {
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
      if token.kind == TokenKind::Eof {
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
      DeclarationState::Value { .. } if token.kind != TokenKind::LeftCurlyBracket => {
        self.components.consume(token.kind);
        DeclarationState::Value { has_value: true }
      }
      DeclarationState::AfterCurly
        if token.kind == TokenKind::Delim
          && source.get(token.range.start as usize).copied() == Some(b'!') =>
      {
        DeclarationState::Important
      }
      DeclarationState::Important if token.kind == TokenKind::Ident => {
        let mut normalized = [0; MAX_CSS_KEYWORD_LEN];
        if decode_css_keyword(
          std::str::from_utf8(&source[token.range.start as usize..token.range.end as usize])
            .expect("token boundaries are UTF-8 boundaries"),
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
  Prelude,
  Body(Pos),
  Rejected,
}

impl RuleCandidate {
  fn advance(&mut self, token: Token, nested: bool) {
    let Self::Prelude = self else {
      return;
    };
    if RecoveryBoundary::at(token.kind, nested).is_some() {
      *self = Self::Rejected;
    } else if !nested && token.kind == TokenKind::LeftCurlyBracket {
      // The rule body belongs to the existing block parser once selected.
      *self = Self::Body(token.range.start);
    }
  }
}

/// Classify the ambiguous identifier prefix without collecting dependencies.
/// The selected grammar is parsed once afterwards, so selector modes, warnings
/// and declaration dependencies never need speculative state or rollback.
pub(crate) fn classify_block_item(
  mut next: impl FnMut() -> Token,
  source: &[u8],
  custom: bool,
) -> (BlockItemKind, Pos) {
  let mut declaration = DeclarationCandidate {
    state: DeclarationState::Colon,
    components: ComponentValues::default(),
  };
  let mut rule = RuleCandidate::Prelude;
  loop {
    let token = next();
    rule.advance(token, declaration.components.is_nested());
    declaration.advance(token, source);
    if custom && matches!(declaration.state, DeclarationState::Value { .. }) {
      return (BlockItemKind::Declaration, token.range.end);
    }
    match declaration.state {
      DeclarationState::Accepted => return (BlockItemKind::Declaration, token.range.start),
      DeclarationState::Rejected => match rule {
        RuleCandidate::Body(start) => return (BlockItemKind::Rule, start),
        RuleCandidate::Rejected => return (BlockItemKind::Invalid, token.range.start),
        RuleCandidate::Prelude => declaration.components.consume(token.kind),
      },
      _ => {}
    }
  }
}
