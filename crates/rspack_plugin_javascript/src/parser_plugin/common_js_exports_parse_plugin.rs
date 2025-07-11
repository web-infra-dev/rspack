use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals,
  RuntimeRequirementsDependency, SpanExt,
};
use swc_core::{
  atoms::Atom,
  common::Spanned,
  ecma::ast::{
    AssignExpr, AssignTarget, CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr,
    ObjectLit, Prop, PropName, PropOrSpread, SimpleAssignTarget, ThisExpr, UnaryExpr, UnaryOp,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsSelfReferenceDependency,
    ConsumeSharedExportsDependency, ExportsBase, ModuleDecoratorDependency,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    expr_like::ExprLike, expr_matcher, AllowedMemberTypes, JavascriptParser, MemberExpressionInfo,
    TopLevelScope,
  },
};

const MODULE_NAME: &str = "module";
const EXPORTS_NAME: &str = "exports";

fn get_member_expression_info<E: ExprLike>(
  parser: &mut JavascriptParser,
  expr: &E,
  is_module_exports_start: Option<bool>,
) -> Option<Vec<Atom>> {
  let is_module_exports_start = match is_module_exports_start {
    Some(v) => v,
    None => is_module_exports_member_expr_start(expr),
  };
  expr.as_member().and_then(|expr: &MemberExpr| {
    let members = parser
      .get_member_expression_info(expr, AllowedMemberTypes::Expression)
      .and_then(|info| match info {
        MemberExpressionInfo::Call(_) => None,
        MemberExpressionInfo::Expression(info) => Some(info.members),
      })
      .map(|members| {
        members
          .iter()
          .skip(if is_module_exports_start { 1 } else { 0 })
          .map(|n| n.to_owned())
          .collect::<Vec<_>>()
      })?;
    match expr.obj {
      box Expr::Call(_) => Some(members),
      box Expr::Ident(_) => Some(members),
      box Expr::MetaProp(_) => Some(members),
      box Expr::This(_) => Some(members),
      _ if expr_matcher::is_module_exports(&*expr.obj) => Some(members),
      _ => None,
    }
  })
}

fn is_module_exports_member_expr_start<E: ExprLike>(expr: &E) -> bool {
  fn walk_each<E: ExprLike>(expr: &E) -> bool {
    if expr_matcher::is_module_exports(expr) {
      true
    } else if let Some(MemberExpr { obj, .. }) = expr.as_member() {
      walk_each(&**obj)
    } else {
      false
    }
  }
  walk_each(expr)
}

fn get_value_of_property_description(expr_or_spread: &ExprOrSpread) -> Option<&Expr> {
  if let Expr::Object(ObjectLit { props, .. }) = expr_or_spread.expr.unwrap_parens() {
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

impl JavascriptParser<'_> {
  fn is_exports_member_expr_start<E: ExprLike>(&mut self, expr: &E) -> bool {
    fn walk_each<E: ExprLike>(parser: &mut JavascriptParser, expr: &E) -> bool {
      if parser.is_exports_expr(expr) {
        true
      } else if let Some(MemberExpr { obj, .. }) = expr.as_member() {
        walk_each(parser, &**obj)
      } else {
        false
      }
    }
    walk_each(self, expr)
  }

  fn is_module_ident(&mut self, ident: &Ident) -> bool {
    ident.sym == MODULE_NAME && self.is_unresolved_ident(MODULE_NAME)
  }

  fn is_exports_ident<E: ExprLike>(&mut self, expr: &E) -> bool {
    expr
      .as_ident()
      .is_some_and(|ident| ident.sym == EXPORTS_NAME && self.is_unresolved_ident(EXPORTS_NAME))
  }

  fn is_exports_expr<E: ExprLike>(&mut self, expr: &E) -> bool {
    expr
      .as_ident()
      .is_some_and(|ident| self.is_exports_ident(ident))
  }

  fn is_top_level_this(&self, _expr: &ThisExpr) -> bool {
    !matches!(self.top_level_scope, TopLevelScope::False)
  }

  fn is_top_level_this_expr<E: ExprLike>(&self, expr: &E) -> bool {
    expr.as_this().is_some_and(|e| self.is_top_level_this(e))
  }

  fn is_exports_or_module_exports_or_this_expr(&mut self, expr: &Expr) -> bool {
    self.is_exports_expr(expr)
      || expr_matcher::is_module_exports(expr)
      || self.is_top_level_this_expr(expr)
  }

  // can't scan `__esModule` value
  fn bailout(&mut self) {
    if matches!(self.parser_exports_state, Some(true)) {
      self.build_meta.exports_type = BuildMetaExportsType::Unset;
      self.build_meta.default_object = BuildMetaDefaultObject::False;
    }
    self.parser_exports_state = Some(false);
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
    self.parser_exports_state = Some(true);
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

  fn is_this_member_expr_start<E: ExprLike>(&self, expr: &E) -> bool {
    if self.enter_call != 0 {
      return false;
    }
    fn walk_each<E: ExprLike>(parser: &JavascriptParser, expr: &E) -> bool {
      if parser.is_top_level_this_expr(expr) {
        true
      } else if let Some(MemberExpr { obj, .. }) = expr.as_member() {
        walk_each(parser, &**obj)
      } else {
        false
      }
    }
    walk_each(self, expr)
  }

  fn is_require_call(&mut self, node: &CallExpr) -> bool {
    node
      .callee
      .as_expr()
      .map(|expr| matches!(expr, box Expr::Ident(ident) if &ident.sym == "require" && self.is_unresolved_ident("require")))
      .unwrap_or_default()
  }

  fn is_require_call_expr(&mut self, expr: &Expr) -> bool {
    matches!(expr, Expr::Call(call_expr) if self.is_require_call(call_expr))
  }

  // FIXME: this function should be deleted because it just a hack
  fn append_module_runtime(&mut self) {
    self
      .presentational_dependencies
      .push(Box::new(RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE,
      )));
  }
}

pub struct CommonJsExportsParserPlugin;

impl CommonJsExportsParserPlugin {
  /// Detect if this module should use ConsumeSharedExportsDependency based on Module Federation context
  fn detect_shared_module_key(parser: &JavascriptParser) -> Option<String> {
    // Check if this module has a shared_key in BuildMeta (set by ProvideSharedPlugin)
    if let Some(ref shared_key) = parser.build_meta.shared_key {
      return Some(shared_key.clone());
    }

    // Check if this module has consume_shared_key in BuildMeta (set by ConsumeSharedPlugin)
    if let Some(ref consume_shared_key) = parser.build_meta.consume_shared_key {
      return Some(consume_shared_key.clone());
    }

    // No shared context found
    None
  }
}

impl JavascriptParserPlugin for CommonJsExportsParserPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    if parser.is_module_ident(ident) {
      parser.append_module_runtime();
      // matches!( self.build_meta.exports_type, BuildMetaExportsType::Namespace)
      let decorator = if parser.is_esm {
        RuntimeGlobals::ESM_MODULE_DECORATOR
      } else {
        RuntimeGlobals::NODE_MODULE_DECORATOR
      };
      parser.bailout();
      parser
        .dependencies
        .push(Box::new(ModuleDecoratorDependency::new(
          decorator,
          !parser.is_esm,
        )));
      Some(true)
    } else if !parser.is_esm && parser.is_exports_ident(ident) {
      parser.bailout();
      parser
        .dependencies
        .push(Box::new(CommonJsSelfReferenceDependency::new(
          ident.span().into(),
          ExportsBase::Exports,
          vec![],
          false,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
  ) -> Option<bool> {
    if parser.is_esm {
      None
    } else if parser.is_top_level_this(expr) {
      parser.bailout();
      parser
        .dependencies
        .push(Box::new(CommonJsSelfReferenceDependency::new(
          expr.span().into(),
          ExportsBase::This,
          vec![],
          false,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(&self, parser: &mut JavascriptParser, expr: &MemberExpr, _name: &str) -> Option<bool> {
    if parser.is_esm {
      return None;
    }

    let handle_remaining = |parser: &mut JavascriptParser, base: ExportsBase| {
      let is_module_exports_start = matches!(base, ExportsBase::ModuleExports);
      if let Some(remaining) =
        get_member_expression_info(parser, expr, Some(is_module_exports_start))
      {
        if remaining.is_empty() {
          parser.bailout();
        }
        parser
          .dependencies
          .push(Box::new(CommonJsSelfReferenceDependency::new(
            expr.span().into(),
            base,
            remaining,
            false,
          )));
        Some(true)
      } else {
        None
      }
    };
    if parser.is_exports_member_expr_start(expr) {
      // `exports.x.y`
      handle_remaining(parser, ExportsBase::Exports)
    } else if is_module_exports_member_expr_start(expr) {
      // `module.exports.x.y`
      parser.append_module_runtime();
      handle_remaining(parser, ExportsBase::ModuleExports)
    } else if parser.is_this_member_expr_start(expr) {
      // `this.x.y`
      handle_remaining(parser, ExportsBase::This)
    } else {
      None
    }
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    assign_expr: &AssignExpr,
    _for_name: Option<&str>,
  ) -> Option<bool> {
    if parser.is_esm {
      return None;
    }
    let AssignTarget::Simple(SimpleAssignTarget::Member(left_expr)) = &assign_expr.left else {
      return None;
    };

    let handle_remaining = |parser: &mut JavascriptParser, base: ExportsBase| {
      let is_module_exports_start = matches!(base, ExportsBase::ModuleExports);
      let remaining = get_member_expression_info(parser, left_expr, Some(is_module_exports_start))?;

      if (remaining.is_empty() || remaining.first().is_some_and(|i| i != "__esModule"))
        && parser.is_require_call_expr(&assign_expr.right)
        && let Some(right_expr) = assign_expr.right.as_call()
        && let Some(first_arg) = right_expr.args.first().map(|arg| &arg.expr)
      {
        let param = parser.evaluate_expression(first_arg);
        if param.is_string() {
          parser.enable();
          if remaining.is_empty() {
            // exports = require('xx');
            // module.exports = require('xx');
            // this = require('xx');
            // It's possible to reexport __esModule, so we must convert to a dynamic module
            parser.set_dynamic();
          }
          // exports.aaa = require('xx');
          // module.exports.aaa = require('xx');
          // this.aaa = require('xx');
          let range: DependencyRange = assign_expr.span.into();
          parser
            .dependencies
            .push(Box::new(CommonJsExportRequireDependency::new(
              param.string().to_string(),
              parser.in_try,
              range,
              base,
              remaining,
              !parser.is_statement_level_expression(assign_expr.span()),
            )));
          return Some(true);
        }
      }

      if remaining.is_empty() {
        return None;
      }

      parser.enable();
      // exports.__esModule = true;
      // module.exports.__esModule = true;
      // this.__esModule = true;
      if let Some(first_member) = remaining.first()
        && first_member == "__esModule"
      {
        parser.check_namespace(
          // const flagIt = () => (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = false
          // const flagIt = () => { exports.__esModule = true }; => stmt_level = 2, last_stmt_is_expr_stmt = true
          // (exports.__esModule = true); => stmt_level = 1, last_stmt_is_expr_stmt = true
          parser.statement_path.len() == 1
            && parser.is_statement_level_expression(assign_expr.span()),
          Some(&assign_expr.right),
        );
      }
      // exports.a = 1;
      // module.exports.a = 1;
      // this.a = 1;

      // Check if we're in ConsumeShared OR ProvideShared context
      let shared_key = Self::detect_shared_module_key(parser);

      if let Some(shared_key) = shared_key {
        // For assignments, pass the member expression span for now
        // TODO: We need to figure out why assign_expr.span doesn't include the full assignment
        parser
          .dependencies
          .push(Box::new(ConsumeSharedExportsDependency::new(
            left_expr.span().into(),
            Some(assign_expr.span.into()),
            base,
            remaining.to_owned(),
            shared_key,
          )));
      } else {
        // For assignments, pass both member expression and assignment spans
        parser
          .dependencies
          .push(Box::new(CommonJsExportsDependency::new(
            left_expr.span().into(),
            Some(assign_expr.span.into()),
            base,
            remaining.to_owned(),
          )));
      }
      parser.walk_expression(&assign_expr.right);
      Some(true)
    };

    if parser.is_exports_member_expr_start(left_expr) {
      // exports.x = y;
      handle_remaining(parser, ExportsBase::Exports)
    } else if is_module_exports_member_expr_start(left_expr) {
      // module.exports.x = y;
      parser.append_module_runtime();
      handle_remaining(parser, ExportsBase::ModuleExports)
    } else if parser.is_this_member_expr_start(left_expr) {
      // this.x = y
      handle_remaining(parser, ExportsBase::This)
    } else {
      None
    }
  }

  fn call(&self, parser: &mut JavascriptParser, call_expr: &CallExpr, _name: &str) -> Option<bool> {
    if parser.is_esm {
      None
    } else if let Callee::Expr(expr) = &call_expr.callee {
      let handle_remaining = |parser: &mut JavascriptParser, base: ExportsBase| {
        let is_module_exports_start = matches!(base, ExportsBase::ModuleExports);
        if let Some(remaining) =
          get_member_expression_info(parser, &**expr, Some(is_module_exports_start))
        {
          // exports()
          // module.exports()
          // this()
          if remaining.is_empty() {
            parser.bailout();
          }

          // exports.a.b()
          // module.exports.a.b()
          // this.a.b()
          parser
            .dependencies
            .push(Box::new(CommonJsSelfReferenceDependency::new(
              expr.span().into(),
              base,
              remaining,
              true,
            )));
          parser.walk_expr_or_spread(&call_expr.args);
          Some(true)
        } else {
          None
        }
      };
      // Object.defineProperty(exports, "xxx", { value: 1 });
      // Object.defineProperty(module.exports, "xxx", { value: 1 });
      // Object.defineProperty(this, "xxx", { value: 1 });
      if expr_matcher::is_object_define_property(&**expr)
        && parser.is_statement_level_expression(call_expr.span())
        && let Some(ExprOrSpread { expr, .. }) = call_expr.args.first()
        && parser.is_exports_or_module_exports_or_this_expr(expr)
        && let Some(arg2) = call_expr.args.get(2)
      {
        let Some(ExprOrSpread {
          expr: box Expr::Lit(Lit::Str(str)),
          ..
        }) = call_expr.args.get(1)
        else {
          return None;
        };

        parser.enable();
        // Object.defineProperty(exports, "__esModule", { value: true });
        // Object.defineProperty(module.exports, "__esModule", { value: true });
        // Object.defineProperty(this, "__esModule", { value: true });
        if str.value == "__esModule" {
          parser.check_namespace(
            parser.statement_path.len() == 1,
            get_value_of_property_description(arg2),
          );
        }

        let base = if parser.is_exports_expr(&**expr) {
          ExportsBase::DefinePropertyExports
        } else if expr_matcher::is_module_exports(&**expr) {
          ExportsBase::DefinePropertyModuleExports
        } else if parser.is_top_level_this_expr(&**expr) {
          ExportsBase::DefinePropertyThis
        } else {
          panic!("Unexpected expr type");
        };
        // Check if we're in ConsumeShared OR ProvideShared context
        let shared_key = Self::detect_shared_module_key(parser);

        if let Some(shared_key) = shared_key {
          parser
            .dependencies
            .push(Box::new(ConsumeSharedExportsDependency::new(
              call_expr.span.into(),
              Some(arg2.span().into()),
              base,
              vec![str.value.clone()],
              shared_key,
            )));
        } else {
          parser
            .dependencies
            .push(Box::new(CommonJsExportsDependency::new(
              call_expr.span.into(),
              Some(arg2.span().into()),
              base,
              vec![str.value.clone()],
            )));
        }

        parser.walk_expression(&arg2.expr);
        Some(true)
      } else if parser.is_exports_member_expr_start(&**expr) {
        // exports.x()
        handle_remaining(parser, ExportsBase::Exports)
      } else if is_module_exports_member_expr_start(&**expr) {
        // module.exports.x()
        parser.append_module_runtime();
        handle_remaining(parser, ExportsBase::ModuleExports)
      } else if parser.is_this_member_expr_start(&**expr) {
        // this.x()
        handle_remaining(parser, ExportsBase::This)
      } else {
        None
      }
    } else {
      None
    }
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    (for_name == "module" || for_name == "exports").then(|| {
      eval::evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      )
    })
  }
}
