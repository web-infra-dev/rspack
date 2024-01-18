#![allow(unused)]

mod walk;
mod walk_block_pre;
mod walk_pre;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use rspack_core::needs_refactor::WorkerSyntaxList;
use rspack_core::{BoxDependency, CompilerOptions, DependencyLocation, DependencyTemplate};
use rspack_core::{JavascriptParserUrl, ModuleType, SpanExt};
use rspack_error::miette::Diagnostic;
use rustc_hash::FxHashSet;
use swc_core::common::{SourceFile, Spanned};
use swc_core::ecma::ast::{ArrayPat, AssignPat, ObjectPat, ObjectPatProp, Pat, Program, Stmt};
use swc_core::ecma::ast::{BlockStmt, Expr, Ident, Lit, MemberExpr, RestPat};

use super::api_scanner::ApiParserPlugin;
use crate::parser_plugin::{self, JavaScriptParserPluginDrive};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::scope_info::{ScopeInfoDB, ScopeInfoId};

pub struct JavascriptParser<'parser> {
  pub(crate) source_file: Arc<SourceFile>,
  pub(crate) errors: &'parser mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  pub(crate) dependencies: &'parser mut Vec<BoxDependency>,
  pub(crate) presentational_dependencies: &'parser mut Vec<Box<dyn DependencyTemplate>>,
  pub(crate) ignored: &'parser mut FxHashSet<DependencyLocation>,
  // TODO: remove `worker_syntax_list`
  pub(crate) worker_syntax_list: &'parser WorkerSyntaxList,
  pub(crate) plugin_drive: Rc<JavaScriptParserPluginDrive>,
  pub(super) definitions_db: ScopeInfoDB,
  // ===== scope info =======
  // TODO: `in_if` can be removed after eval identifier
  pub(crate) in_if: bool,
  pub(crate) in_try: bool,
  pub(crate) in_short_hand: bool,
  pub(super) definitions: ScopeInfoId,
}

impl<'parser> JavascriptParser<'parser> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    source_file: Arc<SourceFile>,
    compiler_options: &CompilerOptions,
    dependencies: &'parser mut Vec<BoxDependency>,
    presentational_dependencies: &'parser mut Vec<Box<dyn DependencyTemplate>>,
    ignored: &'parser mut FxHashSet<DependencyLocation>,
    module_type: &ModuleType,
    worker_syntax_list: &'parser WorkerSyntaxList,
    errors: &'parser mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  ) -> Self {
    let mut plugins: Vec<parser_plugin::BoxJavascriptParserPlugin> = vec![
      Box::new(parser_plugin::CheckVarDeclaratorIdent),
      Box::new(parser_plugin::ConstPlugin),
      Box::new(parser_plugin::CommonJsImportsParserPlugin),
      Box::new(parser_plugin::RequireContextDependencyParserPlugin),
      Box::new(ApiParserPlugin),
    ];

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
      plugins.push(Box::new(parser_plugin::WebpackIsIncludedPlugin));
      plugins.push(Box::new(parser_plugin::ExportsInfoApiPlugin));
    }

    if module_type.is_js_auto() || module_type.is_js_esm() {
      let parse_url = &compiler_options
        .module
        .parser
        .as_ref()
        .and_then(|p| p.get(module_type))
        .and_then(|p| p.get_javascript(module_type))
        .map(|p| p.url)
        .unwrap_or(JavascriptParserUrl::Enable);

      if !matches!(parse_url, JavascriptParserUrl::Disable) {
        plugins.push(Box::new(parser_plugin::URLPlugin {
          relative: matches!(parse_url, JavascriptParserUrl::Relative),
        }));
      }
    }

    let plugin_drive = Rc::new(JavaScriptParserPluginDrive::new(plugins));
    let mut db = ScopeInfoDB::new();
    Self {
      source_file,
      errors,
      dependencies,
      presentational_dependencies,
      in_try: false,
      in_if: false,
      in_short_hand: false,
      definitions: db.create(),
      definitions_db: db,
      ignored,
      plugin_drive,
      worker_syntax_list,
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
        self.set_strict(true);
        self.pre_walk_module_declarations(&m.body);
        self.block_pre_walk_module_declarations(&m.body);
        self.walk_module_declarations(&m.body);
      }
      Program::Script(s) => {
        self.detect_mode(&s.body);
        self.pre_walk_statements(&s.body);
        self.block_pre_walk_statements(&s.body);
        self.walk_statements(&s.body);
      }
    };
    // TODO: `hooks.finish.call`
  }

  fn set_strict(&mut self, value: bool) {
    let current_scope = self.definitions_db.expect_get_mut(&self.definitions);
    current_scope.is_strict = value;
  }

  fn detect_mode(&mut self, stmts: &[Stmt]) {
    let Some(Lit::Str(str)) = stmts
      .first()
      .and_then(|stmt| stmt.as_expr())
      .and_then(|expr_stmt| expr_stmt.expr.as_lit())
    else {
      return;
    };

    if str.value.as_str() == "use strict" {
      self.set_strict(true);
    }
  }

  pub fn is_strict(&mut self) -> bool {
    let scope = self.definitions_db.expect_get(&self.definitions);
    scope.is_strict
  }

  // TODO: remove
  pub fn is_unresolved_ident(&mut self, str: &str) -> bool {
    self.definitions_db.get(&self.definitions, str).is_none()
  }

  // TODO: remove
  pub fn is_unresolved_require(&mut self, expr: &Expr) -> bool {
    let ident = match expr {
      Expr::Ident(ident) => Some(ident),
      Expr::Member(mem) => mem.obj.as_ident(),
      _ => None,
    };
    let Some(ident) = ident else {
      unreachable!("please don't use this fn in other case");
    };
    assert!(ident.sym.eq("require"));
    self.is_unresolved_ident(ident.sym.as_str())
  }

  // TODO: remove
  pub fn is_unresolved_member_object_ident(&mut self, expr: &Expr) -> bool {
    if let Expr::Member(member) = expr {
      if let Expr::Ident(ident) = &*member.obj {
        return self.is_unresolved_ident(ident.sym.as_str());
      };
    }
    false
  }
}

impl JavascriptParser<'_> {
  pub fn evaluate_expression(&mut self, expr: &Expr) -> BasicEvaluatedExpression {
    match self.evaluating(expr) {
      Some(evaluated) => {
        if evaluated.is_compile_time_value() {
          let _ = self.ignored.insert(DependencyLocation::new(
            expr.span().real_lo(),
            expr.span().real_hi(),
          ));
        }
        evaluated
      }
      None => BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0),
    }
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating(&mut self, expr: &Expr) -> Option<BasicEvaluatedExpression> {
    match expr {
      Expr::Tpl(tpl) => eval::eval_tpl_expression(self, tpl),
      Expr::Lit(lit) => eval::eval_lit_expr(lit),
      Expr::Cond(cond) => eval::eval_cond_expression(self, cond),
      Expr::Unary(unary) => eval::eval_unary_expression(self, unary),
      Expr::Bin(binary) => eval::eval_binary_expression(self, binary),
      Expr::Array(array) => eval::eval_array_expression(self, array),
      Expr::New(new) => eval::eval_new_expression(self, new),
      _ => None,
    }
  }
}
