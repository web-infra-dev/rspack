use rustc_hash::FxHashSet;
use swc_core::{
  common::{BytePos, Span, Spanned},
  ecma::{
    parser::token::{Token, TokenAndSpan},
    visit::{Visit, VisitWith},
  },
};

/// Auto inserted semicolon
/// See: https://262.ecma-international.org/7.0/#sec-rules-of-automatic-semicolon-insertion
pub(crate) struct InsertedSemicolons<'a> {
  pub(crate) semicolons: &'a mut FxHashSet<BytePos>,
  pub(crate) tokens: &'a Vec<TokenAndSpan>,
}

impl<'a> InsertedSemicolons<'a> {
  /// Find the starting token of this span.
  /// Returns [None] if there's no token is found.
  /// This might be happen if there's an error in the lexer.
  #[inline]
  fn curr_token(&self, span: &Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|t| t.span.lo.cmp(&span.lo))
      .ok()
  }

  /// Find the next token of this span.
  /// Returns [None] if there's no token is found.
  /// This might be happen if there's an error in the lexer.
  #[inline]
  fn next_token(&self, span: &Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|t| t.span.hi.cmp(&span.hi))
      .ok()
      .map(|i| i + 1)
  }

  #[inline]
  fn can_insert_semi(&self, token_index: usize) -> bool {
    if token_index == self.tokens.len() {
      // eof
      return true;
    }
    let token = &self.tokens[token_index];
    matches!(token.token, Token::RBrace) || token.had_line_break
  }

  #[inline]
  fn semi(&mut self, span: &Span) {
    let Some(index) = self.curr_token(span) else {
      return;
    };
    if index > 0 {
      let prev = &self.tokens[index - 1];
      if !matches!(prev.token, Token::Semi) && self.can_insert_semi(index) {
        self.semicolons.insert(prev.span_hi());
      }
    }
  }

  #[inline]
  fn post_semi(&mut self, span: &Span) {
    let Some(index) = self.next_token(span) else {
      return;
    };
    if index > 0 {
      let prev = &self.tokens[index - 1];
      if !matches!(prev.token, Token::Semi) && self.can_insert_semi(index) {
        self.semicolons.insert(prev.span_hi());
      }
    }
  }
}

impl<'a> Visit for InsertedSemicolons<'a> {
  fn visit_expr_stmt(&mut self, n: &swc_core::ecma::ast::ExprStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_var_decl(&mut self, n: &swc_core::ecma::ast::VarDecl) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_update_expr(&mut self, n: &swc_core::ecma::ast::UpdateExpr) {
    self.semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_continue_stmt(&mut self, n: &swc_core::ecma::ast::ContinueStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_break_stmt(&mut self, n: &swc_core::ecma::ast::BreakStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_return_stmt(&mut self, n: &swc_core::ecma::ast::ReturnStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_throw_stmt(&mut self, n: &swc_core::ecma::ast::ThrowStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_yield_expr(&mut self, n: &swc_core::ecma::ast::YieldExpr) {
    self.post_semi(&n.span);
    if let Some(arg) = &n.arg {
      arg.visit_children_with(self)
    }
  }
}
