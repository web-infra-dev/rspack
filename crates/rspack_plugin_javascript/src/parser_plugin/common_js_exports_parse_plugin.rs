use std::sync::Arc;

use rspack_core::{
  BoxDependency, BuildMetaDefaultObject, BuildMetaExportsType, DependencyRange, RuntimeGlobals,
  UsedByExports,
};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, BlockStmt, BlockStmtOrExpr, CallExpr, Class, ClassMember, Expr, ExprOrSpread,
  Function, GetSpan, GetterProp, Ident, Key, Lit, MemberExpr, MethodProp, Prop, PropName,
  PropOrSpread, SetterProp, Span, Stmt, ThisExpr, UnaryExpr, UnaryOp, Visit, VisitWith,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    CommonJsExportRequireDependency, CommonJsExportsDependency, CommonJsFullRequireDependency,
    CommonJsObjectExportDependency, CommonJsObjectExportKind, CommonJsRequireDependency,
    CommonJsSelfReferenceDependency, ExportsBase, ModuleArgumentDependency,
    ModuleDecoratorDependency,
  },
  parser_plugin::{
    common_js_imports_parse_plugin::is_require_call_expr,
    side_effects_parser_plugin::is_pure_expression,
  },
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::JavascriptParser,
};

#[derive(Default)]
struct OwnThisVisitor {
  span: Option<Span>,
}

impl Visit<'_> for OwnThisVisitor {
  fn visit_this_expr(&mut self, expr: &ThisExpr) {
    self.span.get_or_insert(expr.span);
  }

  // Nested ordinary functions have their own `this`. Arrow functions are
  // intentionally traversed because they inherit it from the exported function.
  fn visit_function(&mut self, _function: &Function<'_>) {}

  fn visit_getter_prop(&mut self, property: &GetterProp<'_>) {
    if let PropName::Computed(key) = &property.key {
      key.expr.visit_with(self);
    }
  }

  fn visit_setter_prop(&mut self, property: &SetterProp<'_>) {
    if let PropName::Computed(key) = &property.key {
      key.expr.visit_with(self);
    }
  }

  fn visit_method_prop(&mut self, property: &MethodProp<'_>) {
    if let PropName::Computed(key) = &property.key {
      key.expr.visit_with(self);
    }
  }

  fn visit_class(&mut self, class: &Class<'_>) {
    class.decorators.visit_with(self);
    class.super_class.visit_with(self);
    for member in &class.body {
      let key = match member {
        ClassMember::Constructor(constructor) => Some(&constructor.key),
        ClassMember::Method(method) => Some(&method.key),
        ClassMember::ClassProp(property) => Some(&property.key),
        ClassMember::AutoAccessor(accessor) => match &accessor.key {
          Key::Public(key) => Some(&**key),
          Key::Private(_) => None,
        },
        ClassMember::PrivateMethod(_)
        | ClassMember::PrivateProp(_)
        | ClassMember::Empty(_)
        | ClassMember::StaticBlock(_) => None,
      };
      if let Some(PropName::Computed(key)) = key {
        key.expr.visit_with(self);
      }
    }
  }
}

fn own_this_in_function(function: &Function) -> Option<Span> {
  let mut visitor = OwnThisVisitor::default();
  function.params.visit_with(&mut visitor);
  function.body.visit_with(&mut visitor);
  visitor.span
}

fn own_this_in_property(prop: &Prop) -> Option<Span> {
  match prop {
    Prop::KeyValue(key_value) => key_value
      .value
      .as_fn()
      .and_then(|function| own_this_in_function(&function.function)),
    Prop::Getter(getter) => {
      let mut visitor = OwnThisVisitor::default();
      getter.body.visit_with(&mut visitor);
      visitor.span
    }
    Prop::Setter(setter) => {
      let mut visitor = OwnThisVisitor::default();
      setter.param.visit_with(&mut visitor);
      setter.body.visit_with(&mut visitor);
      visitor.span
    }
    Prop::Method(method) => own_this_in_function(&method.function),
    Prop::Shorthand(_) | Prop::Assign(_) => None,
  }
}

fn static_object_export_name(property: &PropOrSpread) -> Option<Atom> {
  let PropOrSpread::Prop(property) = property else {
    return None;
  };
  let name = match &**property {
    Prop::Shorthand(ident) => Atom::from(ident.sym.as_str()),
    Prop::KeyValue(key_value) => static_object_export_key(&key_value.key)?,
    Prop::Getter(getter) => static_object_export_key(&getter.key)?,
    Prop::Setter(setter) => static_object_export_key(&setter.key)?,
    Prop::Method(method) => static_object_export_key(&method.key)?,
    Prop::Assign(_) => return None,
  };
  (name != "__proto__").then_some(name)
}

fn static_object_export_key(key: &PropName) -> Option<Atom> {
  match key {
    PropName::Ident(ident) => Some(Atom::from(ident.sym.as_str())),
    PropName::Str(value) => Some(Atom::from(value.value.to_string_lossy().as_ref())),
    PropName::Num(_) | PropName::Computed(_) | PropName::BigInt(_) => None,
  }
}

fn object_export_kind(property: &Prop) -> CommonJsObjectExportKind {
  match property {
    Prop::Shorthand(_) => CommonJsObjectExportKind::Shorthand,
    Prop::KeyValue(_) | Prop::Assign(_) => CommonJsObjectExportKind::KeyValue,
    Prop::Getter(_) => CommonJsObjectExportKind::Getter,
    Prop::Setter(_) => CommonJsObjectExportKind::Setter,
    Prop::Method(method) => match (method.function.is_async, method.function.is_generator) {
      (true, true) => CommonJsObjectExportKind::AsyncGeneratorMethod,
      (true, false) => CommonJsObjectExportKind::AsyncMethod,
      (false, true) => CommonJsObjectExportKind::GeneratorMethod,
      (false, false) => CommonJsObjectExportKind::Method,
    },
  }
}

fn object_export_key_span(property: &Prop) -> Span {
  match property {
    Prop::Shorthand(ident) => ident.span,
    Prop::KeyValue(key_value) => key_value.key.span(),
    Prop::Getter(getter) => getter.key.span(),
    Prop::Setter(setter) => setter.key.span(),
    Prop::Method(method) => method.key.span(),
    Prop::Assign(assign) => assign.key.span,
  }
}

fn add_this_self_reference(parser: &mut JavascriptParser, span: Span) {
  parser.add_dependency(BoxDependency::new(CommonJsSelfReferenceDependency::new(
    span.into(),
    // The dependency exists to mark the whole exports object as referenced. Keep
    // the original `this` expression intact so extracting the function from the
    // exports object still loses its receiver as required by JavaScript semantics.
    ExportsBase::This,
    vec![],
    vec![],
    false,
  )));
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

fn single_return_expression<'a>(body: &'a BlockStmt<'a>) -> Option<&'a Expr<'a>> {
  let [Stmt::Return(return_statement)] = body.stmts.as_slice() else {
    return None;
  };
  return_statement.arg.as_ref()
}

fn getter_reexport_expression<'a>(expr: &'a Expr<'a>) -> Option<&'a Expr<'a>> {
  match expr {
    Expr::Arrow(arrow) => match &arrow.body {
      BlockStmtOrExpr::BlockStmt(body) => single_return_expression(body),
      BlockStmtOrExpr::Expr(expr) => Some(expr),
    },
    Expr::Fn(function) => function
      .function
      .body
      .as_deref()
      .and_then(single_return_expression),
    _ => None,
  }
}

fn descriptor_key<'a>(property: &'a Prop<'a>) -> Option<&'a str> {
  let key = match property {
    Prop::KeyValue(property) => &property.key,
    Prop::Method(property) => &property.key,
    Prop::Getter(_) | Prop::Setter(_) | Prop::Shorthand(_) | Prop::Assign(_) => return None,
  };
  let PropName::Ident(key) = key else {
    return None;
  };
  Some(key.sym.as_str())
}

fn is_boolean_descriptor_field(property: &Prop) -> bool {
  matches!(
    property,
    Prop::KeyValue(property)
      if matches!(&property.value, Expr::Lit(lit) if matches!(&**lit, Lit::Bool(_)))
  )
}

fn get_reexport_of_property_descriptor<'a>(
  descriptor: &'a Expr<'a>,
) -> Option<(&'a Expr<'a>, bool)> {
  let Expr::Object(descriptor) = descriptor else {
    return None;
  };
  let mut value = None;
  let mut getter = None;
  let mut writable = false;
  for property in &descriptor.props {
    let PropOrSpread::Prop(property) = property else {
      return None;
    };
    let key = descriptor_key(property)?;
    match key {
      "value" => {
        let Prop::KeyValue(property) = &**property else {
          return None;
        };
        if value.is_some() {
          return None;
        }
        value = Some(&property.value);
      }
      "get" => {
        if getter.is_some() {
          return None;
        }
        getter = match &**property {
          Prop::KeyValue(property) => getter_reexport_expression(&property.value),
          Prop::Method(property) => property
            .function
            .body
            .as_deref()
            .and_then(single_return_expression),
          Prop::Getter(_) | Prop::Setter(_) | Prop::Shorthand(_) | Prop::Assign(_) => None,
        };
        getter?;
      }
      "enumerable" | "configurable" if is_boolean_descriptor_field(property) => {}
      "writable" if is_boolean_descriptor_field(property) => writable = true,
      _ => return None,
    }
  }
  if (value.is_some() && getter.is_some()) || (getter.is_some() && writable) {
    return None;
  }
  value
    .map(|value| (value, false))
    .or_else(|| getter.map(|getter| (getter, true)))
}

fn handle_object_literal_export(
  parser: &mut JavascriptParser,
  assign_expr: &AssignExpr,
  base: ExportsBase,
) -> Option<bool> {
  if !base.is_module_exports()
    || parser.statement_path.len() != 1
    || !parser.is_statement_level_expression(assign_expr.span)
  {
    return None;
  }
  let Expr::Object(object) = &assign_expr.right else {
    return None;
  };

  // Resolve every key before enabling structured exports. One unknown key
  // makes the entire object opaque, just as it does in webpack.
  let Some(names) = object
    .props
    .iter()
    .map(static_object_export_name)
    .collect::<Option<Vec<_>>>()
  else {
    parser.bailout();
    // Let the default assignment walker handle both sides so `module` still
    // receives its normal argument dependency in the bailout path.
    return None;
  };

  parser.enable();
  // Keep the statically known export graph while preserving the runtime
  // namespace behavior of a whole CommonJS exports-object replacement.
  parser.build_info.commonjs_object_literal_exports = true;

  // Returning from the assignment hook skips walking the left-hand side, so
  // preserve the normal `module` argument rewrite explicitly.
  let module_range = DependencyRange::new(
    assign_expr.left.span().real_lo(),
    assign_expr.left.span().real_lo() + "module".len() as u32,
  );
  let module_loc = parser.to_dependency_location(module_range);
  parser.add_presentational_dependency(Arc::new(ModuleArgumentDependency::new(
    None,
    module_range,
    module_loc,
  )));

  for (property, name) in object.props.iter().zip(names) {
    let PropOrSpread::Prop(property) = property else {
      unreachable!("spread properties were rejected during static key collection")
    };
    let property = &**property;
    let kind = object_export_kind(property);
    let property_range: DependencyRange = property.span().into();
    let key_span = object_export_key_span(property);
    let key_range: DependencyRange = key_span.into();
    let value_range = match property {
      Prop::KeyValue(key_value) => key_value.value.span().into(),
      Prop::Shorthand(_) => property_range,
      Prop::Getter(_) | Prop::Setter(_) | Prop::Method(_) => {
        DependencyRange::new(key_span.real_hi(), property_range.end)
      }
      Prop::Assign(_) => {
        unreachable!("assign properties were rejected during static key collection")
      }
    };
    let pure = match property {
      Prop::KeyValue(key_value) => {
        is_pure_expression(parser, true, &key_value.value, parser.ast.comments, None)
      }
      _ => false,
    };
    let value_has_leading_parenthesis = parser
      .source
      .get(property_range.start as usize..value_range.start as usize)
      .is_some_and(|prefix| prefix.trim_end().ends_with('('));

    if name == "__esModule" {
      let value = match property {
        Prop::KeyValue(key_value) => Some(&key_value.value),
        _ => None,
      };
      parser.check_namespace(true, value);
    }

    parser.add_dependency(BoxDependency::new(CommonJsObjectExportDependency::new(
      property_range,
      key_range,
      value_range,
      name.clone(),
      kind,
      pure,
      value_has_leading_parenthesis,
    )));

    if let Some(span) = own_this_in_property(property) {
      add_this_self_reference(parser, span);
    }

    let nested_dependency_start = kind.is_function().then(|| parser.next_dependency_idx());
    parser.walk_property(property);
    if let Some(start) = nested_dependency_start {
      let used_by_exports = UsedByExports::set(FxHashSet::from_iter([name]));
      for index in start..parser.next_dependency_idx() {
        let Some(dependency) = parser.get_dependency_mut(index) else {
          continue;
        };
        if let Some(dependency) = dependency.downcast_mut::<CommonJsRequireDependency>() {
          dependency.set_used_by_exports(Some(used_by_exports.clone()));
        } else if let Some(dependency) = dependency.downcast_mut::<CommonJsFullRequireDependency>()
        {
          dependency.set_used_by_exports(Some(used_by_exports.clone()));
        }
      }
    }
  }

  Some(true)
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
    parser.add_dependency(BoxDependency::new(CommonJsExportRequireDependency::new(
      arg.string().clone(),
      parser.in_try,
      range,
      None,
      None,
      None,
      base,
      remaining.to_vec(),
      ids,
      !parser.is_statement_level_expression(assign_expr.span),
      false,
    )));
    return Some(true);
  }

  if remaining.is_empty() {
    return handle_object_literal_export(parser, assign_expr, base);
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
  parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
    assign_expr.left.span().into(),
    None,
    base,
    remaining.to_owned(),
  )));
  if let Expr::Fn(function) = &assign_expr.right
    && let Some(span) = own_this_in_function(&function.function)
  {
    add_this_self_reference(parser, span);
  }
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
  parser.add_dependency(BoxDependency::new(CommonJsSelfReferenceDependency::new(
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
      if &property != "__esModule"
        && let Some((reexport, getter)) = get_reexport_of_property_descriptor(arg2)
        && let Some((argument, ids)) = parse_require_call(parser, reexport)
        && argument.is_string()
      {
        parser.add_dependency(BoxDependency::new(CommonJsExportRequireDependency::new(
          argument.string().clone(),
          parser.in_try,
          call_expr.span.into(),
          Some(reexport.span().into()),
          Some(arg0.span().into()),
          Some(arg1.span().into()),
          base,
          vec![property.into()],
          ids,
          false,
          getter,
        )));
        return Some(true);
      }
      // Object.defineProperty(exports, "__esModule", { value: true });
      // Object.defineProperty(module.exports, "__esModule", { value: true });
      // Object.defineProperty(this, "__esModule", { value: true });
      if &property == "__esModule" {
        parser.check_namespace(
          parser.statement_path.len() == 1,
          get_value_of_property_description(arg2),
        );
      }
      parser.add_dependency(BoxDependency::new(CommonJsExportsDependency::new(
        call_expr.span.into(),
        Some(arg2.span().into()),
        base,
        vec![property.into()],
      )));

      if let Expr::Object(descriptor) = arg2 {
        for property in &descriptor.props {
          if let PropOrSpread::Prop(property) = property
            && let Some(span) = own_this_in_property(property)
          {
            add_this_self_reference(parser, span);
          }
        }
      }

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
      parser.add_dependency(BoxDependency::new(ModuleDecoratorDependency::new(
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
