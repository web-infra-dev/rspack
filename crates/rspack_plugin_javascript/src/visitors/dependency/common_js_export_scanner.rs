use rspack_core::{
  extract_member_expression_chain, BoxDependency, BuildMeta, BuildMetaDefaultObject,
  BuildMetaExportsType, DependencyTemplate, ModuleType, RuntimeGlobals, SpanExt, UsedName,
};
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{
      ArrowExpr, AssignExpr, CallExpr, Callee, ClassMember, Expr, ExprOrSpread, FnDecl, FnExpr,
      Ident, Lit, MemberExpr, ModuleItem, ObjectLit, Pat, PatOrExpr, Program, Prop, PropName,
      PropOrSpread, Stmt, UnaryOp,
    },
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{expr_matcher, is_require_call_expr};
use crate::{
  dependency::{CommonJsExportsDependency, ExportsBase, ModuleDecoratorDependency},
  ClassExt,
};

pub struct CommonJsExportDependencyScanner<'a> {
  dependencies: &'a mut Vec<BoxDependency>,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  unresolved_ctxt: SyntaxContext,
  build_meta: &'a mut BuildMeta,
  module_type: ModuleType,
  is_harmony: bool,
  parser_exports_state: &'a mut Option<bool>,
  enter_call: u32,
  stmt_level: u32,
  last_stmt_is_expr_stmt: bool,
  is_top_level: bool,
}

impl<'a> CommonJsExportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    build_meta: &'a mut BuildMeta,
    module_type: ModuleType,
    parser_exports_state: &'a mut Option<bool>,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      unresolved_ctxt,
      build_meta,
      module_type,
      is_harmony: false,
      parser_exports_state,
      enter_call: 0,
      stmt_level: 0,
      last_stmt_is_expr_stmt: false,
      is_top_level: true,
    }
  }
}

impl Visit for CommonJsExportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &Program) {
    self.is_harmony = matches!(self.module_type, ModuleType::JsEsm | ModuleType::JsxEsm)
      || matches!(program, Program::Module(module) if module.body.iter().any(|s| matches!(s, ModuleItem::ModuleDecl(_))));
    program.visit_children_with(self);
  }

  fn visit_stmt(&mut self, stmt: &Stmt) {
    self.stmt_level += 1;
    let old_last_stmt_is_expr_stmt = self.last_stmt_is_expr_stmt;
    if stmt.is_expr() {
      self.last_stmt_is_expr_stmt = true
    }
    stmt.visit_children_with(self);
    self.last_stmt_is_expr_stmt = old_last_stmt_is_expr_stmt;
    self.stmt_level -= 1;
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if &ident.sym == "module" && ident.span.ctxt == self.unresolved_ctxt {
      // here should use, but scanner is not one pass, so here use extra `visit_program` to calculate is_harmony
      // matches!( self.build_meta.exports_type, BuildMetaExportsType::Namespace)
      let decorator = if self.is_harmony {
        RuntimeGlobals::HARMONY_MODULE_DECORATOR
      } else {
        RuntimeGlobals::NODE_MODULE_DECORATOR
      };
      self
        .presentational_dependencies
        .push(Box::new(ModuleDecoratorDependency::new(decorator)));
      self.bailout();
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_id(expr)
      || expr_matcher::is_module_loaded(expr)
      || expr_matcher::is_module_hot(expr)
      || expr_matcher::is_module_hot_accept(expr)
      || expr_matcher::is_module_hot_decline(expr)
      || (!self.is_harmony && expr_matcher::is_module_exports(expr))
    {
      return;
    }
    // var a = exports/module.exports/this;
    // Object.setPrototypeOf(exports/module.exports/this, a);
    // ...
    if self.is_exports_or_module_exports_or_this_expr(expr) {
      self.bailout();
    }
    expr.visit_children_with(self);
  }

  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    if let PatOrExpr::Pat(box Pat::Expr(box expr)) = &assign_expr.left {
      // exports.xxx = 1;
      // module.exports.xxx = 1;
      // this.xxx = 1;
      let is_exports_start = self.is_exports_member_expr_start(expr);
      let is_module_exports_start = self.is_module_exports_member_expr_start(expr);
      let is_this_start: bool = self.is_this_member_expr_start(expr);

      if is_exports_start || is_module_exports_start || is_this_start {
        if is_exports_start {
          self.enable();
        }

        let remaining_members = expr.as_member().map(|expr: &MemberExpr| {
          extract_member_expression_chain(expr)
            .members()
            .iter()
            .skip(if is_module_exports_start { 2 } else { 1 })
            .map(|n| n.0.clone())
            .collect::<Vec<_>>()
        });

        if let Some(remaining_members) = remaining_members
          && !remaining_members.is_empty()
        {
          // exports.__esModule = true;
          // module.exports.__esModule = true;
          // this.__esModule = true;
          if let Some(first_member) = remaining_members.first()
            && first_member == "__esModule"
          {
            self.check_namespace(
              // const flagIt = () => (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = false
              // const flagIt = () => { exports.__esModule = true }; => stmt_level = 2, last_stmt_is_expr_stmt = true
              // (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = true
              self.stmt_level == 1 && self.last_stmt_is_expr_stmt,
              Some(&assign_expr.right),
            );
          }

          self
            .dependencies
            .push(Box::new(CommonJsExportsDependency::new(
              (expr.span().real_lo(), expr.span().real_hi()),
              None,
              if is_exports_start {
                ExportsBase::Exports
              } else if is_module_exports_start {
                ExportsBase::ModuleExports
              } else if is_this_start {
                ExportsBase::This
              } else {
                panic!("Unexpected expr type");
              },
              remaining_members,
            )));
        }

        assign_expr.right.visit_children_with(self);
        return;
      }
      if self.is_exports_or_module_exports_or_this_expr(expr) {
        self.enable();

        if is_require_call_expr(&assign_expr.right, self.unresolved_ctxt) {
          // exports = require('xx');
          // module.exports = require('xx');
          // this = require('xx');
          // It's possible to reexport __esModule, so we must convert to a dynamic module
          self.set_dynamic();
        } else {
          // exports = {};
          // module.exports = {};
          // this = {};
          self.bailout();
        }
        assign_expr.right.visit_children_with(self);
        return;
      }
    }
    assign_expr.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      // Object.defineProperty(exports, "xxx", { value: 1 });
      // Object.defineProperty(module.exports, "xxx", { value: 1 });
      // Object.defineProperty(this, "xxx", { value: 1 });
      if expr_matcher::is_object_define_property(expr)
        && let Some(ExprOrSpread { expr, .. }) = call_expr.args.first()
        && self.is_exports_or_module_exports_or_this_expr(expr)
        && let Some(arg2) = call_expr.args.get(2)
      {
        self.enable();

        if let Some(ExprOrSpread {
          expr: box Expr::Lit(Lit::Str(str)),
          ..
        }) = call_expr.args.get(1)
        {
          // Object.defineProperty(exports, "__esModule", { value: true });
          // Object.defineProperty(module.exports, "__esModule", { value: true });
          // Object.defineProperty(this, "__esModule", { value: true });
          if str.value == "__esModule" {
            self.check_namespace(
              self.stmt_level == 1,
              get_value_of_property_description(arg2),
            );
          }

          self
            .dependencies
            .push(Box::new(CommonJsExportsDependency::new(
              (call_expr.span.real_lo(), call_expr.span.real_hi()),
              Some((arg2.span().real_lo(), arg2.span().real_hi())),
              if self.is_exports_expr(expr) {
                ExportsBase::DefinePropertyExports
              } else if expr_matcher::is_module_exports(expr) {
                ExportsBase::DefinePropertyModuleExports
              } else if self.is_this_expr(expr) {
                ExportsBase::DefinePropertyThis
              } else {
                panic!("Unexpected expr type");
              },
              vec![str.value.clone()],
            )));
        }

        self.enter_call += 1;
        arg2.visit_children_with(self);
        self.enter_call -= 1;
        return;
      }
      // exports()
      // module.exports()
      // this()
      if self.is_exports_or_module_exports_or_this_expr(expr) {
        self.bailout();
        self.enter_call += 1;
        call_expr.args.visit_children_with(self);
        self.enter_call -= 1;
        return;
      }
    }
    self.enter_call += 1;
    call_expr.visit_children_with(self);
    self.enter_call -= 1;
  }

  fn visit_class_member(&mut self, node: &ClassMember) {
    if let Some(key) = node.class_key()
      && key.is_computed()
    {
      key.visit_with(self);
    }

    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
    let top_level = self.is_top_level;
    self.is_top_level = false;
    node.visit_children_with(self);
    self.is_top_level = top_level;
  }
}

impl<'a> CommonJsExportDependencyScanner<'a> {
  fn is_exports_member_expr_start(&self, mut expr: &Expr) -> bool {
    loop {
      match expr {
        _ if self.is_exports_expr(expr) => return true,
        Expr::Member(MemberExpr { obj, .. }) => expr = obj.as_ref(),
        _ => return false,
      }
    }
  }

  fn is_module_exports_member_expr_start(&self, mut expr: &Expr) -> bool {
    loop {
      match expr {
        _ if expr_matcher::is_module_exports(expr) => return true,
        Expr::Member(MemberExpr { obj, .. }) => expr = obj.as_ref(),
        _ => return false,
      }
    }
  }

  fn is_this_member_expr_start(&self, mut expr: &Expr) -> bool {
    if self.enter_call != 0 {
      return false;
    }
    loop {
      match expr {
        _ if self.is_this_expr(expr) => return true,
        Expr::Member(MemberExpr { obj, .. }) => expr = obj.as_ref(),
        _ => return false,
      }
    }
  }

  fn is_exports_or_module_exports_or_this_expr(&self, expr: &Expr) -> bool {
    self.is_exports_expr(expr) || expr_matcher::is_module_exports(expr) || self.is_this_expr(expr)
  }

  fn is_exports_expr(&self, expr: &Expr) -> bool {
    matches!(expr,  Expr::Ident(ident) if &ident.sym == "exports" && ident.span.ctxt == self.unresolved_ctxt)
  }

  fn is_this_expr(&self, expr: &Expr) -> bool {
    matches!(expr,  Expr::This(_) if self.is_top_level)
  }

  fn check_namespace(&mut self, top_level: bool, value_expr: Option<&Expr>) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if let Some(value_expr) = value_expr
      && is_truthy_literal(value_expr)
      && top_level
    {
      self.set_flagged();
    } else {
      self.set_dynamic();
    }
  }

  // can't scan `__esModule` value
  fn bailout(&mut self) {
    if matches!(self.parser_exports_state, Some(true)) {
      self.build_meta.exports_type = BuildMetaExportsType::Unset;
      self.build_meta.default_object = BuildMetaDefaultObject::False;
    }
    *self.parser_exports_state = Some(false);
  }

  // `__esModule` is false
  fn enable(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) {
      return;
    }
    if self.parser_exports_state.is_none() {
      self.build_meta.exports_type = BuildMetaExportsType::Default;
      self.build_meta.default_object = BuildMetaDefaultObject::Redirect;
    }
    *self.parser_exports_state = Some(true);
  }

  // `__esModule` is true
  fn set_flagged(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if matches!(self.build_meta.exports_type, BuildMetaExportsType::Dynamic) {
      return;
    }
    self.build_meta.exports_type = BuildMetaExportsType::Flagged;
  }

  // `__esModule` is dynamic, eg `true && true`
  fn set_dynamic(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    self.build_meta.exports_type = BuildMetaExportsType::Dynamic;
  }
}

fn get_value_of_property_description(expr_or_spread: &ExprOrSpread) -> Option<&Expr> {
  if let ExprOrSpread {
    expr: box Expr::Object(ObjectLit { props, .. }),
    ..
  } = expr_or_spread
  {
    for prop in props {
      if let PropOrSpread::Prop(prop) = prop
        && let Prop::KeyValue(key_value_prop) = &**prop
        && let PropName::Ident(ident) = &key_value_prop.key
        && &ident.sym == "value"
      {
        return Some(&key_value_prop.value);
      }
    }
  }
  None
}

fn is_truthy_literal(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(lit) => is_lit_truthy_literal(lit),
    Expr::Unary(unary) => {
      if unary.op == UnaryOp::Bang {
        return is_falsy_literal(&unary.arg);
      }
      false
    }
    _ => false,
  }
}

fn is_falsy_literal(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(lit) => !is_lit_truthy_literal(lit),
    Expr::Unary(unary) => {
      if unary.op == UnaryOp::Bang {
        return is_truthy_literal(&unary.arg);
      }
      false
    }
    _ => false,
  }
}

fn is_lit_truthy_literal(lit: &Lit) -> bool {
  match lit {
    Lit::Str(str) => !str.value.is_empty(),
    Lit::Bool(bool) => bool.value,
    Lit::Null(_) => false,
    Lit::Num(num) => num.value != 0.0,
    _ => true,
  }
}
