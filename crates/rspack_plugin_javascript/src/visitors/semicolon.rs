use rspack_core::{ConcatenationScopeIdent, ConcatenationScopeSnapshot};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{
  BreakStmt, ClassExpr, ClassMember, ContinueStmt, DebuggerStmt, ExportAll, ExportDefaultExpr,
  ExprStmt, Ident, ImportDecl, NamedExport, ObjectPatProp, Prop, ReturnStmt, Span, ThrowStmt,
  UpdateExpr, VarDecl, Visit, VisitWith, YieldExpr,
};
use swc_experimental_ecma_parser::unstable::{Token, TokenAndSpan};
use swc_experimental_ecma_semantic::resolver::Semantic;

struct ConcatenationScopeSnapshotCollector<'a> {
  semantic: &'a Semantic,
  top_level_idents: Vec<ConcatenationScopeIdent>,
  global_idents: Vec<ConcatenationScopeIdent>,
  used_names: FxHashSet<Atom>,
}

impl ConcatenationScopeSnapshotCollector<'_> {
  #[inline]
  fn add_ident(&mut self, ident: &Ident<'_>, shorthand: bool, class_expr_with_ident: bool) {
    if ident.symbol_id.get().is_none() {
      return;
    }
    let symbol = Atom::from(ident.sym.as_str());
    let scope = self.semantic.node_scope(ident);
    if scope == self.semantic.unresolved_scope_id() {
      self.global_idents.push(ConcatenationScopeIdent {
        symbol: symbol.clone(),
        range: ident.span.into(),
        shorthand,
      });
      self.used_names.insert(symbol);
    } else if class_expr_with_ident || scope != self.semantic.top_level_scope_id() {
      self.used_names.insert(symbol);
    } else {
      self.top_level_idents.push(ConcatenationScopeIdent {
        symbol,
        range: ident.span.into(),
        shorthand,
      });
    }
  }

  fn into_snapshot(self) -> ConcatenationScopeSnapshot {
    ConcatenationScopeSnapshot {
      module_ctxt: self.semantic.top_level_scope_id().raw(),
      global_ctxt: self.semantic.unresolved_scope_id().raw(),
      top_level_idents: self.top_level_idents,
      global_idents: self.global_idents,
      used_names: self.used_names.into_iter().collect(),
    }
  }
}

/// Auto inserted semicolon
/// See: https://262.ecma-international.org/7.0/#sec-rules-of-automatic-semicolon-insertion
pub struct InsertedSemicolons<'a> {
  semicolons: &'a mut FxHashSet<u32>,
  tokens: &'a [TokenAndSpan],
  concatenation_scope: Option<ConcatenationScopeSnapshotCollector<'a>>,
}

impl<'a> InsertedSemicolons<'a> {
  pub fn new(semicolons: &'a mut FxHashSet<u32>, tokens: &'a [TokenAndSpan]) -> Self {
    Self {
      semicolons,
      tokens,
      concatenation_scope: None,
    }
  }

  pub fn with_concatenation_scope(mut self, semantic: &'a Semantic) -> Self {
    self.concatenation_scope = Some(ConcatenationScopeSnapshotCollector {
      semantic,
      top_level_idents: Vec::new(),
      global_idents: Vec::new(),
      used_names: FxHashSet::default(),
    });
    self
  }

  pub fn into_concatenation_scope_snapshot(self) -> Option<ConcatenationScopeSnapshot> {
    self
      .concatenation_scope
      .map(ConcatenationScopeSnapshotCollector::into_snapshot)
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
  fn visit_ident(&mut self, ident: &Ident<'a>) {
    if let Some(collector) = &mut self.concatenation_scope {
      collector.add_ident(ident, false, false);
    }
  }

  fn visit_object_pat_prop(&mut self, prop: &ObjectPatProp<'a>) {
    match prop {
      ObjectPatProp::Assign(assign) => {
        if let Some(collector) = &mut self.concatenation_scope {
          collector.add_ident(&assign.key.id, true, false);
        }
        assign.value.visit_with(self);
      }
      ObjectPatProp::KeyValue(_) | ObjectPatProp::Rest(_) => prop.visit_children_with(self),
    }
  }

  fn visit_prop(&mut self, prop: &Prop<'a>) {
    if let Prop::Shorthand(ident) = prop {
      if let Some(collector) = &mut self.concatenation_scope {
        collector.add_ident(ident, true, false);
      }
    } else {
      prop.visit_children_with(self);
    }
  }

  fn visit_class_expr(&mut self, class_expr: &ClassExpr<'a>) {
    if let Some(ident) = &class_expr.ident
      && class_expr.class.super_class.is_some()
    {
      if let Some(collector) = &mut self.concatenation_scope {
        collector.add_ident(ident, false, true);
      }
      class_expr.class.visit_with(self);
    } else {
      class_expr.visit_children_with(self);
    }
  }

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
