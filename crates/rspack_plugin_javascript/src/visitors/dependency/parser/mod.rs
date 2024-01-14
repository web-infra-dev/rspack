#![allow(unused)]

mod walk;
mod walk_block_pre;
mod walk_pre;

use std::borrow::Cow;

use rspack_core::{BoxDependency, DependencyTemplate};
use swc_core::ecma::ast::{ArrayPat, AssignPat, BlockStmt, ObjectPat, ObjectPatProp, Pat, Program};
use swc_core::ecma::ast::{Ident, Lit, RestPat};

use crate::visitors::scope_info::{ScopeInfoDB, ScopeInfoId};

pub struct JavascriptParser<'parser> {
  pub(crate) dependencies: &'parser mut Vec<BoxDependency>,
  pub(crate) presentational_dependencies: &'parser mut Vec<Box<dyn DependencyTemplate>>,
  pub(super) definitions_db: ScopeInfoDB,
  // ===== scope info =======
  pub(crate) in_try: bool,
  pub(crate) in_if: bool,
  pub(crate) in_short_hand: bool,
  pub(super) definitions: ScopeInfoId,
}

impl<'parser> JavascriptParser<'parser> {
  pub fn new(
    dependencies: &'parser mut Vec<BoxDependency>,
    presentational_dependencies: &'parser mut Vec<Box<dyn DependencyTemplate>>,
  ) -> Self {
    let mut db = ScopeInfoDB::new();
    Self {
      dependencies,
      presentational_dependencies,
      in_try: false,
      in_if: false,
      in_short_hand: false,
      definitions: db.create(),
      definitions_db: db,
    }
  }

  fn define_variable(&mut self, name: String) {
    if let Some(id) = self.definitions_db.get(&self.definitions, &name)
      && self.definitions == id
    {
      return;
    }
    self.definitions_db.set(self.definitions, name)
  }

  fn undefined_variable(&mut self, name: String) {
    self.definitions_db.delete(self.definitions, name)
  }

  fn enter_ident<F>(&mut self, ident: &Ident, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident),
  {
    // TODO: add hooks here;
    on_ident(self, ident);
  }

  fn enter_array_pattern<F>(&mut self, array_pat: &ArrayPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    array_pat
      .elems
      .iter()
      .flatten()
      .for_each(|ele| self.enter_pattern(Cow::Borrowed(ele), on_ident));
  }

  fn enter_assignment_pattern<F>(&mut self, assign: &AssignPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    self.enter_pattern(Cow::Borrowed(&assign.left), on_ident);
  }

  fn enter_object_pattern<F>(&mut self, obj: &ObjectPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    for prop in &obj.props {
      match prop {
        ObjectPatProp::KeyValue(kv) => self.enter_pattern(Cow::Borrowed(&kv.value), on_ident),
        ObjectPatProp::Assign(assign) => self.enter_ident(&assign.key, on_ident),
        ObjectPatProp::Rest(rest) => self.enter_rest_pattern(rest, on_ident),
      }
    }
  }

  fn enter_rest_pattern<F>(&mut self, rest: &RestPat, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    self.enter_pattern(Cow::Borrowed(&rest.arg), on_ident)
  }

  fn enter_pattern<F>(&mut self, pattern: Cow<Pat>, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
  {
    match &*pattern {
      Pat::Ident(ident) => self.enter_ident(&ident.id, on_ident),
      Pat::Array(array) => self.enter_array_pattern(array, on_ident),
      Pat::Assign(assign) => self.enter_assignment_pattern(assign, on_ident),
      Pat::Object(obj) => self.enter_object_pattern(obj, on_ident),
      Pat::Rest(rest) => self.enter_rest_pattern(rest, on_ident),
      Pat::Invalid(_) => (),
      Pat::Expr(_) => (),
    }
  }

  fn enter_patterns<'a, I, F>(&mut self, patterns: I, on_ident: F)
  where
    F: FnOnce(&mut Self, &Ident) + Copy,
    I: Iterator<Item = Cow<'a, Pat>>,
  {
    for pattern in patterns {
      self.enter_pattern(pattern, on_ident);
    }
  }

  pub fn visit(&mut self, ast: &Program) {
    // TODO: `hooks.program.call`
    match ast {
      Program::Module(m) => {
        self.pre_walk_module_declarations(&m.body);
        self.block_pre_walk_module_declarations(&m.body);
        self.walk_module_declarations(&m.body);
      }
      Program::Script(s) => {
        self.pre_walk_statements(&s.body);
        self.block_pre_walk_statements(&s.body);
        self.walk_statements(&s.body);
      }
    };
    // TODO: `hooks.finish.call`
  }

  fn detect_mode(&mut self, stmt: &BlockStmt) {
    let Some(Lit::Str(str)) = stmt
      .stmts
      .first()
      .and_then(|stmt| stmt.as_expr())
      .and_then(|expr_stmt| expr_stmt.expr.as_lit())
    else {
      return;
    };

    if str.value.as_str() == "use strict" {
      let current_scope = self.definitions_db.expect_get_mut(&self.definitions);
      current_scope.is_strict = true;
    }
  }
}
