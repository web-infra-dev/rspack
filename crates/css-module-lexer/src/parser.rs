//! Block-item candidates and their shared error-recovery boundaries.

use smallvec::SmallVec;

use crate::{
  Pos, Token, TokenKind,
  css_syntax::{MAX_CSS_KEYWORD_LEN, decode_css_keyword},
  lexer::TokenStream,
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

/// A grammar transition, published by token consumption and raw scanning.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockItemProgress {
  #[default]
  Continue,
  ResolveRule,
  Finished(BlockItemKind),
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
  Body,
  Rejected,
}

impl RuleCandidate {
  fn advance(&mut self, kind: TokenKind, nested: bool) {
    let Self::Prelude = self else {
      return;
    };
    if RecoveryBoundary::at(kind, nested).is_some() {
      *self = Self::Rejected;
    } else if !nested && kind == TokenKind::LeftCurlyBracket {
      // The rule body belongs to the existing block parser once selected.
      *self = Self::Body;
    }
  }
}

/// Candidates advance beside semantic parsing, including its raw value scanner.
/// Only semantically significant tokens are retained for the rule alternative;
/// accepting a declaration commits effects without replaying its tokens.
/// https://drafts.csswg.org/css-syntax/#consume-block-contents
pub(crate) struct BlockItemCandidates {
  declaration: DeclarationCandidate,
  rule: RuleCandidate,
  custom: bool,
  first: bool,
  has_mode: bool,
  pub(crate) progress: BlockItemProgress,
  retain_next_atom: bool,
  retain_current_atom: bool,
  preserve_next_ident: bool,
  pub(crate) reuse_declaration_effects: bool,
  pub(crate) flat_probe_start: Option<Pos>,
  pub(crate) collect_dashed: bool,
}

impl BlockItemCandidates {
  pub(crate) fn new(custom: bool, has_mode: bool) -> Self {
    Self {
      declaration: DeclarationCandidate {
        state: DeclarationState::Colon,
        components: ComponentValues::default(),
      },
      rule: RuleCandidate::Prelude,
      custom,
      first: true,
      has_mode,
      progress: BlockItemProgress::Continue,
      retain_next_atom: true,
      retain_current_atom: true,
      preserve_next_ident: false,
      reuse_declaration_effects: !has_mode,
      flat_probe_start: None,
      collect_dashed: false,
    }
  }

  #[inline]
  pub(crate) fn observe(&mut self, token: Token, source: &[u8]) {
    if matches!(self.progress, BlockItemProgress::Finished(_)) {
      return;
    }
    if !token.kind.is_trivia() && token.kind != TokenKind::BadComment {
      self.retain_current_atom = self.retain_next_atom;
      // Selector helpers peek immediately after names, dots and colons.
      // Keep that token even when it otherwise has no semantic effect.
      self.retain_next_atom = matches!(
        token.kind,
        TokenKind::Ident | TokenKind::Function | TokenKind::Colon | TokenKind::Delim
      );
    }
    self.preserve_next_ident = token.kind == TokenKind::Colon
      || (token.kind == TokenKind::Delim
        && source
          .get(token.range.start as usize)
          .is_some_and(|byte| matches!(byte, b'.' | b'#')));
    // Ordinary atoms cannot select a rule or change nesting. The semantic
    // scanner already classified them; only the first value atom matters.
    if !self.first
      && matches!(
        token.kind,
        TokenKind::Ident
          | TokenKind::Number
          | TokenKind::Dimension
          | TokenKind::Percentage
          | TokenKind::Hash
          | TokenKind::IdHash
          | TokenKind::Url
          | TokenKind::QuotedString
          | TokenKind::Delim
      )
    {
      if let DeclarationState::Value { has_value } = &mut self.declaration.state {
        *has_value = true;
        return;
      }
      if self.declaration.components.is_nested() {
        return;
      }
    }
    if token.kind == TokenKind::LeftCurlyBracket
      && !self.declaration.components.is_nested()
      && matches!(
        self.declaration.state,
        DeclarationState::Value { has_value: false }
      )
    {
      // A sole curly value may later become a rule body. Its contents then
      // need block semantics rather than declaration-value semantics.
      self.reuse_declaration_effects = false;
    }
    self
      .rule
      .advance(token.kind, self.declaration.components.is_nested());
    if self.first {
      self.first = false;
      return;
    }
    self.declaration.advance(token, source);
    if self.declaration_rejected() && matches!(self.rule, RuleCandidate::Prelude) {
      self.declaration.components.consume(token.kind);
    }
    self.progress = match self.declaration.state {
      DeclarationState::Accepted => BlockItemProgress::Finished(BlockItemKind::Declaration),
      DeclarationState::Rejected => match self.rule {
        RuleCandidate::Body => BlockItemProgress::Finished(BlockItemKind::Rule),
        RuleCandidate::Rejected => BlockItemProgress::Finished(BlockItemKind::Invalid),
        RuleCandidate::Prelude => BlockItemProgress::ResolveRule,
      },
      DeclarationState::Value { .. } if self.custom && token.kind == TokenKind::Colon => {
        BlockItemProgress::Finished(BlockItemKind::Declaration)
      }
      _ => BlockItemProgress::Continue,
    };
  }

  /// The raw scanner has already proved this atom cannot change structure.
  /// Reuse that proof rather than dispatching through both grammar states.
  #[inline]
  pub(crate) fn opaque_value(&mut self) {
    if let DeclarationState::Value { has_value } = &mut self.declaration.state {
      *has_value = true;
    }
    self.preserve_next_ident = false;
    self.retain_next_atom = false;
  }

  #[inline]
  pub(crate) fn is_nested(&self) -> bool {
    self.declaration.components.is_nested()
  }

  #[inline]
  pub(crate) fn expects(&self, kind: TokenKind) -> bool {
    self.declaration.components.expects(kind)
  }

  #[inline]
  pub(crate) fn retain_rule_event(&self, kind: TokenKind) -> bool {
    if self.declaration_rejected() || matches!(self.rule, RuleCandidate::Body) {
      // The selected rule body is parsed as block contents, not as a selector.
      return true;
    }
    self.has_mode
      && (self.retain_current_atom
        || !matches!(
          kind,
          TokenKind::Number
            | TokenKind::Percentage
            | TokenKind::Dimension
            | TokenKind::IncludeMatch
            | TokenKind::DashMatch
            | TokenKind::PrefixMatch
            | TokenKind::SuffixMatch
            | TokenKind::SubstringMatch
        ))
  }

  pub(crate) fn declaration_rejected(&self) -> bool {
    matches!(self.declaration.state, DeclarationState::Rejected)
  }

  pub(crate) fn can_scan_value(&self) -> bool {
    self.progress == BlockItemProgress::Continue
      && !matches!(self.rule, RuleCandidate::Body)
      && matches!(
        self.declaration.state,
        DeclarationState::Value { .. } | DeclarationState::CurlyValue
      )
  }

  pub(crate) fn preserve_selector_tokens(&self) -> bool {
    self.has_mode && self.progress == BlockItemProgress::Continue
  }

  #[inline]
  pub(crate) fn preserve_ident(&self) -> bool {
    self.preserve_selector_tokens() && self.preserve_next_ident
  }
}

/// Once the declaration fails, finish the rule candidate on the same stream.
/// No semantic effects are applied to this rejected declaration suffix.
pub(crate) fn resolve_block_item<V: crate::LexerVisitor>(
  stream: &mut TokenStream<'_, '_, V>,
  keep_comments: bool,
) {
  while stream.block_item_progress() == BlockItemProgress::ResolveRule {
    stream.next(keep_comments);
  }
}
