use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{
  BreakStmt, ClassMember, ContinueStmt, DebuggerStmt, ExportAll, ExportDefaultExpr, ExprStmt,
  ImportDecl, NamedExport, ReturnStmt, Span, ThrowStmt, UpdateExpr, VarDecl, Visit, VisitWith,
  YieldExpr,
};
use swc_experimental_ecma_parser::unstable::{Token, TokenAndSpan};

/// Auto inserted semicolon
/// See: https://262.ecma-international.org/7.0/#sec-rules-of-automatic-semicolon-insertion
pub struct InsertedSemicolons<'a> {
  semicolons: &'a mut FxHashSet<u32>,
  tokens: &'a [TokenAndSpan],
}

impl<'a> InsertedSemicolons<'a> {
  pub fn new(semicolons: &'a mut FxHashSet<u32>, tokens: &'a [TokenAndSpan]) -> Self {
    Self { semicolons, tokens }
  }

  /// Find the starting token of this span.
  /// Returns [None] if there's no token is found.
  /// This might be happen if there's an error in the lexer.
  #[inline]
  fn curr_token(&self, span: &Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|t| t.span.start.cmp(&span.start))
      .ok()
  }

  /// Find the next token of this span.
  /// Returns [None] if there's no token is found.
  /// This might be happen if there's an error in the lexer.
  #[inline]
  fn next_token(&self, span: &Span) -> Option<usize> {
    self
      .tokens
      .binary_search_by(|t| t.span.end.cmp(&span.end))
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
        self.semicolons.insert(prev.span.end);
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
        self.semicolons.insert(prev.span.end);
      }
    }
  }
}

impl<'a> Visit<'a> for InsertedSemicolons<'_> {
  fn visit_expr_stmt(&mut self, n: &ExprStmt<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_var_decl(&mut self, n: &VarDecl<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_update_expr(&mut self, n: &UpdateExpr<'a>) {
    self.semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_continue_stmt(&mut self, n: &ContinueStmt<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_break_stmt(&mut self, n: &BreakStmt<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_return_stmt(&mut self, n: &ReturnStmt<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_throw_stmt(&mut self, n: &ThrowStmt<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_yield_expr(&mut self, n: &YieldExpr<'a>) {
    self.post_semi(&n.span);
    if let Some(arg) = &n.arg {
      arg.visit_children_with(self)
    }
  }

  fn visit_import_decl(&mut self, n: &ImportDecl<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_named_export(&mut self, n: &NamedExport<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_export_default_expr(&mut self, n: &ExportDefaultExpr<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_export_all(&mut self, n: &ExportAll<'a>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_debugger_stmt(&mut self, n: &DebuggerStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self);
  }

  fn visit_class_member(&mut self, n: &ClassMember<'a>) {
    match n {
      ClassMember::ClassProp(prop) => self.post_semi(&prop.span),
      ClassMember::PrivateProp(prop) => self.post_semi(&prop.span),
      _ => {}
    };
    n.visit_children_with(self);
  }
}
