use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ConstDependency, Dependency, DependencyRange,
  DependencyType, ImportPhase, RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, BinaryOp, CallExpr, Expr, ExprOrSpread, GetSpan, Ident, Lit, MemberExpr, Prop,
  PropName, PropOrSpread, Span, Stmt, ThisExpr, UnaryExpr, UnaryOp, VarDeclarator,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsSelfReferenceDependency,
    ESMExportImportedSpecifierDependency, ESMImportSideEffectDependency, ExportsBase,
    ModuleDecoratorDependency,
  },
  parser_plugin::common_js_imports_parse_plugin::is_require_call_expr,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{JavascriptParser, VariableDeclaration},
};

const TYPESCRIPT_EXPORT_STAR_TAG: &str = "typescript export star";
const TYPESCRIPT_EXPORT_STAR_HELPER: &str = "(this&&this.__exportStar)||function(m,exports){for(varpinm)if(p!==\"default\"&&!Object.prototype.hasOwnProperty.call(exports,p))__createBinding(exports,m,p);}";
const TYPESCRIPT_ASSIGN_HELPER: &str = "(this&&this.__assign)||function(){__assign=Object.assign||function(t){for(vars,i=1,n=arguments.length;i<n;i++){s=arguments[i];for(varpins)if(Object.prototype.hasOwnProperty.call(s,p))t[p]=s[p];}returnt;};return__assign.apply(this,arguments);}";
const TYPESCRIPT_DECORATE_HELPER: &str = "(this&&this.__decorate)||function(decorators,target,key,desc){varc=arguments.length,r=c<3?target:desc===null?desc=Object.getOwnPropertyDescriptor(target,key):desc,d;if(typeofReflect===\"object\"&&typeofReflect.decorate===\"function\")r=Reflect.decorate(decorators,target,key,desc);elsefor(vari=decorators.length-1;i>=0;i--)if(d=decorators[i])r=(c<3?d(r):c>3?d(target,key,r):d(target,key))||r;returnc>3&&r&&Object.defineProperty(target,key,r),r;}";

#[derive(Clone)]
struct TypeScriptExportStarTagData;

fn is_typescript_export_star_helper(parser: &JavascriptParser, declarator: &VarDeclarator) -> bool {
  if !parser.is_top_level_scope()
    || declarator
      .name
      .as_ident()
      .is_none_or(|ident| ident.id.sym.as_str() != "__exportStar")
  {
    return false;
  }
  let Some(init) = &declarator.init else {
    return false;
  };
  let range = DependencyRange::from(init.span());
  let Some(source) = parser
    .source()
    .get(range.start as usize..range.end as usize)
  else {
    return false;
  };
  source
    .chars()
    .filter(|char| !char.is_whitespace())
    .eq(TYPESCRIPT_EXPORT_STAR_HELPER.chars())
}

fn typescript_assign_fallback(
  parser: &JavascriptParser,
  declarator: &VarDeclarator,
) -> Option<Span> {
  if !parser.is_top_level_scope()
    || declarator
      .name
      .as_ident()
      .is_none_or(|ident| ident.id.sym.as_str() != "__assign")
  {
    return None;
  }
  let init = declarator.init.as_ref()?;
  let range = DependencyRange::from(init.span());
  let source = parser
    .source()
    .get(range.start as usize..range.end as usize)?;
  if !source
    .chars()
    .filter(|char| !char.is_whitespace())
    .eq(TYPESCRIPT_ASSIGN_HELPER.chars())
  {
    return None;
  }

  let Expr::Bin(cached_helper) = init else {
    return None;
  };
  if cached_helper.op != BinaryOp::LogicalOr {
    return None;
  }
  let Expr::Fn(helper) = &cached_helper.right else {
    return None;
  };
  let Stmt::Expr(assignment) = helper.function.body.as_ref()?.stmts.first()? else {
    return None;
  };
  let Expr::Assign(assignment) = &assignment.expr else {
    return None;
  };
  let Expr::Bin(assign_implementation) = &assignment.right else {
    return None;
  };
  if assign_implementation.op != BinaryOp::LogicalOr {
    return None;
  }
  let Expr::Fn(fallback) = &assign_implementation.right else {
    return None;
  };
  Some(fallback.span())
}

fn typescript_decorate_fallback(
  parser: &JavascriptParser,
  declarator: &VarDeclarator,
) -> Option<Span> {
  if !parser.is_top_level_scope()
    || declarator
      .name
      .as_ident()
      .is_none_or(|ident| ident.id.sym.as_str() != "__decorate")
  {
    return None;
  }
  let init = declarator.init.as_ref()?;
  let range = DependencyRange::from(init.span());
  let source = parser
    .source()
    .get(range.start as usize..range.end as usize)?;
  if !source
    .chars()
    .filter(|char| !char.is_whitespace())
    .eq(TYPESCRIPT_DECORATE_HELPER.chars())
  {
    return None;
  }

  let Expr::Bin(cached_helper) = init else {
    return None;
  };
  if cached_helper.op != BinaryOp::LogicalOr {
    return None;
  }
  let Expr::Fn(fallback) = &cached_helper.right else {
    return None;
  };
  Some(fallback.function.span)
}

fn is_typescript_cached_helper(parser: &JavascriptParser, declarator: &VarDeclarator) -> bool {
  if !parser.is_top_level_scope() {
    return false;
  }
  let Some(name) = declarator
    .name
    .as_ident()
    .map(|ident| ident.id.sym.as_str())
    .filter(|name| matches!(*name, "__createBinding" | "__decorate" | "__exportStar"))
  else {
    return false;
  };
  let Some(init) = &declarator.init else {
    return false;
  };
  let range = DependencyRange::from(init.span());
  let Some(source) = parser
    .source()
    .get(range.start as usize..range.end as usize)
  else {
    return false;
  };
  let source = source
    .chars()
    .filter(|char| !char.is_whitespace())
    .collect::<String>();
  source.starts_with(&format!("(this&&this.{name})||")) && !source.contains("require(")
}

fn is_typescript_export_star_barrel(parser: &JavascriptParser) -> bool {
  let source = parser
    .source()
    .chars()
    .filter(|char| !char.is_whitespace())
    .collect::<String>();
  !source.contains("exports.")
    && !source.contains("module.exports")
    && source.matches("Object.defineProperty(exports,").count() == 1
    && source.contains("Object.defineProperty(exports,\"__esModule\",")
}

fn get_value_of_property_description<'a>(expr: &'a Expr<'a>) -> Option<&'a Expr<'a>> {
  if let Expr::Object(obj) = expr {
    for prop in &obj.props {
      if let PropOrSpread::Prop(prop) = prop
        && let Prop::KeyValue(key_value_prop) = &**prop
        && let PropName::Ident(ident) = &key_value_prop.key
        && ident.sym.as_str() == "value"
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
    Lit::Str(str) => !str.value.as_wtf8().is_empty(),
    Lit::Bool(bool) => bool.value,
    Lit::Null(_) => false,
    Lit::Num(num) => num.value != 0.0,
    _ => true,
  }
}

impl JavascriptParser<'_> {
  // can't scan `__esModule` value
  fn bailout(&mut self) {
    if matches!(self.parser_exports_state, Some(true)) {
      self.build_meta.clear_exports_type();
      self
        .build_meta
        .set_default_object(BuildMetaDefaultObject::False);
    }
    self.parser_exports_state = Some(false);
  }

  // `__esModule` is false
  fn enable(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) {
      return;
    }
    if self.parser_exports_state.is_none() {
      self
        .build_meta
        .set_exports_type(BuildMetaExportsType::Default);
      self
        .build_meta
        .set_default_object(BuildMetaDefaultObject::Redirect);
    }
    self.parser_exports_state = Some(true);
  }

  // `__esModule` is true
  fn set_flagged(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    if matches!(
      self.build_meta.exports_type(),
      BuildMetaExportsType::Dynamic
    ) {
      return;
    }
    self
      .build_meta
      .set_exports_type(BuildMetaExportsType::Flagged);
  }

  // `__esModule` is dynamic, eg `true && true`
  fn set_dynamic(&mut self) {
    if matches!(self.parser_exports_state, Some(false)) || self.parser_exports_state.is_none() {
      return;
    }
    self
      .build_meta
      .set_exports_type(BuildMetaExportsType::Dynamic);
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
}

fn parse_require_call<'p: 'a, 'a>(
  parser: &mut JavascriptParser<'p>,
  mut expr: &'a Expr<'a>,
) -> Option<(BasicEvaluatedExpression<'a>, Vec<Atom>)> {
  let mut ids = Vec::new();
  while let Some(member) = expr.as_member() {
    if let Some(prop) = member.prop.as_ident() {
      ids.push(Atom::from(prop.sym.as_str()));
    } else if let Some(prop) = member.prop.as_computed()
      && let prop = parser.evaluate_expression(&prop.expr)
      && let Some(prop) = prop.as_string()
    {
      ids.push(prop.into());
    } else {
      return None;
    }
    expr = &member.obj;
  }
  if let Some(call) = expr.as_call()
    && is_require_call_expr(parser, call)
  {
    let arg = &call.args[0];
    if arg.spread.is_some() {
      return None;
    }
    let arg = parser.evaluate_expression(&arg.expr);
    ids.reverse();
    return Some((arg, ids));
  }
  None
}

fn handle_assign_export(
  parser: &mut JavascriptParser,
  assign_expr: &AssignExpr,
  remaining: &[Atom],
  base: ExportsBase,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if (remaining.is_empty() || remaining.first().is_some_and(|i| i != "__esModule"))
    && let Some((arg, ids)) = parse_require_call(parser, &assign_expr.right)
    && arg.is_string()
  {
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
    parser.add_dependency(Box::new(CommonJsExportRequireDependency::new(
      arg.string().clone(),
      parser.in_try,
      range,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_expr.span),
    )));
    return Some(true);
  }

  if remaining.is_empty() {
    return None;
  }

  if parser.is_statement_level_expression(assign_expr.span)
    && let Some(name) = remaining.first()
    && name != "__esModule"
  {
    parser.common_js_named_exports.insert(name.clone());
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
      parser.statement_path.len() == 1 && parser.is_statement_level_expression(assign_expr.span),
      Some(&assign_expr.right),
    );
  }
  // exports.a = 1;
  // module.exports.a = 1;
  // this.a = 1;
  parser.add_dependency(Box::new(CommonJsExportsDependency::new(
    assign_expr.left.span().into(),
    None,
    base,
    remaining.to_owned(),
  )));
  parser.walk_expression(&assign_expr.right);
  Some(true)
}

fn handle_access_export(
  parser: &mut JavascriptParser,
  expr_span: Span,
  remaining: &[Atom],
  remaining_optionals: &[bool],
  base: ExportsBase,
  call_args: Option<&[ExprOrSpread<'_>]>,
) -> Option<bool> {
  if parser.is_esm {
    return None;
  }
  if remaining.is_empty() {
    parser.bailout();
  }
  parser.add_dependency(Box::new(CommonJsSelfReferenceDependency::new(
    expr_span.into(),
    base,
    remaining.to_vec(),
    remaining_optionals.to_vec(),
    call_args.is_some(),
  )));
  if let Some(call_args) = call_args {
    parser.walk_expr_or_spread(call_args);
  }
  Some(true)
}

pub struct CommonJsExportsParserPlugin {
  skip_in_esm: bool,
}

impl CommonJsExportsParserPlugin {
  pub fn new(skip_in_esm: bool) -> Self {
    Self { skip_in_esm }
  }

  fn should_skip_handler(&self, parser: &JavascriptParser) -> bool {
    self.skip_in_esm && parser.is_esm
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CommonJsExportsParserPlugin {
  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: &VarDeclarator,
    _declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if is_typescript_export_star_helper(parser, declarator) {
      let ident = declarator
        .name
        .as_ident()
        .expect("TypeScript export star helper should have an identifier");
      parser.tag_variable(
        Atom::from(ident.id.sym.as_str()),
        TYPESCRIPT_EXPORT_STAR_TAG,
        Some(TypeScriptExportStarTagData),
      );
    }
    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: &VarDeclarator,
    _declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if let Some(fallback) = typescript_assign_fallback(parser, declarator) {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        fallback.into(),
        RuntimeGlobals::TYPESCRIPT_ASSIGN,
      )));
    }
    if let Some(fallback) = typescript_decorate_fallback(parser, declarator) {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        fallback.into(),
        RuntimeGlobals::TYPESCRIPT_DECORATE,
      )));
    }

    if !is_typescript_cached_helper(parser, declarator) {
      return None;
    }

    if is_typescript_export_star_barrel(parser)
      && declarator
        .name
        .as_ident()
        .is_some_and(|ident| matches!(ident.id.sym.as_str(), "__createBinding" | "__exportStar"))
      && let Some(init) = &declarator.init
    {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        init.span().into(),
        "undefined".into(),
      )));
    }

    Some(true)
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    assign_expr: &AssignExpr,
    remaining: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.x = y;
      return handle_assign_export(parser, assign_expr, remaining, ExportsBase::Exports);
    }
    if for_name == "module" && matches!(remaining.first(), Some(first) if first == "exports") {
      // module.exports.x = y;
      return handle_assign_export(
        parser,
        assign_expr,
        &remaining[1..],
        ExportsBase::ModuleExports,
      );
    }
    if for_name == "this" && parser.is_top_level_scope() {
      // this.x = y
      return handle_assign_export(parser, assign_expr, remaining, ExportsBase::This);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_esm {
      return None;
    }
    if (for_name == TYPESCRIPT_EXPORT_STAR_TAG
      || parser
        .get_tag_data::<TypeScriptExportStarTagData>(
          &Atom::from(for_name),
          TYPESCRIPT_EXPORT_STAR_TAG,
        )
        .is_some())
      && parser.is_statement_level_expression(call_expr.span)
      && call_expr.args.len() == 2
      && let [
        ExprOrSpread {
          spread: None,
          expr: imported,
        },
        ExprOrSpread {
          spread: None,
          expr: exports,
        },
      ] = call_expr.args.as_slice()
      && let exports = parser.evaluate_expression(exports)
      && exports.is_identifier()
      && exports.identifier().as_str() == "exports"
      && let Some((request, ids)) = parse_require_call(parser, imported)
      && request.is_string()
      && ids.is_empty()
    {
      parser.enable();
      parser.last_esm_import_order += 1;
      let source_order = parser.last_esm_import_order;
      let request = Atom::from(request.string().as_str());
      let range = DependencyRange::from(call_expr.span);
      let loc = parser.to_dependency_location(range);

      parser
        .add_presentational_dependency(Box::new(ConstDependency::new(range, String::new().into())));

      let mut side_effect_dep = ESMImportSideEffectDependency::new(
        request.clone(),
        source_order,
        range,
        DependencyType::EsmExportImport,
        ImportPhase::Evaluation,
        None,
        loc.clone(),
        true,
      );
      let mut export_dep = ESMExportImportedSpecifierDependency::new(
        request,
        source_order,
        vec![],
        None,
        Some(parser.build_info.all_star_exports.clone()),
        range,
        ESMExportImportedSpecifierDependency::create_export_presence_mode(
          parser.javascript_options,
        ),
        ImportPhase::Evaluation,
        None,
        loc,
      );
      export_dep.set_commonjs_export_star(parser.common_js_named_exports.iter().cloned().collect());
      parser.build_info.all_star_exports.push(export_dep.id);
      if parser
        .factory_meta
        .and_then(|meta| meta.side_effect_free)
        .unwrap_or_default()
      {
        side_effect_dep.set_lazy();
        export_dep.set_lazy();
      }
      parser.add_dependency(Box::new(side_effect_dep));
      parser.add_dependency(Box::new(export_dep));
      return Some(true);
    }
    if for_name == "Object.defineProperty"
      && parser.is_statement_level_expression(call_expr.span)
      && call_expr.args.len() == 3
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg0,
      }) = call_expr.args.first()
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg1,
      }) = call_expr.args.get(1)
      && let Some(ExprOrSpread {
        spread: None,
        expr: arg2,
      }) = call_expr.args.get(2)
    {
      let exports_arg = parser.evaluate_expression(arg0);
      if !exports_arg.is_identifier() {
        return None;
      }
      let base = match exports_arg.identifier().as_str() {
        "exports" => ExportsBase::DefinePropertyExports,
        "module.exports" => ExportsBase::DefinePropertyModuleExports,
        "this" if parser.is_top_level_scope() => ExportsBase::DefinePropertyThis,
        _ => return None,
      };
      let property = parser.evaluate_expression(arg1).as_string()?;
      parser.enable();
      // Object.defineProperty(exports, "__esModule", { value: true });
      // Object.defineProperty(module.exports, "__esModule", { value: true });
      // Object.defineProperty(this, "__esModule", { value: true });
      if &property == "__esModule" {
        parser.check_namespace(
          parser.statement_path.len() == 1,
          get_value_of_property_description(arg2),
        );
      } else {
        parser
          .common_js_named_exports
          .insert(property.clone().into());
      }
      parser.add_dependency(Box::new(CommonJsExportsDependency::new(
        call_expr.span.into(),
        Some(arg2.span().into()),
        base,
        vec![property.into()],
      )));

      parser.walk_expression(arg2);
      return Some(true);
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module" {
      let decorator = if parser.is_esm {
        RuntimeGlobals::ESM_MODULE_DECORATOR
      } else {
        RuntimeGlobals::NODE_MODULE_DECORATOR
      };
      parser.bailout();
      parser.add_dependency(Box::new(ModuleDecoratorDependency::new(
        decorator,
        !parser.is_esm,
      )));
      return Some(true);
    }

    if for_name == "exports" {
      // exports
      return handle_access_export(parser, ident.span, &[], &[], ExportsBase::Exports, None);
    }

    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if parser.is_top_level_this() {
      // this
      return handle_access_export(parser, expr.span, &[], &[], ExportsBase::This, None);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "module.exports" {
      // module.exports
      return handle_access_export(
        parser,
        expr.span,
        &[],
        &[],
        ExportsBase::ModuleExports,
        None,
      );
    }
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c
      return handle_access_export(
        parser,
        expr.span,
        members,
        members_optionals,
        ExportsBase::Exports,
        None,
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c
      return handle_access_export(
        parser,
        expr.span,
        &members[1..],
        &members_optionals[1..],
        ExportsBase::ModuleExports,
        None,
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c
      return handle_access_export(
        parser,
        expr.span,
        members,
        members_optionals,
        ExportsBase::This,
        None,
      );
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if self.should_skip_handler(parser) {
      return None;
    }

    if for_name == "exports" {
      // exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        members,
        members_optionals,
        ExportsBase::Exports,
        Some(&expr.args),
      );
    }

    if for_name == "module" && matches!(members.first(), Some(first) if first == "exports") {
      // module.exports.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        &members[1..],
        &members_optionals[1..],
        ExportsBase::ModuleExports,
        Some(&expr.args),
      );
    }

    if for_name == "this" && parser.is_top_level_scope() {
      // this.a.b.c()
      return handle_access_export(
        parser,
        expr.callee.span(),
        members,
        members_optionals,
        ExportsBase::This,
        Some(&expr.args),
      );
    }

    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if self.should_skip_handler(parser) {
      return None;
    }

    (for_name == "module" || for_name == "exports").then(|| {
      eval::evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      )
    })
  }
}
