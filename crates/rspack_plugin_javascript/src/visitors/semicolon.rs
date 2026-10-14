use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
  Ast, BreakStatement, ContinueStatement, DebuggerStatement, ExportAllDeclaration,
  ExportDefaultDeclaration, ExportNamedDeclaration, ExpressionStatement, GetSpan,
  ImportDeclaration, PropertyDefinition, ReturnStatement, Span, ThrowStatement, UpdateExpression,
  VariableDeclaration, Visit, VisitWith, YieldExpression,
};
use swc_next_ecma_parser::{Token, TokenTag};

/// Finds positions where ECMAScript automatic semicolon insertion terminated a
/// statement. SWC Next spans are zero-based byte offsets, so the collected
/// positions can be used directly as Rspack source ranges.
pub struct InsertedSemicolons<'view, 'ast> {
  ast: &'view Ast<'ast>,
  semicolons: &'view mut FxHashSet<u32>,
  tokens: &'view [Token],
}

impl<'view, 'ast> InsertedSemicolons<'view, 'ast> {
  pub fn new(
    ast: &'view Ast<'ast>,
    semicolons: &'view mut FxHashSet<u32>,
    tokens: &'view [Token],
  ) -> Self {
    Self {
      ast,
      semicolons,
      tokens,
    }
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

impl<'ast> Visit<'ast> for InsertedSemicolons<'_, 'ast> {
  fn ast(&self) -> &Ast<'ast> {
    self.ast
  }

  fn visit_expression_statement(&mut self, node: ExpressionStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_variable_declaration(&mut self, node: VariableDeclaration) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_update_expression(&mut self, node: UpdateExpression) {
    self.semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_continue_statement(&mut self, node: ContinueStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_break_statement(&mut self, node: BreakStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_return_statement(&mut self, node: ReturnStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_throw_statement(&mut self, node: ThrowStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_yield_expression(&mut self, node: YieldExpression) {
    self.post_semi(node.span(self.ast));
    if let Some(argument) = node.argument(self.ast) {
      argument.visit_children_with(self);
    }
  }

  fn visit_import_declaration(&mut self, node: ImportDeclaration) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_debugger_statement(&mut self, node: DebuggerStatement) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }

  fn visit_property_definition(&mut self, node: PropertyDefinition) {
    self.post_semi(node.span(self.ast));
    node.visit_children_with(self);
  }
}
