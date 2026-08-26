use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{Ast, NodeId, NodeKind, Span};
use swc_next_ecma_parser::{Token, TokenTag};

/// Finds positions where ECMAScript automatic semicolon insertion terminated a
/// statement. SWC Next spans are zero-based byte offsets, so the collected
/// positions can be used directly as Rspack source ranges.
struct InsertedSemicolons<'view> {
  semicolons: &'view mut FxHashSet<u32>,
  tokens: &'view [Token],
}

impl<'view> InsertedSemicolons<'view> {
  fn new(semicolons: &'view mut FxHashSet<u32>, tokens: &'view [Token]) -> Self {
    Self { semicolons, tokens }
  }

  #[inline]
  fn curr_token(&self, span: Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|token| token.span.start.cmp(&span.start))
      .ok()
  }

  #[inline]
  fn next_token(&self, span: Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|token| token.span.end.cmp(&span.end))
      .ok()
      .map(|index| index + 1)
  }

  #[inline]
  fn can_insert_semi(&self, token_index: usize) -> bool {
    if token_index == self.tokens.len() {
      return true;
    }
    let token = self.tokens[token_index];
    token.tag == TokenTag::RightBrace || token.has_line_terminator_before()
  }

  #[inline]
  fn semi(&mut self, span: Span) {
    let Some(index) = self.curr_token(span) else {
      return;
    };
    if index > 0 {
      let previous = self.tokens[index - 1];
      if previous.tag != TokenTag::Semicolon && self.can_insert_semi(index) {
        self.semicolons.insert(previous.span.end);
      }
    }
  }

  #[inline]
  fn post_semi(&mut self, span: Span) {
    let Some(index) = self.next_token(span) else {
      return;
    };
    if index > 0 {
      let previous = self.tokens[index - 1];
      if previous.tag != TokenTag::Semicolon && self.can_insert_semi(index) {
        self.semicolons.insert(previous.span.end);
      }
    }
  }
}

pub fn collect(ast: &Ast<'_>, semicolons: &mut FxHashSet<u32>, tokens: &[Token]) {
  let mut collector = InsertedSemicolons::new(semicolons, tokens);
  for index in 0..ast.node_count() {
    let node = NodeId::from_raw_unchecked(index as u32);
    match ast.node_kind(node) {
      NodeKind::UpdateExpression => collector.semi(ast.span(node)),
      NodeKind::ExpressionStatement
      | NodeKind::VariableDeclaration
      | NodeKind::ContinueStatement
      | NodeKind::BreakStatement
      | NodeKind::ReturnStatement
      | NodeKind::ThrowStatement
      | NodeKind::YieldExpression
      | NodeKind::ImportDeclaration
      | NodeKind::ExportNamedDeclaration
      | NodeKind::ExportDefaultDeclaration
      | NodeKind::ExportAllDeclaration
      | NodeKind::DebuggerStatement
      | NodeKind::PropertyDefinition => collector.post_semi(ast.span(node)),
      _ => {}
    }
  }
}
