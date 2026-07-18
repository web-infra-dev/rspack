use rspack_core::{
  ConcatenationScopeIdent, ConcatenationScopeIdentKind, ConcatenationScopeSnapshot,
};
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use swc_experimental_allocator::atom::Atom as AstAtom;
use swc_experimental_ecma_ast::{
  BreakStmt, ClassExpr, ClassMember, ContinueStmt, DebuggerStmt, ExportAll, ExportDefaultExpr,
  ExprStmt, Ident, ImportDecl, NamedExport, ObjectPatProp, Prop, ReturnStmt, Span, ThrowStmt,
  UpdateExpr, VarDecl, Visit, VisitWith, YieldExpr,
};
use swc_experimental_ecma_parser::unstable::{Token, TokenAndSpan};
use swc_experimental_ecma_semantic::resolver::Semantic;

#[derive(Clone, Copy)]
struct PendingConcatenationScopeIdent {
  range: rspack_core::DependencyRange,
  shorthand: bool,
}

pub(crate) struct PendingConcatenationScopeSnapshot<'ast> {
  module_ctxt: u32,
  global_ctxt: u32,
  top_level_idents: SmallVec<[PendingConcatenationScopeIdent; 8]>,
  global_idents: SmallVec<[PendingConcatenationScopeIdent; 4]>,
  used_names: SmallVec<[(AstAtom<'ast>, rspack_core::DependencyRange); 8]>,
}

impl<'ast> PendingConcatenationScopeSnapshot<'ast> {
  pub(crate) fn into_snapshot(self) -> ConcatenationScopeSnapshot {
    let mut idents = Vec::with_capacity(
      self.top_level_idents.len() + self.global_idents.len() + self.used_names.len(),
    );
    idents.extend(
      self
        .top_level_idents
        .into_iter()
        .map(|ident| ConcatenationScopeIdent {
          range: ident.range,
          shorthand: ident.shorthand,
          kind: ConcatenationScopeIdentKind::TopLevel,
        }),
    );
    idents.extend(
      self
        .global_idents
        .into_iter()
        .map(|ident| ConcatenationScopeIdent {
          range: ident.range,
          shorthand: ident.shorthand,
          kind: ConcatenationScopeIdentKind::Global,
        }),
    );
    let mut used_names = SmallVec::<[AstAtom<'ast>; 8]>::new();
    idents.extend(self.used_names.into_iter().filter_map(|(symbol, range)| {
      if used_names.contains(&symbol) {
        return None;
      }
      used_names.push(symbol);
      Some(ConcatenationScopeIdent {
        range,
        shorthand: false,
        kind: ConcatenationScopeIdentKind::UsedName,
      })
    }));
    ConcatenationScopeSnapshot {
      module_ctxt: self.module_ctxt,
      global_ctxt: self.global_ctxt,
      idents,
    }
  }
}

struct ConcatenationScopeSnapshotCollector<'semantic, 'ast> {
  semantic: &'semantic Semantic,
  top_level_idents: SmallVec<[PendingConcatenationScopeIdent; 8]>,
  global_idents: SmallVec<[PendingConcatenationScopeIdent; 4]>,
  used_names: SmallVec<[(AstAtom<'ast>, rspack_core::DependencyRange); 8]>,
}

impl<'ast> ConcatenationScopeSnapshotCollector<'_, 'ast> {
  #[inline]
  fn add_ident(&mut self, ident: &Ident<'ast>, shorthand: bool, class_expr_with_ident: bool) {
    if ident.symbol_id.get().is_none() {
      return;
    }
    let scope = self.semantic.node_scope(ident);
    let range = ident.span.into();
    if scope == self.semantic.unresolved_scope_id() {
      self
        .global_idents
        .push(PendingConcatenationScopeIdent { range, shorthand });
    } else if class_expr_with_ident || scope != self.semantic.top_level_scope_id() {
      self.used_names.push((ident.sym, range));
    } else {
      self
        .top_level_idents
        .push(PendingConcatenationScopeIdent { range, shorthand });
    }
  }

  fn into_pending(self) -> PendingConcatenationScopeSnapshot<'ast> {
    PendingConcatenationScopeSnapshot {
      module_ctxt: self.semantic.top_level_scope_id().raw(),
      global_ctxt: self.semantic.unresolved_scope_id().raw(),
      top_level_idents: self.top_level_idents,
      global_idents: self.global_idents,
      used_names: self.used_names,
    }
  }
}

/// Auto inserted semicolon
/// See: https://262.ecma-international.org/7.0/#sec-rules-of-automatic-semicolon-insertion
pub struct InsertedSemicolons<'a, 'ast> {
  semicolons: &'a mut FxHashSet<u32>,
  tokens: &'a [TokenAndSpan],
  concatenation_scope: Option<ConcatenationScopeSnapshotCollector<'a, 'ast>>,
}

impl<'a, 'ast> InsertedSemicolons<'a, 'ast> {
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
      top_level_idents: SmallVec::new(),
      global_idents: SmallVec::new(),
      used_names: SmallVec::new(),
    });
    self
  }

  pub(crate) fn into_concatenation_scope_snapshot(
    self,
  ) -> Option<PendingConcatenationScopeSnapshot<'ast>> {
    self
      .concatenation_scope
      .map(ConcatenationScopeSnapshotCollector::into_pending)
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

impl<'ast> Visit<'ast> for InsertedSemicolons<'_, 'ast> {
  fn visit_ident(&mut self, ident: &Ident<'ast>) {
    if let Some(collector) = &mut self.concatenation_scope {
      collector.add_ident(ident, false, false);
    }
  }

  fn visit_object_pat_prop(&mut self, prop: &ObjectPatProp<'ast>) {
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

  fn visit_prop(&mut self, prop: &Prop<'ast>) {
    if let Prop::Shorthand(ident) = prop {
      if let Some(collector) = &mut self.concatenation_scope {
        collector.add_ident(ident, true, false);
      }
    } else {
      prop.visit_children_with(self);
    }
  }

  fn visit_class_expr(&mut self, class_expr: &ClassExpr<'ast>) {
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

  fn visit_expr_stmt(&mut self, n: &ExprStmt<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_var_decl(&mut self, n: &VarDecl<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_update_expr(&mut self, n: &UpdateExpr<'ast>) {
    self.semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_continue_stmt(&mut self, n: &ContinueStmt<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_break_stmt(&mut self, n: &BreakStmt<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_return_stmt(&mut self, n: &ReturnStmt<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_throw_stmt(&mut self, n: &ThrowStmt<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_yield_expr(&mut self, n: &YieldExpr<'ast>) {
    self.post_semi(&n.span);
    if let Some(arg) = &n.arg {
      arg.visit_children_with(self)
    }
  }

  fn visit_import_decl(&mut self, n: &ImportDecl<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_named_export(&mut self, n: &NamedExport<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_export_default_expr(&mut self, n: &ExportDefaultExpr<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_export_all(&mut self, n: &ExportAll<'ast>) {
    self.post_semi(&n.span);
    n.visit_children_with(self)
  }

  fn visit_debugger_stmt(&mut self, n: &DebuggerStmt) {
    self.post_semi(&n.span);
    n.visit_children_with(self);
  }

  fn visit_class_member(&mut self, n: &ClassMember<'ast>) {
    match n {
      ClassMember::ClassProp(prop) => self.post_semi(&prop.span),
      ClassMember::PrivateProp(prop) => self.post_semi(&prop.span),
      _ => {}
    };
    n.visit_children_with(self);
  }
}
