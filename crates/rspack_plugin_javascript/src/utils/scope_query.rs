use rustc_hash::FxHashMap;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{Visit, VisitWith};

struct ScopeScanner {
  /// ctxt -> ctxt parnet
  ctxt_parent_map: FxHashMap<SyntaxContext, SyntaxContext>,
  /// ctxt -> related scope kind
  ctxt_scope_kind_map: FxHashMap<SyntaxContext, ScopeKind>,
  stack: Vec<ScopeKind>,
  unresolved_ctxt: SyntaxContext,
  top_level_ctxt: SyntaxContext,
}

impl ScopeScanner {}

impl Visit for ScopeScanner {
  noop_visit_type!();

  fn visit_ident(&mut self, n: &Ident) {
    let ctxt = n.span.ctxt;
    if ctxt == self.unresolved_ctxt || ctxt == self.top_level_ctxt {
      return;
    }
  }

  fn visit_arrow_expr(&mut self, n: &ArrowExpr) {
    self.with_scope(ScopeKind::Fn, |v| {
      let old = v.is_param;
      v.is_param = true;
      n.params.visit_with(v);
      v.is_param = old;

      match &mut *n.body {
        BlockStmtOrExpr::BlockStmt(b) => {
          b.visit_children_with(v);
        }
        BlockStmtOrExpr::Expr(b) => {
          b.visit_with(v);
        }
      }
    });
  }

  fn visit_block_stmt(&mut self, n: &BlockStmt) {
    self.with_scope(ScopeKind::Block, |v| {
      n.visit_children_with(v);
    });
  }

  fn visit_catch_clause(&mut self, n: &CatchClause) {
    let old_is_param = self.is_param;
    self.is_param = true;

    let old_var_decl_kind = self.var_decl_kind;
    self.var_decl_kind = None;

    n.visit_children_with(self);

    self.var_decl_kind = old_var_decl_kind;
    self.is_param = old_is_param;
  }

  fn visit_constructor(&mut self, n: &Constructor) {
    self.with_scope(ScopeKind::Fn, |v| {
      n.params.visit_with(v);

      if let Some(body) = &mut n.body {
        body.visit_children_with(v);
      }
    });
  }

  fn visit_for_in_stmt(&mut self, n: &ForInStmt) {
    n.right.visit_with(self);

    match &n.left {
      ForHead::VarDecl(v)
        if matches!(
          &**v,
          VarDecl {
            kind: VarDeclKind::Let | VarDeclKind::Const,
            ..
          }
        ) =>
      {
        self.with_scope(ScopeKind::Block, |v| {
          n.left.visit_with(v);
          n.body.visit_with(v);
        });
      }
      _ => {
        n.left.visit_with(self);
        n.body.visit_with(self);
      }
    }
  }

  fn visit_for_of_stmt(&mut self, n: &ForOfStmt) {
    n.right.visit_with(self);

    match &n.left {
      ForHead::VarDecl(v)
        if matches!(
          &**v,
          VarDecl {
            kind: VarDeclKind::Let | VarDeclKind::Const,
            ..
          }
        ) =>
      {
        self.with_scope(ScopeKind::Block, |v| {
          n.left.visit_with(v);
          n.body.visit_with(v);
        });
      }
      _ => {
        n.left.visit_with(self);
        n.body.visit_with(self);
      }
    }
  }

  fn visit_for_stmt(&mut self, n: &ForStmt) {
    match &n.init {
      Some(VarDeclOrExpr::VarDecl(v))
        if matches!(
          &**v,
          VarDecl {
            kind: VarDeclKind::Let | VarDeclKind::Const,
            ..
          }
        ) =>
      {
        self.with_scope(ScopeKind::Block, |v| {
          n.init.visit_with(v);
          n.update.visit_with(v);
          n.test.visit_with(v);

          n.body.visit_with(v);
        });
      }
      _ => {
        n.init.visit_with(self);
        n.update.visit_with(self);
        n.test.visit_with(self);

        n.body.visit_with(self);
      }
    }
  }

  fn visit_function(&mut self, n: &Function) {
    n.decorators.visit_with(self);

    self.with_scope(ScopeKind::Fn, |v| {
      n.params.visit_with(v);

      if let Some(body) = &n.body {
        body.visit_children_with(v);
      }
    });
  }

  fn visit_module(&mut self, n: &Module) {
    self.handle_program(n)
  }

  fn visit_script(&mut self, n: &Script) {
    self.handle_program(n)
  }
}
