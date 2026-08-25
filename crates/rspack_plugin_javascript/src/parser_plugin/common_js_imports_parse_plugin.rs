#[cfg(windows)]
use std::path::Path;
use std::sync::Arc;

use rspack_core::{
  BoxDependency, ConstDependency, Context, ContextDependency, ContextMode, ContextModulePattern,
  ContextOptions, DependencyCategory, DependencyRange, DependencyType, ImportMetaKnownProperties,
  ModuleType, ReferencedSpecifier, RuntimeGlobals, RuntimeRequirementsDependency, get_context,
};
use rspack_error::{Diagnostic, Severity};
use rspack_util::{SpanExt, json_stringify_str};
use swc_next_ecma_ast::{
  Argument, ArgumentData, AssignmentExpression, AssignmentOperator, Ast, CallExpression, Expr,
  ExprData, GetSpan, MemberExpression, NewExpression, PropertyKeyData, Span, TypedSubRange,
  UnaryExpression, UnaryOperator, VariableDeclarator,
};
use url::Url;

use super::{
  InnerGraphParserPlugin, JavascriptParserPlugin,
  esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  get_url_request,
  inner_graph::state::InnerGraphUsageOperation,
  url_plugin::is_meta_url,
};
use crate::{
  Atom,
  dependency::{
    CommonJsFullRequireDependency, CommonJsRequireContextDependency, CommonJsRequireDependency,
    DependencyBranchGuard, ESMImportSpecifierDependency, RequireHeaderDependency,
    RequireResolveContextDependency, RequireResolveDependency, RequireResolveHeaderDependency,
    local_module_dependency::LocalModuleDependency,
  },
  magic_comment::try_extract_magic_comment,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    CallHooksName, ExportedVariableInfo, HookMemberExpression, Identifier, JavascriptParser,
    StatementPath, TagInfoData, VariableDeclaration, VariableDeclarationKind, VariableInfo,
    VariableInfoFlags, context_reg_exp, create_context_dependency, create_traceable_error,
    expr_name, get_non_optional_part, member_property_key_to_atom,
  },
};

const COMMONJS_REQUIRE_TAG: &str = "commonjs require";
pub const CREATE_REQUIRE_SPECIFIER_TAG: &str = "createRequire";
pub const CREATE_REQUIRE_EVALUATED_TAG: &str = "\0createRequire";
pub const CREATED_REQUIRE_IDENTIFIER_TAG: &str = "createRequire()";

#[derive(Clone)]
pub struct CreatedRequireTagData {
  pub(crate) context: Context,
  pub(crate) side_effects: String,
  // The deferred `const req = createRequire(import.meta.url)` declaration, if any.
  pub(crate) pending_call: Option<Span>,
  // Whether unhandled uses may keep using the real runtime require object.
  pub(crate) preserve_unhandled: bool,
}

struct CreateRequireArgument {
  value: String,
  context: Context,
  replace_argument: bool,
}

#[derive(Default)]
pub struct CreatedRequireReferencesState {
  pending: rustc_hash::FxHashMap<Span, PendingCreatedRequire>,
  exported_locals: rustc_hash::FxHashSet<Atom>,
}

struct PendingCreatedRequire {
  must_keep: bool,
  callee: DeferredCreateRequireCallee,
  // Deferred calls skip this expression until their keep/strip state is known.
  argument: Expr,
  statement_path: Vec<StatementPath>,
  prev_statement: Option<StatementPath>,
}

struct DeferredCreateRequireCallee {
  settings: ESMSpecifierData,
  range: DependencyRange,
  ids: Vec<Atom>,
  asi_safe: bool,
  direct_import: bool,
  ns_access: bool,
  branch_guard: Option<DependencyBranchGuard>,
}

impl CreatedRequireReferencesState {
  fn add_pending(
    &mut self,
    call_span: Span,
    callee: DeferredCreateRequireCallee,
    argument: Expr,
    statement_path: Vec<StatementPath>,
    prev_statement: Option<StatementPath>,
  ) {
    // Normal walk refreshes provisional pre-walk data after earlier references may mark it.
    let must_keep = self
      .pending
      .get(&call_span)
      .is_some_and(|pending| pending.must_keep);
    self.pending.insert(
      call_span,
      PendingCreatedRequire {
        must_keep,
        callee,
        argument,
        statement_path,
        prev_statement,
      },
    );
  }

  fn mark_must_keep(&mut self, call_span: Span) {
    if let Some(pending) = self.pending.get_mut(&call_span) {
      pending.must_keep = true;
    }
  }

  fn take_pending(&mut self) -> Vec<(Span, PendingCreatedRequire)> {
    let mut pending = std::mem::take(&mut self.pending)
      .into_iter()
      .collect::<Vec<_>>();
    pending.sort_unstable_by_key(|(span, _)| span.real_lo());
    pending
  }

  pub(crate) fn record_exported_local(&mut self, name: Atom) {
    self.exported_locals.insert(name);
  }

  fn take_exported_locals(&mut self) -> rustc_hash::FxHashSet<Atom> {
    std::mem::take(&mut self.exported_locals)
  }
}

#[derive(Debug, Default)]
pub struct RequireReferencesState {
  inner: rustc_hash::FxHashMap<Span, RequireReferences>,
}

impl RequireReferencesState {
  pub fn add_require(&mut self, require: Span) {
    self.inner.insert(require, RequireReferences::default());
  }

  fn get_require_mut(&mut self, require: &Span) -> Option<&mut RequireReferences> {
    self.inner.get_mut(require)
  }

  fn get_require_mut_expect(&mut self, require: &Span) -> &mut RequireReferences {
    self.get_require_mut(require).expect("should get require")
  }

  fn take_all_require_references(
    &mut self,
  ) -> impl Iterator<Item = (RequireDependencyLocator, Atom, Vec<ReferencedSpecifier>)> + use<> {
    let inner = std::mem::take(&mut self.inner);
    inner.into_values().filter_map(|value| {
      value.dep_locator.map(|dep_locator| {
        (
          dep_locator,
          value.variable_name.expect("should have variable_name"),
          value.references,
        )
      })
    })
  }
}

#[derive(Debug, Default)]
struct RequireReferences {
  dep_locator: Option<RequireDependencyLocator>,
  variable_name: Option<Atom>,
  references: Vec<ReferencedSpecifier>,
}

impl RequireReferences {
  pub fn add_reference(&mut self, reference: Vec<Atom>) {
    self.references.push(ReferencedSpecifier::new(reference));
  }

  pub fn add_call_reference(&mut self, reference: Vec<Atom>, namespace_object_as_context: bool) {
    self.references.push(ReferencedSpecifier::new_call(
      reference,
      namespace_object_as_context,
    ));
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RequireDependencyLocator {
  dep_idx: usize,
  block_idx: Option<usize>,
  dep_type: DependencyType,
}

#[derive(Clone)]
struct RequireTagData {
  require_span: Span,
}

#[inline(never)]
pub fn is_create_require_import(
  parser: &JavascriptParser,
  source: &Atom,
  export_name: Option<&Atom>,
) -> bool {
  let Some(specifier) = create_require_import_specifier(parser, source) else {
    return false;
  };
  export_name.is_some_and(|export_name| export_name == &specifier)
}

#[inline(never)]
fn create_require_import_specifier(parser: &JavascriptParser, source: &Atom) -> Option<Atom> {
  let option = parser.javascript_options.create_require_option()?;
  let (specifier, module) = option.split_once(" from ")?;
  (!specifier.is_empty()
    && !module.is_empty()
    && (source.as_ref() == module || (module == "module" && source.as_ref() == "node:module")))
    .then(|| specifier.into())
}

#[inline(never)]
pub fn tag_create_require(parser: &mut JavascriptParser, name: Atom) {
  parser.tag_variable_without_data(name, CREATE_REQUIRE_SPECIFIER_TAG);
}

#[inline(never)]
fn is_current_create_require_tag(parser: &JavascriptParser) -> bool {
  parser.current_tag_info.is_some_and(|tag_info| {
    parser.definitions_db.expect_get_tag_info(tag_info).tag == CREATE_REQUIRE_SPECIFIER_TAG
  })
}

#[inline(never)]
pub fn is_create_require_specifier(parser: &mut JavascriptParser, name: &str) -> bool {
  let Some(variable_info) = parser.get_variable_info(name) else {
    return false;
  };
  let mut tag_info_id = variable_info.tag_info;
  while let Some(id) = tag_info_id {
    let tag_info = parser.definitions_db.expect_get_tag_info(id);
    if tag_info.tag == CREATE_REQUIRE_SPECIFIER_TAG {
      return true;
    }
    tag_info_id = tag_info.next;
  }
  false
}

#[cold]
#[inline(never)]
fn should_handle_create_require_specifier(parser: &JavascriptParser, for_name: &str) -> bool {
  for_name == CREATE_REQUIRE_EVALUATED_TAG
    || (for_name == CREATE_REQUIRE_SPECIFIER_TAG && is_current_create_require_tag(parser))
}

#[cold]
#[inline(never)]
fn should_handle_create_require_call(
  parser: &mut JavascriptParser,
  for_name: &str,
  callee: Option<Expr>,
) -> bool {
  should_handle_create_require_specifier(parser, for_name)
    || callee.is_some_and(|callee| is_create_require_namespace_member(parser, callee))
}

#[cold]
#[inline(never)]
fn is_evaluated_create_require(parser: &mut JavascriptParser, expr: Expr) -> bool {
  let evaluated = parser.evaluate_expression(expr);
  evaluated.is_identifier() && evaluated.identifier() == CREATE_REQUIRE_EVALUATED_TAG
}

#[cold]
#[inline(never)]
fn is_create_require_module_object_import(settings: &ESMSpecifierData) -> bool {
  settings.namespace_import
    || (settings.ids.len() == 1
      && settings
        .ids
        .first()
        .is_some_and(|id| id.as_ref() == "default"))
}

#[cold]
#[inline(never)]
pub(crate) fn is_create_require_namespace_member(
  parser: &mut JavascriptParser,
  expr: Expr,
) -> bool {
  let ast = parser.ast.ast;
  let Some(member_expr) = expr.as_member_expression(ast) else {
    return false;
  };
  let Some(namespace) = member_expr.object(ast).as_identifier_reference(ast) else {
    return false;
  };
  let Some(settings) =
    parser.get_tag_data::<ESMSpecifierData>(ast.get_utf8(namespace.name(ast)), ESM_SPECIFIER_TAG)
  else {
    return false;
  };
  let source = settings.source.clone();
  let module_object_import = is_create_require_module_object_import(settings);
  let Some(member) = static_member_name(ast, member_expr) else {
    return false;
  };
  module_object_import
    && create_require_import_specifier(parser, &source).is_some_and(|specifier| member == specifier)
}

#[cold]
#[inline(never)]
fn is_create_require_namespace_member_param(
  parser: &JavascriptParser,
  property: &str,
  param: &BasicEvaluatedExpression,
) -> bool {
  if !param.is_identifier() {
    return false;
  }
  let ExportedVariableInfo::VariableInfo(variable) = param.root_info() else {
    return false;
  };
  let Some(settings) =
    parser.get_variable_tag_data::<ESMSpecifierData>(*variable, ESM_SPECIFIER_TAG)
  else {
    return false;
  };
  is_create_require_module_object_import(settings)
    && create_require_import_specifier(parser, &settings.source)
      .is_some_and(|specifier| property == specifier.as_ref())
}

#[cold]
#[inline(never)]
fn static_member_name(ast: &Ast<'_>, member_expr: MemberExpression) -> Option<Atom> {
  if member_expr.computed(ast) {
    member_property_key_to_atom(ast, member_expr.property(ast))
  } else if let PropertyKeyData::IdentifierName(identifier) =
    ast.property_key_data(member_expr.property(ast))
  {
    Some(Atom::from(ast.get_utf8(identifier.name(ast))))
  } else {
    None
  }
}

#[derive(Clone, Copy)]
struct Arguments<'a, 'ast> {
  ast: &'a Ast<'ast>,
  range: TypedSubRange<Argument>,
}

impl<'a, 'ast> Arguments<'a, 'ast> {
  fn len(self) -> usize {
    self.range.len()
  }

  fn is_empty(self) -> bool {
    self.range.is_empty()
  }

  fn first(self) -> Option<Argument> {
    self.get(0)
  }

  fn get(self, index: usize) -> Option<Argument> {
    self.range.get_node(self.ast, index)
  }

  fn at(self, index: usize) -> Argument {
    self
      .get(index)
      .expect("argument index should be within range")
  }

  fn iter(self) -> impl Iterator<Item = Argument> + 'a {
    self
      .range
      .iter()
      .map(move |id| self.ast.get_node_in_sub_range(id))
  }
}

fn arguments_view<'a, 'ast>(
  ast: &'a Ast<'ast>,
  range: TypedSubRange<Argument>,
) -> Arguments<'a, 'ast> {
  Arguments { ast, range }
}

fn argument_expression(ast: &Ast<'_>, argument: Argument) -> Expr {
  match ast.argument_data(argument) {
    ArgumentData::Expr(expression) => expression,
    ArgumentData::SpreadElement(spread) => spread.argument(ast),
  }
}

#[cold]
#[inline(never)]
fn has_invalid_or_encoded_separator(value: &str) -> bool {
  let bytes = value.as_bytes();
  let mut index = 0;
  while index + 2 < bytes.len() {
    if bytes[index] == b'%' {
      let hi = bytes[index + 1].to_ascii_lowercase();
      let lo = bytes[index + 2].to_ascii_lowercase();
      if !hi.is_ascii_hexdigit() || !lo.is_ascii_hexdigit() {
        return true;
      }
      if hi == b'2' && lo == b'f' || cfg!(windows) && hi == b'5' && lo == b'c' {
        return true;
      }
      index += 3;
    } else {
      index += 1;
    }
  }
  (bytes.len().saturating_sub(index) == 1 || bytes.len().saturating_sub(index) == 2)
    && bytes[index] == b'%'
}

#[cold]
#[inline(never)]
fn file_url_to_path(value: &str) -> Option<(String, bool)> {
  let parsed = Url::parse(value).ok()?;
  if parsed.scheme() != "file" || has_invalid_or_encoded_separator(parsed.path()) {
    return None;
  }
  let is_directory_request = parsed.path().ends_with('/');
  let path = parsed
    .to_file_path()
    .ok()
    .and_then(|path| path.into_os_string().into_string().ok())?;
  Some((path, is_directory_request))
}

#[cold]
#[inline(never)]
fn create_require_context_from_path(value: &str) -> Option<Context> {
  #[cfg(not(windows))]
  {
    let (path, is_directory_request) = if let Some(path) = file_url_to_path(value) {
      path
    } else {
      if !value.starts_with('/') {
        return None;
      }
      (value.to_string(), value.ends_with('/'))
    };
    let context = if is_directory_request {
      let context = path.trim_end_matches('/');
      if context.is_empty() { "/" } else { context }
    } else {
      dirname(&path)?
    };
    Some(Context::new(context.into()))
  }

  #[cfg(windows)]
  {
    let (path, is_directory_request) = if let Some(path) = file_url_to_path(value) {
      path
    } else {
      if !Path::new(value).is_absolute() {
        return None;
      }
      (
        value.to_string(),
        value.ends_with('/') || value.ends_with('\\'),
      )
    };
    let path = Path::new(&path);
    let context = if is_directory_request {
      path
    } else {
      path.parent()?
    };
    let context = if context.parent().is_none() {
      context.to_string_lossy().to_string()
    } else {
      context
        .to_string_lossy()
        .trim_end_matches(['/', '\\'])
        .to_string()
    };
    Some(Context::new(context.into()))
  }
}

#[cfg(not(windows))]
#[cold]
#[inline(never)]
fn dirname(path: &str) -> Option<&str> {
  let path = path.trim_end_matches('/');
  path
    .rfind('/')
    .map(|idx| if idx == 0 { "/" } else { &path[..idx] })
}

#[cold]
#[inline(never)]
fn evaluate_create_require_argument(parser: &mut JavascriptParser, arg: Expr) -> Option<String> {
  let evaluated = parser.evaluate_expression(arg);
  if let Some(value) = evaluated.as_string() {
    return Some(value);
  }

  if let Some(member) = arg.as_member_expression(parser.ast.ast)
    && is_meta_url(parser, member)
  {
    return Url::from_file_path(parser.resource_data.resource())
      .ok()
      .map(|url| url.to_string());
  }

  let ast = parser.ast.ast;
  let new_expr = arg.as_new_expression(ast)?;
  if ast.get_utf8(new_expr.callee(ast).as_identifier_reference(ast)?.name(ast)) != "URL"
    || parser.get_variable_info("URL").is_some()
  {
    return None;
  }
  let args = arguments_view(ast, new_expr.arguments(ast));
  if let Some(first_arg) = args.first().and_then(|arg| arg.as_expr(ast))
    && let Some(value) = parser.evaluate_expression(first_arg).as_string()
    && value.starts_with("file:/")
  {
    if let Some(base) = args.get(1)
      && !is_valid_ignored_url_base_arg(parser, base)
    {
      return None;
    }
    return file_url_to_path(&value).map(|(path, _)| path);
  }
  let (request, _, _) = get_url_request(parser, new_expr)?;
  if request.starts_with("//") {
    let mut value = String::with_capacity("file:".len() + request.len());
    value.push_str("file:");
    value.push_str(&request);
    return Some(value);
  }
  if request.starts_with("file:/") {
    return file_url_to_path(&request).map(|(path, _)| path);
  }
  if !request.starts_with("file:")
    && request
      .find([':', '/', '?', '#'])
      .is_some_and(|idx| request.as_bytes()[idx] == b':')
  {
    return None;
  }
  let request_path = request.split(['?', '#']).next()?;
  if has_invalid_or_encoded_separator(request_path) {
    return None;
  }
  let url = Url::from_file_path(parser.resource_data.resource())
    .ok()?
    .join(&request)
    .ok()?;
  if url.scheme() != "file" {
    return None;
  }
  file_url_to_path(url.as_str()).map(|(path, _)| path)
}

#[cold]
#[inline(never)]
fn ignored_url_args_are_side_effect_free_from(
  parser: &mut JavascriptParser,
  args: &Arguments<'_, '_>,
  start: usize,
) -> bool {
  let ast = parser.ast.ast;
  args.iter().skip(start).all(|arg| {
    arg
      .as_expr(ast)
      .is_some_and(|expr| is_side_effect_free_ignored_url_arg(parser, expr))
  })
}

#[cold]
#[inline(never)]
fn is_side_effect_free_ignored_url_arg(parser: &mut JavascriptParser, expr: Expr) -> bool {
  let ast = parser.ast.ast;
  match ast.expr_data(expr) {
    ExprData::StringLiteral(_)
    | ExprData::NumericLiteral(_)
    | ExprData::BigIntLiteral(_)
    | ExprData::BooleanLiteral(_)
    | ExprData::NullLiteral(_)
    | ExprData::RegExpLiteral(_) => true,
    ExprData::IdentifierReference(ident) => {
      ast.get_utf8(ident.name(ast)) == "undefined"
        && parser.get_variable_info("undefined").is_none()
    }
    ExprData::UnaryExpression(unary) if unary.operator(ast) == UnaryOperator::Void => {
      is_side_effect_free_ignored_url_arg(parser, unary.argument(ast))
    }
    _ => false,
  }
}

#[cold]
#[inline(never)]
fn parse_create_require_argument(
  parser: &mut JavascriptParser,
  call_expr: CallExpression,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  let ast = parser.ast.ast;
  let args = arguments_view(ast, call_expr.arguments(ast));
  parse_create_require_argument_from_args(parser, &args, call_expr.span(ast), emit_warning)
}

#[cold]
#[inline(never)]
fn parse_create_require_argument_from_args(
  parser: &mut JavascriptParser,
  args: &Arguments<'_, '_>,
  span: Span,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  if args.is_empty() {
    if emit_warning {
      add_create_require_warning(parser, "module.createRequire requires one argument.", span);
    }
    return None;
  }

  let ast = parser.ast.ast;
  let first_arg = args.at(0);
  let Some(arg) = first_arg.as_expr(ast) else {
    if emit_warning {
      add_create_require_warning(
        parser,
        "module.createRequire does not support spread arguments.",
        first_arg.span(ast),
      );
    }
    return None;
  };

  let Some(value) = evaluate_create_require_argument(parser, arg) else {
    if emit_warning {
      add_create_require_warning(
        parser,
        "module.createRequire failed parsing argument.",
        arg.span(parser.ast.ast),
      );
    }
    return None;
  };
  let context = create_require_context_from_path(&value);
  if context.is_none() && emit_warning {
    add_create_require_warning(
      parser,
      "module.createRequire supports only file URLs and absolute paths.",
      arg.span(parser.ast.ast),
    );
  }
  Some(CreateRequireArgument {
    value,
    context: context?,
    replace_argument: should_replace_create_require_argument(parser, arg),
  })
}

#[cold]
#[inline(never)]
fn parse_create_require_new_argument(
  parser: &mut JavascriptParser,
  new_expr: NewExpression,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  let ast = parser.ast.ast;
  let args = arguments_view(ast, new_expr.arguments(ast));
  parse_create_require_argument_from_args(parser, &args, new_expr.span(ast), emit_warning)
}

#[inline(never)]
fn should_replace_create_require_argument(parser: &mut JavascriptParser, arg: Expr) -> bool {
  let ast = parser.ast.ast;
  if let Some(member) = arg.as_member_expression(ast)
    && is_meta_url(parser, member)
  {
    return parser
      .javascript_options
      .import_meta()
      .is_known_property_enabled(ImportMetaKnownProperties::URL);
  }
  let Some(new_expr) = arg.as_new_expression(ast) else {
    return true;
  };
  if new_expr
    .callee(ast)
    .as_identifier_reference(ast)
    .is_some_and(|ident| ast.get_utf8(ident.name(ast)) == "URL")
    && parser.get_variable_info("URL").is_none()
  {
    let is_absolute_file_url = is_absolute_file_url_constructor_arg(parser, arg);
    let start = if is_absolute_file_url { 1 } else { 2 };
    let args = arguments_view(ast, new_expr.arguments(ast));
    if is_absolute_file_url
      && let Some(base) = args.get(1)
      && !is_valid_ignored_url_base_arg(parser, base)
    {
      return false;
    }
    ignored_url_args_are_side_effect_free_from(parser, &args, start)
  } else {
    true
  }
}

#[inline(never)]
fn can_defer_create_require_call(parser: &mut JavascriptParser, args: &Arguments<'_, '_>) -> bool {
  let ast = parser.ast.ast;
  args.len() == 1
    && args
      .at(0)
      .as_expr(ast)
      .and_then(|expr| expr.as_member_expression(ast))
      .is_some_and(|member| is_meta_url(parser, member))
}

#[inline(never)]
fn should_clear_create_require_call(
  parser: &mut JavascriptParser,
  args: &Arguments<'_, '_>,
) -> bool {
  !matches!(parser.javascript_options.require_resolve, Some(false))
    && can_defer_create_require_call(parser, args)
}

#[inline(never)]
fn clear_create_require_call(parser: &mut JavascriptParser, span: Span) {
  parser.add_presentational_dependency(Arc::new(ConstDependency::new(
    span.into(),
    "/* createRequire() */ undefined".into(),
  )));
}

#[inline(never)]
fn is_valid_ignored_url_base_arg(parser: &mut JavascriptParser, base: Argument) -> bool {
  let ast = parser.ast.ast;
  let Some(base) = base.as_expr(ast) else {
    return false;
  };
  if let Some(ident) = base.as_identifier_reference(ast)
    && ast.get_utf8(ident.name(ast)) == "undefined"
    && parser.get_variable_info("undefined").is_none()
  {
    return true;
  }
  if let Some(unary) = base.as_unary_expression(ast)
    && unary.operator(ast) == UnaryOperator::Void
    && is_side_effect_free_ignored_url_arg(parser, unary.argument(ast))
  {
    return true;
  }
  parser
    .evaluate_expression(base)
    .as_string()
    .is_some_and(|base| Url::parse(&base).is_ok())
}

#[inline(never)]
fn is_absolute_file_url_constructor_arg(parser: &mut JavascriptParser, arg: Expr) -> bool {
  let ast = parser.ast.ast;
  let Some(new_expr) = arg.as_new_expression(ast) else {
    return false;
  };
  if new_expr
    .callee(ast)
    .as_identifier_reference(ast)
    .is_none_or(|ident| ast.get_utf8(ident.name(ast)) != "URL")
    || parser.get_variable_info("URL").is_some()
  {
    return false;
  };
  let args = arguments_view(ast, new_expr.arguments(ast));
  args
    .first()
    .and_then(|arg| arg.as_expr(ast))
    .and_then(|arg| parser.evaluate_expression(arg).as_string())
    .is_some_and(|value| value.starts_with("file:/"))
}

#[inline(never)]
fn walk_create_require_callee(parser: &mut JavascriptParser, call_expr: CallExpression) {
  parser.walk_expression(call_expr.callee(parser.ast.ast));
}

fn walk_create_require_ignored_args(parser: &mut JavascriptParser, call_expr: CallExpression) {
  let ast = parser.ast.ast;
  let args = arguments_view(ast, call_expr.arguments(ast));
  if args.len() > 1 {
    parser.walk_arguments(args.iter().skip(1));
  }
}

#[inline(never)]
fn is_unbound_url_constructor(parser: &mut JavascriptParser, callee: Expr) -> bool {
  callee
    .as_identifier_reference(parser.ast.ast)
    .is_some_and(|ident| parser.ast.ast.get_utf8(ident.name(parser.ast.ast)) == "URL")
    && parser.get_variable_info("URL").is_none()
}

#[inline(never)]
fn walk_create_require_argument_side_effects(parser: &mut JavascriptParser, arg: Expr) {
  let ast = parser.ast.ast;
  let Some(new_expr) = arg.as_new_expression(ast) else {
    return;
  };
  if !is_unbound_url_constructor(parser, new_expr.callee(ast)) {
    return;
  };
  let args = arguments_view(ast, new_expr.arguments(ast));
  if args.len() > 1 {
    parser.walk_arguments(args.iter().skip(1));
  }
}

#[inline(never)]
fn source_for_span(parser: &JavascriptParser, span: Span) -> Option<String> {
  parser
    .source()
    .get(span.real_lo() as usize..span.real_hi() as usize)
    .map(str::to_string)
}

#[inline(never)]
fn push_side_effect(side_effects: &mut String, source: &str) {
  if !side_effects.is_empty() {
    side_effects.push_str(", ");
  }
  side_effects.push_str(source);
}

#[inline(never)]
fn push_spread_side_effect(side_effects: &mut String, source: &str) {
  if !side_effects.is_empty() {
    side_effects.push_str(", ");
  }
  side_effects.push_str("[...(");
  side_effects.push_str(source);
  side_effects.push_str(")]");
}

#[inline(never)]
fn side_effects_with_suffix(side_effects: &str, suffix: &str) -> Box<str> {
  let mut replacement = String::with_capacity(side_effects.len() + suffix.len() + 3);
  replacement.push('(');
  replacement.push_str(side_effects);
  replacement.push_str(", ");
  replacement.push_str(suffix);
  replacement.into_boxed_str()
}

#[inline(never)]
fn create_require_url_arg_side_effects(parser: &mut JavascriptParser, arg: Expr) -> String {
  let ast = parser.ast.ast;
  let Some(new_expr) = arg.as_new_expression(ast) else {
    return String::new();
  };
  if !is_unbound_url_constructor(parser, new_expr.callee(ast)) {
    return String::new();
  };
  let args = arguments_view(ast, new_expr.arguments(ast));
  let start = if is_absolute_file_url_constructor_arg(parser, arg) {
    1
  } else {
    2
  };
  let mut side_effects = String::new();
  for argument in args.iter().skip(start) {
    let expression = argument_expression(ast, argument);
    let Some(source) = source_for_span(parser, expression.span(ast)) else {
      continue;
    };
    if argument.as_expr(ast).is_none() {
      push_spread_side_effect(&mut side_effects, &source);
    } else if !is_side_effect_free_ignored_url_arg(parser, expression)
      && !expression
        .as_member_expression(ast)
        .is_some_and(|expr| is_meta_url(parser, expr))
    {
      push_side_effect(&mut side_effects, &source);
    }
  }
  side_effects
}

#[inline(never)]
fn create_require_unsupported_member_replacement(side_effects: &str) -> Box<str> {
  if side_effects.is_empty() {
    "undefined".into()
  } else {
    side_effects_with_suffix(side_effects, "undefined)")
  }
}

#[inline(never)]
fn wrap_span_with_side_effects(parser: &mut JavascriptParser, span: Span, side_effects: &str) {
  if side_effects.is_empty() {
    return;
  }
  parser.add_presentational_dependency(Arc::new(ConstDependency::new(
    (span.real_lo(), span.real_lo()).into(),
    side_effects_with_suffix(side_effects, ""),
  )));
  parser.add_presentational_dependency(Arc::new(ConstDependency::new(
    (span.real_hi(), span.real_hi()).into(),
    ")".into(),
  )));
}

#[inline(never)]
fn create_require_extra_arg_side_effects(
  parser: &JavascriptParser,
  args: &Arguments<'_, '_>,
) -> String {
  let ast = parser.ast.ast;
  let mut side_effects = String::new();
  for argument in args.iter().skip(1) {
    let expression = argument_expression(ast, argument);
    let Some(source) = source_for_span(parser, expression.span(ast)) else {
      continue;
    };
    if argument.as_expr(ast).is_none() {
      push_spread_side_effect(&mut side_effects, &source);
    } else {
      push_side_effect(&mut side_effects, &source);
    }
  }
  side_effects
}

#[inline(never)]
fn create_require_args_side_effects(
  parser: &mut JavascriptParser,
  args: &Arguments<'_, '_>,
  argument: &CreateRequireArgument,
) -> String {
  let mut side_effects = if argument.replace_argument {
    String::new()
  } else {
    create_require_url_arg_side_effects(parser, argument_expression(parser.ast.ast, args.at(0)))
  };
  let extra_side_effects = create_require_extra_arg_side_effects(parser, args);
  if !extra_side_effects.is_empty() {
    push_side_effect(&mut side_effects, &extra_side_effects);
  }
  side_effects
}

#[inline(never)]
fn evaluate_created_require<'p>(
  parser: &mut JavascriptParser<'p>,
  range: Span,
  args: &Arguments<'_, '_>,
  argument: CreateRequireArgument,
) -> BasicEvaluatedExpression<'p> {
  let side_effects = create_require_args_side_effects(parser, args, &argument);
  let has_side_effects = !side_effects.is_empty();
  let evaluated_name = Atom::from(range.real_lo().to_string());
  parser.tag_variable(
    evaluated_name.clone(),
    CREATED_REQUIRE_IDENTIFIER_TAG,
    Some(CreatedRequireTagData {
      context: argument.context,
      side_effects,
      pending_call: None,
      preserve_unhandled: false,
    }),
  );
  let mut evaluated = BasicEvaluatedExpression::with_range(range.real_lo(), range.real_hi());
  evaluated.set_identifier(
    evaluated_name.clone(),
    ExportedVariableInfo::Name(evaluated_name),
    None,
    None,
    None,
  );
  evaluated.set_side_effects(has_side_effects);
  evaluated.set_truthy();
  evaluated
}

#[inline(never)]
pub(crate) fn evaluate_create_require_new_expression<'p>(
  parser: &mut JavascriptParser<'p>,
  for_name: &str,
  callee: Option<Expr>,
  expr: NewExpression,
) -> Option<BasicEvaluatedExpression<'p>> {
  if !should_handle_create_require_call(parser, for_name, callee) {
    return None;
  }
  let argument = parse_create_require_new_argument(parser, expr, false)?;
  let ast = parser.ast.ast;
  let args = arguments_view(ast, expr.arguments(ast));
  Some(evaluate_created_require(
    parser,
    expr.span(ast),
    &args,
    argument,
  ))
}

#[inline(never)]
fn evaluate_create_require_call_expression<'p>(
  parser: &mut JavascriptParser<'p>,
  expr: CallExpression,
) -> Option<BasicEvaluatedExpression<'p>> {
  let argument = parse_create_require_argument(parser, expr, false)?;
  let ast = parser.ast.ast;
  let args = arguments_view(ast, expr.arguments(ast));
  Some(evaluate_created_require(
    parser,
    expr.span(ast),
    &args,
    argument,
  ))
}

#[inline(never)]
fn current_created_require_side_effects(parser: &mut JavascriptParser) -> String {
  parser
    .current_tag_info
    .and_then(|tag_info| {
      parser
        .definitions_db
        .expect_get_tag_info(tag_info)
        .data
        .clone()
    })
    .map(CreatedRequireTagData::downcast)
    .map(|data| data.side_effects)
    .unwrap_or_default()
}

#[inline(never)]
fn wrap_created_require_with_side_effects(parser: &mut JavascriptParser, span: Span) {
  let side_effects = current_created_require_side_effects(parser);
  wrap_span_with_side_effects(parser, span, &side_effects);
}

#[cold]
#[inline(never)]
fn add_create_require_warning(parser: &mut JavascriptParser, message: &str, span: Span) {
  let mut error = create_traceable_error(
    "Unsupported feature".into(),
    message.to_string(),
    parser.source.to_string(),
    span.into(),
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  parser.add_warning(error.into());
}

#[cold]
#[inline(never)]
fn add_unsupported_create_require_member_warning(parser: &mut JavascriptParser, span: Span) {
  add_create_require_warning(
    parser,
    "The accessed createRequire() member is not supported by Rspack.",
    span,
  );
}

fn deferred_create_require_callee(
  parser: &mut JavascriptParser,
  callee: Expr,
  call_span: Span,
) -> Option<DeferredCreateRequireCallee> {
  let ast = parser.ast.ast;
  let (settings, range, ids, direct_import, ns_access) =
    if let Some(ident) = callee.as_identifier_reference(ast) {
      let settings = parser
        .get_tag_data::<ESMSpecifierData>(ast.get_utf8(ident.name(ast)), ESM_SPECIFIER_TAG)?
        .clone();
      let ids = settings.ids.clone().into_vec();
      (settings, ident.span(ast).into(), ids, true, false)
    } else {
      let member = callee.as_member_expression(ast)?;
      let namespace = member.object(ast).as_identifier_reference(ast)?;
      let settings = parser
        .get_tag_data::<ESMSpecifierData>(ast.get_utf8(namespace.name(ast)), ESM_SPECIFIER_TAG)?
        .clone();
      let mut ids = settings.ids.clone().into_vec();
      ids.push(static_member_name(ast, member)?);
      let ns_access = settings.namespace_import && !ids.is_empty();
      (settings, callee.span(ast).into(), ids, false, ns_access)
    };
  Some(DeferredCreateRequireCallee {
    settings,
    range,
    ids,
    asi_safe: !parser.is_asi_position(call_span.real_lo()),
    direct_import,
    ns_access,
    branch_guard: parser.current_branch_guard.clone(),
  })
}

fn add_deferred_create_require_callee_dependency(
  parser: &mut JavascriptParser,
  callee: DeferredCreateRequireCallee,
) {
  let DeferredCreateRequireCallee {
    settings,
    range,
    ids,
    asi_safe,
    direct_import,
    ns_access,
    branch_guard,
  } = callee;
  let mut dep = ESMImportSpecifierDependency::new(
    settings.source,
    settings.name,
    settings.source_order,
    false,
    asi_safe,
    range,
    ids,
    true,
    direct_import,
    ns_access,
    ESMImportSpecifierDependency::create_export_presence_mode(parser.javascript_options),
    None,
    settings.phase,
    settings.attributes,
    parser.to_dependency_location(range),
  );
  dep.namespace_object_as_context = parser
    .javascript_options
    .strict_this_context_on_imports
    .unwrap_or(false)
    && !direct_import;
  if let Some(branch_guard) = branch_guard {
    dep.set_branch_guard(branch_guard);
  }
  let dep_idx = parser.next_dependency_idx();
  parser.add_dependency(BoxDependency::new(dep));
  InnerGraphParserPlugin::on_usage(
    parser,
    InnerGraphUsageOperation::ESMImportSpecifier(dep_idx),
  );
}

fn keep_deferred_create_require_call(
  parser: &mut JavascriptParser,
  pending: PendingCreatedRequire,
) {
  add_deferred_create_require_callee_dependency(parser, pending.callee);
  // Let the regular parser plugins own children of a call that survives.
  let statement_path = std::mem::replace(&mut parser.statement_path, pending.statement_path);
  let prev_statement = std::mem::replace(&mut parser.prev_statement, pending.prev_statement);
  parser.walk_expression(pending.argument);
  parser.statement_path = statement_path;
  parser.prev_statement = prev_statement;
}

fn pre_tag_created_require_declarator(
  parser: &mut JavascriptParser,
  declarator: VariableDeclarator,
  declaration: VariableDeclaration,
) {
  // Register metadata only; the normal declarator walk still creates or clears dependencies.
  let ast = parser.ast.ast;
  if declaration.kind(ast) != VariableDeclarationKind::Const {
    return;
  }
  let Some(binding) = declarator.id(ast).as_binding_identifier(ast) else {
    return;
  };
  let Some(call) = declarator
    .init(ast)
    .and_then(|expr| expr.as_call_expression(ast))
  else {
    return;
  };
  let callee = call.callee(ast);
  let is_create_require_callee = callee
    .as_identifier_reference(ast)
    .is_some_and(|ident| is_create_require_specifier(parser, ast.get_utf8(ident.name(ast))))
    || is_create_require_namespace_member(parser, callee);
  let args = arguments_view(ast, call.arguments(ast));
  if !is_create_require_callee || !can_defer_create_require_call(parser, &args) {
    return;
  }
  let Some(argument) = parse_create_require_argument(parser, call, false) else {
    return;
  };
  let call_span = call.span(ast);
  let Some(deferred_callee) = deferred_create_require_callee(parser, callee, call_span) else {
    return;
  };
  let CreateRequireArgument {
    value: _,
    context,
    replace_argument: _,
  } = argument;
  let name = Atom::from(ast.get_utf8(binding.name(ast)));
  parser.define_variable(name.clone());
  parser.tag_variable(
    name,
    CREATED_REQUIRE_IDENTIFIER_TAG,
    Some(CreatedRequireTagData {
      context,
      side_effects: String::new(),
      pending_call: Some(call_span),
      preserve_unhandled: true,
    }),
  );
  let statement_path = parser.statement_path.clone();
  let prev_statement = parser.prev_statement;
  parser.created_require_references.add_pending(
    call_span,
    deferred_callee,
    argument_expression(parser.ast.ast, args.at(0)),
    statement_path,
    prev_statement,
  );
}

#[cold]
#[inline(never)]
fn tag_created_require_declarator(
  parser: &mut JavascriptParser,
  binding: swc_next_ecma_ast::BindingIdentifier,
  call_span: Span,
  clear_call: bool,
  args: &Arguments<'_, '_>,
  deferred_callee: Option<DeferredCreateRequireCallee>,
  argument: CreateRequireArgument,
) {
  let CreateRequireArgument {
    value,
    context,
    replace_argument,
  } = argument;
  let deferred = deferred_callee.is_some();
  let binding_name = Atom::from(parser.ast.ast.get_utf8(binding.name(parser.ast.ast)));
  parser.define_variable(binding_name.clone());
  parser.tag_variable(
    binding_name,
    CREATED_REQUIRE_IDENTIFIER_TAG,
    Some(CreatedRequireTagData {
      context,
      side_effects: String::new(),
      pending_call: deferred.then_some(call_span),
      preserve_unhandled: deferred || !clear_call,
    }),
  );
  if let Some(callee) = deferred_callee {
    let statement_path = parser.statement_path.clone();
    let prev_statement = parser.prev_statement;
    parser.created_require_references.add_pending(
      call_span,
      callee,
      argument_expression(parser.ast.ast, args.at(0)),
      statement_path,
      prev_statement,
    );
  } else if clear_call {
    clear_create_require_call(parser, call_span);
  } else if replace_argument {
    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      argument_expression(parser.ast.ast, args.at(0))
        .span(parser.ast.ast)
        .into(),
      json_stringify_str(&value).into(),
    )));
  } else {
    walk_create_require_argument_side_effects(
      parser,
      argument_expression(parser.ast.ast, args.at(0)),
    );
  }
  parser.walk_arguments(args.iter().skip(1));
}

fn clear_create_require_tag(parser: &mut JavascriptParser, name: &str) {
  if let Some(declared_scope) = parser
    .get_variable_info(name)
    .map(|info| info.declared_scope)
  {
    let info = VariableInfo::create(
      &mut parser.definitions_db,
      declared_scope,
      None,
      VariableInfoFlags::NORMAL,
      None,
    );
    parser
      .definitions_db
      .set(declared_scope, Atom::from(name), info);
  }
}

#[inline(never)]
fn add_require_cache_dependency(parser: &mut JavascriptParser, range: DependencyRange) {
  parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
    range,
    RuntimeGlobals::MODULE_CACHE,
  )));
}

#[inline(never)]
fn require_cache_range(
  ast: &Ast<'_>,
  member_expr: HookMemberExpression,
  member_ranges: &[Span],
  members: &[Atom],
) -> Span {
  if members.len() > 1 {
    member_ranges[1]
  } else {
    member_expr.span(ast)
  }
}

#[inline(never)]
fn handle_created_require_member(
  parser: &mut JavascriptParser,
  member_span: Span,
  cache_range: Span,
  members: &[Atom],
  unsupported_replacement: Box<str>,
) {
  if members
    .first()
    .is_some_and(|member| member.as_ref() == "cache")
  {
    add_require_cache_dependency(parser, cache_range.into());
  } else {
    add_unsupported_create_require_member_warning(parser, member_span);
    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      member_span.into(),
      unsupported_replacement,
    )));
  }
}

#[inline(never)]
fn current_created_require_context(parser: &JavascriptParser) -> Option<Context> {
  parser
    .current_tag_info
    .and_then(|tag_info| {
      parser
        .definitions_db
        .expect_get_tag_info(tag_info)
        .data
        .clone()
    })
    .map(CreatedRequireTagData::downcast)
    .map(|data| data.context)
}

fn preserve_unhandled_created_require(parser: &mut JavascriptParser) -> bool {
  let data = parser
    .current_tag_info
    .and_then(|tag_info| {
      parser
        .definitions_db
        .expect_get_tag_info(tag_info)
        .data
        .as_deref()
    })
    .map(CreatedRequireTagData::downcast_ref);
  let Some(data) = data.filter(|data| data.preserve_unhandled) else {
    return false;
  };
  if let Some(call_span) = data.pending_call {
    parser.created_require_references.mark_must_keep(call_span);
  }
  true
}

#[cold]
#[inline(never)]
fn walk_unsupported_create_require_resolve(
  parser: &mut JavascriptParser,
  inner_call_expr: CallExpression,
  call_expr: CallExpression,
) {
  walk_create_require_callee(parser, inner_call_expr);
  let ast = parser.ast.ast;
  let inner_args = arguments_view(ast, inner_call_expr.arguments(ast));
  if inner_args.len() == 1
    && let Some(arg) = inner_args.at(0).as_expr(ast)
  {
    if let Some(value) = evaluate_create_require_argument(parser, arg) {
      if should_replace_create_require_argument(parser, arg) {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          arg.span(parser.ast.ast).into(),
          json_stringify_str(&value).into(),
        )));
      } else {
        walk_create_require_argument_side_effects(parser, arg);
      }
    } else if let Some(new_expr) = arg.as_new_expression(parser.ast.ast)
      && is_unbound_url_constructor(parser, new_expr.callee(parser.ast.ast))
      && let args = arguments_view(parser.ast.ast, new_expr.arguments(parser.ast.ast))
      && args.len() > 2
    {
      if get_url_request(parser, new_expr).is_some() {
        parser.walk_arguments(args.iter().skip(2));
      } else {
        parser.walk_arguments(args.iter());
      }
    } else {
      parser.walk_expression(arg);
    }
  } else {
    parser.walk_arguments(inner_args.iter());
  }
  let ast = parser.ast.ast;
  parser.walk_arguments(
    call_expr
      .arguments(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id)),
  );
}

fn tag_commonjs_require_referenced(
  parser: &mut JavascriptParser,
  require_call: CallExpression,
  variable_name: Atom,
) {
  let require_span = require_call.span(parser.ast.ast);
  parser
    .common_js_require_references
    .add_require(require_span);
  parser
    .common_js_require_references
    .get_require_mut_expect(&require_span)
    .variable_name = Some(variable_name.clone());
  parser.tag_variable(
    variable_name,
    COMMONJS_REQUIRE_TAG,
    Some(RequireTagData { require_span }),
  );
}

fn create_commonjs_require_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  call_expr: CallExpression,
  arg_expr: Expr,
  referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  request_context: Option<rspack_core::Context>,
) -> CommonJsRequireContextDependency {
  let result = create_context_dependency(param, parser);
  let request = result.request();

  let span = call_expr.span(parser.ast.ast);
  let options = ContextOptions {
    mode: ContextMode::Sync,
    recursive: true,
    pattern: context_reg_exp(&result.reg, "", None, parser).into(),
    category: DependencyCategory::CommonJS,
    request,
    context: get_context(parser.resource_data).to_string(),
    compiler_context: parser.compiler_options.context.clone(),
    replaces: result.replaces,
    start: span.real_lo(),
    end: span.real_hi(),
    ..Default::default()
  };
  let range = span.into();
  let loc = parser
    .to_dependency_location(range)
    .expect("Should get correct loc");
  let mut dep = CommonJsRequireContextDependency::new(
    options,
    loc,
    range,
    Some(arg_expr.span(parser.ast.ast).into()),
    parser.in_try,
    request_context,
  );
  if let Some(referenced_specifiers) = referenced_specifiers {
    dep.set_referenced_specifiers(referenced_specifiers);
  }
  dep.set_critical(result.critical);
  dep
}

fn create_require_resolve_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  range: DependencyRange,
  weak: bool,
  request_context: Option<rspack_core::Context>,
) -> RequireResolveContextDependency {
  let start = range.start;
  let end = range.end;

  let result = create_context_dependency(param, parser);
  let request = result.request();

  let options = ContextOptions {
    mode: if weak {
      ContextMode::Weak
    } else {
      ContextMode::Sync
    },
    recursive: true,
    pattern: context_reg_exp(&result.reg, "", None, parser).into(),
    category: DependencyCategory::CommonJS,
    request,
    context: get_context(parser.resource_data).to_string(),
    compiler_context: parser.compiler_options.context.clone(),
    replaces: result.replaces,
    start,
    end,
    ..Default::default()
  };
  RequireResolveContextDependency::new(options, range, parser.in_try, request_context)
}

pub(crate) fn is_require_call_expr(parser: &mut JavascriptParser, call: CallExpression) -> bool {
  if !should_parse_commonjs_require(parser) {
    return false;
  }

  let ast = parser.ast.ast;
  if call.arguments(ast).len() != 1 {
    return false;
  }
  let callee = call.callee(ast);

  if let Some(ident) = callee.as_identifier_reference(ast) {
    return Atom::from(ast.get_utf8(ident.name(ast)))
      .call_hooks_name(parser, |_, for_name| {
        (for_name == expr_name::REQUIRE).then_some(true)
      })
      .unwrap_or_default();
  }

  if let Some(member) = callee.as_member_expression(parser.ast.ast) {
    return member
      .call_hooks_name(parser, |_, for_name| {
        (for_name == expr_name::MODULE_REQUIRE).then_some(true)
      })
      .unwrap_or_default();
  }

  false
}

fn should_parse_commonjs_require(parser: &JavascriptParser) -> bool {
  matches!(
    parser.module_type,
    ModuleType::JsAuto | ModuleType::JsDynamic
  )
}

#[derive(Clone, Copy)]
enum CallOrNewExpression {
  Call(CallExpression),
  New(NewExpression),
}

impl CallOrNewExpression {
  pub fn callee(self, ast: &Ast<'_>) -> Expr {
    match self {
      CallOrNewExpression::Call(call_expr) => call_expr.callee(ast),
      CallOrNewExpression::New(new_expr) => new_expr.callee(ast),
    }
  }

  pub fn args<'a, 'ast>(self, ast: &'a Ast<'ast>) -> Arguments<'a, 'ast> {
    match self {
      CallOrNewExpression::Call(call_expr) => arguments_view(ast, call_expr.arguments(ast)),
      CallOrNewExpression::New(new_expr) => arguments_view(ast, new_expr.arguments(ast)),
    }
  }

  pub fn span(self, ast: &Ast<'_>) -> Span {
    match self {
      CallOrNewExpression::Call(call_expr) => call_expr.span(ast),
      CallOrNewExpression::New(new_expr) => new_expr.span(ast),
    }
  }
}

pub struct CommonJsImportsParserPlugin;

impl CommonJsImportsParserPlugin {
  fn has_ignore_comment(parser: &mut JavascriptParser, error_span: Span, span: Span) -> bool {
    if !parser
      .javascript_options
      .commonjs_magic_comments
      .unwrap_or(false)
    {
      return false;
    }

    try_extract_magic_comment(parser, error_span, span)
      .get_ignore()
      .unwrap_or_default()
  }

  fn should_process_resolve(parser: &mut JavascriptParser, call_expr: CallExpression) -> bool {
    let ast = parser.ast.ast;
    let Some(member_expr) = call_expr.callee(ast).as_member_expression(ast) else {
      return false;
    };
    let Some(ident) = member_expr.object(ast).as_identifier_reference(ast) else {
      return false;
    };

    if parser
      .get_variable_info(ast.get_utf8(ident.name(ast)))
      .is_some()
    {
      return false;
    }

    true
  }

  fn process_resolve(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    weak: bool,
    request_context: Option<Context>,
  ) {
    let ast = parser.ast.ast;
    let args = arguments_view(ast, call_expr.arguments(ast));
    if args.len() != 1 {
      return;
    }

    if let Some(argument_expr) = args.at(0).as_expr(ast)
      && Self::has_ignore_comment(parser, call_expr.span(ast), argument_expr.span(ast))
    {
      return;
    }

    let argument_expr = argument_expression(ast, args.at(0));
    let param = parser.evaluate_expression(argument_expr);
    let range = call_expr.callee(parser.ast.ast).span(parser.ast.ast).into();
    let loc = parser.to_dependency_location(range);
    let require_resolve_header_dependency =
      BoxDependency::new(RequireResolveHeaderDependency::new(range, loc));

    if param.is_conditional() {
      for option in param.options() {
        if !self.process_resolve_item(parser, option, weak, request_context.clone()) {
          self.process_resolve_context(parser, option, weak, request_context.clone());
        }
      }
      parser.add_dependency(require_resolve_header_dependency);
    } else {
      if !self.process_resolve_item(parser, &param, weak, request_context.clone()) {
        self.process_resolve_context(parser, &param, weak, request_context);
      }
      parser.add_dependency(require_resolve_header_dependency);
    }
  }

  fn process_created_require_resolve_call(
    &self,
    parser: &mut JavascriptParser,
    expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = arguments_view(ast, expr.arguments(ast));
    if args.len() != 1 || args.at(0).as_expr(ast).is_none() {
      preserve_unhandled_created_require(parser);
      parser.walk_arguments(args.iter());
      return Some(true);
    }
    if matches!(parser.javascript_options.require_resolve, Some(false)) {
      preserve_unhandled_created_require(parser);
      parser.walk_arguments(args.iter());
      return Some(true);
    }
    self.process_resolve(parser, expr, false, current_created_require_context(parser));
    Some(true)
  }

  fn process_resolve_item(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
    weak: bool,
    request_context: Option<rspack_core::Context>,
  ) -> bool {
    if param.is_string() {
      if let Some(context) = request_context {
        parser.add_dependency(BoxDependency::new(
          RequireResolveDependency::new_contextual(
            param.string().clone(),
            param.range().into(),
            weak,
            parser.in_try,
            context,
          ),
        ));
      } else {
        parser.add_dependency(BoxDependency::new(RequireResolveDependency::new(
          param.string().clone(),
          param.range().into(),
          weak,
          parser.in_try,
        )));
      }

      return true;
    }

    false
  }

  fn process_resolve_context(
    &self,
    parser: &mut JavascriptParser,
    param: &BasicEvaluatedExpression,
    weak: bool,
    request_context: Option<rspack_core::Context>,
  ) {
    let dep = create_require_resolve_context_dependency(
      parser,
      param,
      param.range().into(),
      weak,
      request_context,
    );

    parser.add_dependency(BoxDependency::new(dep));
  }

  fn chain_handler(
    &self,
    parser: &mut JavascriptParser,
    member_expr: MemberExpression,
    call_expr: CallExpression,
    members: &[Atom],
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    let ast = parser.ast.ast;
    let args = arguments_view(ast, call_expr.arguments(ast));
    if args.len() != 1 {
      return None;
    }
    let arg = args.at(0);
    if let Some(argument_expr) = arg.as_expr(ast)
      && Self::has_ignore_comment(parser, call_expr.span(ast), argument_expr.span(ast))
    {
      return None;
    }
    let param = parser.evaluate_expression(argument_expression(ast, arg));
    let member_span = member_expr.span(parser.ast.ast);
    let range = DependencyRange::from(member_span);
    let loc = parser.to_dependency_location(range);
    param.is_string().then(|| {
      CommonJsFullRequireDependency::new(
        param.string().to_owned(),
        members.to_vec(),
        member_span.into(),
        loc,
        is_call,
        parser
          .javascript_options
          .strict_this_context_on_imports
          .unwrap_or(false)
          && !members.is_empty(),
        parser.in_try,
        !parser.is_asi_position(member_span.start),
      )
    })
  }

  fn process_require_item(
    &self,
    parser: &mut JavascriptParser,
    span: Span,
    param: &BasicEvaluatedExpression,
    request_context: Option<Context>,
  ) -> Option<bool> {
    param.is_string().then(|| {
      let (start, end) = param.range();
      let range_expr = DependencyRange::new(start, end);
      let loc = parser.to_dependency_location(range_expr);
      let referenced_specifiers =
        parser
          .destructuring_assignment_properties
          .get(&span)
          .map(|keys| {
            let mut refs = Vec::new();
            keys.traverse_on_leaf(&mut |stack| {
              let names = stack.iter().map(|p| p.id.clone()).collect();
              refs.push(ReferencedSpecifier::new(names));
            });
            refs
          });
      let mut dep = if let Some(context) = request_context {
        CommonJsRequireDependency::new_contextual(
          param.string().clone(),
          range_expr,
          Some(span.into()),
          parser.in_try,
          context,
          loc,
        )
      } else {
        CommonJsRequireDependency::new(
          param.string().clone(),
          range_expr,
          Some(span.into()),
          parser.in_try,
          loc,
        )
      };
      if let Some(referenced_specifiers) = referenced_specifiers {
        dep.set_referenced_specifiers(referenced_specifiers);
      }
      let dep_idx = parser.next_dependency_idx();
      if let Some(require_references) = parser.common_js_require_references.get_require_mut(&span) {
        require_references.dep_locator = Some(RequireDependencyLocator {
          dep_idx,
          block_idx: parser.collecting_dependencies_for_block,
          dep_type: DependencyType::CjsRequire,
        });
      }
      parser.add_dependency(BoxDependency::new(dep));
      true
    })
  }

  fn process_require_context(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
    request_context: Option<Context>,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let Some(argument_expr) = call_expr.arguments(ast).get_node(ast, 0) else {
      unreachable!("ensure require includes arguments")
    };
    let argument_expr = argument_expression(ast, argument_expr);
    let call_span = call_expr.span(ast);
    let referenced_specifiers = parser
      .destructuring_assignment_properties
      .get(&call_span)
      .map(|keys| {
        let mut refs = Vec::new();
        keys.traverse_on_leaf(&mut |stack| {
          let names = stack.iter().map(|p| p.id.clone()).collect();
          refs.push(ReferencedSpecifier::new(names));
        });
        refs
      });
    let dep = create_commonjs_require_context_dependency(
      parser,
      param,
      call_expr,
      argument_expr,
      referenced_specifiers,
      request_context,
    );
    let dep_idx = parser.next_dependency_idx();
    if let Some(require_references) = parser
      .common_js_require_references
      .get_require_mut(&call_span)
    {
      require_references.dep_locator = Some(RequireDependencyLocator {
        dep_idx,
        block_idx: parser.collecting_dependencies_for_block,
        dep_type: DependencyType::CommonJSRequireContext,
      });
    }
    parser.add_dependency(BoxDependency::new(dep));
    Some(true)
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    expr: CallOrNewExpression,
    request_context: Option<Context>,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let callee = expr.callee(ast);
    let args = expr.args(ast);
    let expr_span = expr.span(ast);

    if args.len() != 1 {
      return None;
    }
    let argument_expr = args.at(0).as_expr(ast)?;

    // Skip adding require() as a dependency when in unreachable code after
    // return/throw (e.g. require("fail") in dead code should not be resolved).
    // We still walk the AST so scope and other code are correct (issue-19514,
    // dead-code-elimination). Mirrors import_parser_plugin's dynamic import check.
    if parser.terminated.is_some() && !parser.is_top_level_scope() {
      return Some(true);
    }

    if Self::has_ignore_comment(parser, expr_span, argument_expr.span(ast)) {
      return Some(true);
    }

    let param = parser.evaluate_expression(argument_expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self
          .process_require_item(parser, expr_span, p, request_context.clone())
          .is_none()
        {
          is_expression = true;
        }
      }
      if !is_expression {
        let range: DependencyRange = callee.span(parser.ast.ast).into();
        let loc = parser.to_dependency_location(range);
        parser.add_presentational_dependency(Arc::new(RequireHeaderDependency::new(range, loc)));
        return Some(true);
      }
    }
    if param.is_string()
      && let Some(local_module) = parser.get_local_module_mut(param.string())
    {
      local_module.flag_used();
      let span = expr_span;
      let dep = Arc::new(LocalModuleDependency::new(
        local_module.clone(),
        Some(span.into()),
        matches!(expr, CallOrNewExpression::New(_)),
      ));
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    if matches!(parser.javascript_options.require_dynamic, Some(false)) && !param.is_string() {
      return None;
    }

    if self
      .process_require_item(parser, expr_span, &param, request_context.clone())
      .is_none()
      && let CallOrNewExpression::Call(call_expr) = expr
    {
      self.process_require_context(parser, call_expr, &param, request_context);
    } else {
      let range: DependencyRange = callee.span(parser.ast.ast).into();
      let loc = parser.to_dependency_location(range);
      parser.add_presentational_dependency(Arc::new(RequireHeaderDependency::new(range, loc)));
    }
    Some(true)
  }

  fn require_as_expression_handler(
    &self,
    parser: &mut JavascriptParser,
    ident: &Identifier,
    request_context: Option<Context>,
  ) -> Option<bool> {
    if parser.javascript_options.require_as_expression == Some(false) {
      return None;
    }

    let span = ident.span();
    let start = span.real_lo();
    let end = span.real_hi();
    let dep = CommonJsRequireContextDependency::new(
      ContextOptions {
        mode: ContextMode::Sync,
        recursive: true,
        pattern: ContextModulePattern::None,
        request: ".".to_string(),
        context: get_context(parser.resource_data).to_string(),
        compiler_context: parser.compiler_options.context.clone(),
        start,
        end,
        ..Default::default()
      },
      parser
        .to_dependency_location(DependencyRange::from(span))
        .expect("Should get correct loc"),
      span.into(),
      None,
      parser.in_try,
      request_context,
    );
    let is_renaming_require = parser
      .is_renaming
      .as_ref()
      .is_some_and(|is_renaming| is_renaming == expr_name::REQUIRE)
      && !parser.javascript_options.require_alias.unwrap_or_default();
    if let Some(true) = parser.javascript_options.unknown_context_critical
      && !is_renaming_require
    {
      let mut error = create_traceable_error(
        "Critical dependency".into(),
        "require function is used in a way in which dependencies cannot be statically extracted"
          .to_string(),
        parser.source.to_string(),
        span.into(),
      );
      error.severity = Severity::Warning;
      dep.set_critical(Some(Diagnostic::from(error)));
    }
    parser.add_dependency(BoxDependency::new(dep));
    Some(true)
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CommonJsImportsParserPlugin {
  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: Expr,
  ) -> Option<bool> {
    if !should_parse_commonjs_require(parser) {
      return None;
    }

    let ast = parser.ast.ast;
    if let Some(call) = expr.as_call_expression(ast)
      && is_require_call_expr(parser, call)
    {
      return Some(true);
    }
    if let Some(ident) = expr.as_identifier_reference(ast)
      && let Some(name_info) = parser.get_name_info_from_variable(ast.get_utf8(ident.name(ast)))
      && let Some(info) = name_info.info
      && let Some(name) = info.name.clone()
      && parser
        .get_tag_data::<RequireTagData>(&name, COMMONJS_REQUIRE_TAG)
        .is_some()
    {
      return Some(true);
    }
    None
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: VariableDeclarator,
    declaration: VariableDeclaration,
  ) -> Option<bool> {
    if parser.javascript_options.is_create_require_enabled() {
      pre_tag_created_require_declarator(parser, declarator, declaration);
    }

    if !should_parse_commonjs_require(parser) {
      return None;
    }

    let ast = parser.ast.ast;
    if declaration.kind(ast) != VariableDeclarationKind::Var
      && let Some(init) = declarator.init(ast)
      && let Some(call) = init.as_call_expression(ast)
      && let Some(binding) = declarator.id(ast).as_binding_identifier(ast)
      && is_require_call_expr(parser, call)
    {
      let name = Atom::from(ast.get_utf8(binding.name(ast)));
      parser.define_variable(name.clone());
      tag_commonjs_require_referenced(parser, call, name);
    }
    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: VariableDeclarator,
    declaration: VariableDeclaration,
  ) -> Option<bool> {
    if !parser.javascript_options.is_create_require_enabled() {
      return None;
    }

    let ast = parser.ast.ast;
    let init = declarator.init(ast)?;
    if let Some(init) = init.as_identifier_reference(ast)
      && let Some(data) = parser
        .get_tag_data::<CreatedRequireTagData>(
          ast.get_utf8(init.name(ast)),
          CREATED_REQUIRE_IDENTIFIER_TAG,
        )
        .cloned()
      && let Some(binding) = declarator.id(ast).as_binding_identifier(ast)
    {
      let name = Atom::from(ast.get_utf8(binding.name(ast)));
      parser.define_variable(name.clone());
      parser.tag_variable(
        name,
        CREATED_REQUIRE_IDENTIFIER_TAG,
        Some(CreatedRequireTagData {
          side_effects: String::new(),
          ..data
        }),
      );
      return Some(true);
    }

    if let Some(init) = init.as_identifier_reference(ast)
      && is_create_require_specifier(parser, ast.get_utf8(init.name(ast)))
      && let Some(binding) = declarator.id(ast).as_binding_identifier(ast)
    {
      let name = Atom::from(ast.get_utf8(binding.name(ast)));
      parser.define_variable(name.clone());
      tag_create_require(parser, name);
    }

    let binding = declarator.id(ast).as_binding_identifier(ast)?;

    if is_create_require_namespace_member(parser, init) {
      let name = Atom::from(ast.get_utf8(binding.name(ast)));
      parser.define_variable(name.clone());
      tag_create_require(parser, name);
    }

    if let Some(call) = init.as_call_expression(ast)
      && let callee = call.callee(ast)
      && (is_evaluated_create_require(parser, callee)
        || is_create_require_namespace_member(parser, callee))
      && let Some(argument) = parse_create_require_argument(parser, call, false)
    {
      let args = arguments_view(parser.ast.ast, call.arguments(parser.ast.ast));
      let call_span = call.span(parser.ast.ast);
      let clear_call = should_clear_create_require_call(parser, &args);
      let deferred_callee = (declaration.kind(parser.ast.ast) == VariableDeclarationKind::Const
        && can_defer_create_require_call(parser, &args))
      .then(|| deferred_create_require_callee(parser, callee, call_span))
      .flatten();
      let walk_callee = !clear_call && deferred_callee.is_none();
      tag_created_require_declarator(
        parser,
        binding,
        call_span,
        clear_call,
        &args,
        deferred_callee,
        argument,
      );
      if walk_callee {
        walk_create_require_callee(parser, call);
      }
      return Some(true);
    }

    if let Some(init) = init.as_new_expression(parser.ast.ast)
      && (is_evaluated_create_require(parser, init.callee(parser.ast.ast))
        || is_create_require_namespace_member(parser, init.callee(parser.ast.ast)))
      && let Some(argument) = parse_create_require_new_argument(parser, init, false)
    {
      let ast = parser.ast.ast;
      let args = arguments_view(ast, init.arguments(ast));
      tag_created_require_declarator(
        parser,
        binding,
        init.span(ast),
        false,
        &args,
        None,
        argument,
      );
      parser.walk_expression(init.callee(parser.ast.ast));
      return Some(true);
    }

    if parser
      .get_tag_data::<CreatedRequireTagData>(
        parser.ast.ast.get_utf8(binding.name(parser.ast.ast)),
        CREATED_REQUIRE_IDENTIFIER_TAG,
      )
      .is_some()
    {
      parser.define_variable(Atom::from(
        parser.ast.ast.get_utf8(binding.name(parser.ast.ast)),
      ));
      parser.walk_expression(init);
      return Some(true);
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == COMMONJS_REQUIRE_TAG && should_parse_commonjs_require(parser) {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let data = RequireTagData::downcast(tag_info.data.clone()?);
      if let Some(keys) = parser
        .destructuring_assignment_properties
        .get(&ident.span())
      {
        let mut refs = Vec::new();
        keys.traverse_on_leaf(&mut |stack| {
          refs.push(stack.iter().map(|p| p.id.clone()).collect::<Vec<Atom>>());
        });
        for ids in refs {
          parser
            .common_js_require_references
            .get_require_mut_expect(&data.require_span)
            .add_reference(ids);
        }
      } else {
        parser
          .common_js_require_references
          .get_require_mut_expect(&data.require_span)
          .add_reference(vec![]);
      }
      return Some(true);
    }

    if for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser) {
      return self.require_as_expression_handler(parser, ident, None);
    }

    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      if preserve_unhandled_created_require(parser) {
        return Some(true);
      }
      let context = current_created_require_context(parser);
      return self.require_as_expression_handler(parser, ident, context);
    }

    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: HookMemberExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      if members
        .first()
        .is_some_and(|member| member.as_ref() == "cache")
      {
        add_require_cache_dependency(
          parser,
          require_cache_range(parser.ast.ast, _expr, member_ranges, members).into(),
        );
      } else if !preserve_unhandled_created_require(parser) {
        handle_created_require_member(
          parser,
          _expr.span(parser.ast.ast),
          require_cache_range(parser.ast.ast, _expr, member_ranges, members),
          members,
          "undefined".into(),
        );
      }
      return Some(true);
    }

    if for_name != COMMONJS_REQUIRE_TAG || !should_parse_commonjs_require(parser) {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = RequireTagData::downcast(tag_info.data.clone()?);
    let ids = get_non_optional_part(members, members_optionals);
    parser
      .common_js_require_references
      .get_require_mut_expect(&data.require_span)
      .add_reference(ids.to_vec());
    Some(true)
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: CallExpression,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      let ids = get_non_optional_part(members, members_optionals);
      if members.is_empty() {
        wrap_created_require_with_side_effects(parser, expr.span(parser.ast.ast));
        return self.require_handler(
          parser,
          CallOrNewExpression::Call(expr),
          current_created_require_context(parser),
        );
      }
      if members.len() == 1 && members[0].as_ref() == "resolve" {
        return self.process_created_require_resolve_call(parser, expr);
      }
      if members
        .first()
        .is_some_and(|member| member.as_ref() != "cache")
        && preserve_unhandled_created_require(parser)
      {
        let ast = parser.ast.ast;
        parser.walk_arguments(
          expr
            .arguments(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id)),
        );
        return Some(true);
      }
      if ids.len() != members.len() {
        let ast = parser.ast.ast;
        parser.walk_arguments(
          expr
            .arguments(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id)),
        );
        return Some(true);
      }
      return None;
    }

    if for_name != COMMONJS_REQUIRE_TAG || !should_parse_commonjs_require(parser) {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = RequireTagData::downcast(tag_info.data.clone()?);
    let ids = get_non_optional_part(members, members_optionals);
    let direct_import = members.is_empty();
    parser
      .common_js_require_references
      .get_require_mut_expect(&data.require_span)
      .add_call_reference(
        ids.to_vec(),
        parser
          .javascript_options
          .strict_this_context_on_imports
          .unwrap_or(false)
          && !direct_import,
      );
    let ast = parser.ast.ast;
    parser.walk_arguments(
      expr
        .arguments(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id)),
    );
    Some(true)
  }

  fn can_rename(&self, parser: &mut JavascriptParser<'p>, for_name: &str) -> Option<bool> {
    if (for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser))
      || for_name == CREATED_REQUIRE_IDENTIFIER_TAG
    {
      Some(true)
    } else {
      None
    }
  }

  fn rename(&self, parser: &mut JavascriptParser<'p>, expr: Expr, for_name: &str) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      if !preserve_unhandled_created_require(parser)
        && let Some(ident) = expr.as_identifier_reference(parser.ast.ast)
      {
        let context = current_created_require_context(parser);
        self.require_as_expression_handler(
          parser,
          &Identifier {
            span: ident.span(parser.ast.ast),
          },
          context,
        )?;
      }
      parser.walk_expression(expr);
      Some(false)
    } else if for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser) {
      if parser.javascript_options.require_alias.unwrap_or_default() {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          expr.span(parser.ast.ast).into(),
          "undefined".into(),
        )));
        Some(false)
      } else {
        let old_is_renaming = parser.is_renaming.clone();
        parser.is_renaming = Some(expr_name::REQUIRE.into());
        parser.walk_expression(expr);
        parser.is_renaming = old_is_renaming;
        Some(true)
      }
    } else {
      None
    }
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    let span = expr.span(parser.ast.ast);
    ((should_parse_commonjs_require(parser)
      && (for_name == expr_name::REQUIRE
        || for_name == expr_name::REQUIRE_RESOLVE
        || for_name == expr_name::REQUIRE_RESOLVE_WEAK))
      || should_handle_create_require_specifier(parser, for_name)
      || for_name == CREATED_REQUIRE_IDENTIFIER_TAG)
      .then(|| eval::evaluate_to_string("function".to_string(), span.real_lo(), span.real_hi()))
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    match for_name {
      expr_name::REQUIRE if should_parse_commonjs_require(parser) => {
        Some(eval::evaluate_to_identifier(
          expr_name::REQUIRE.into(),
          expr_name::REQUIRE.into(),
          Some(true),
          start,
          end,
        ))
      }
      expr_name::REQUIRE_RESOLVE if should_parse_commonjs_require(parser) => {
        Some(eval::evaluate_to_identifier(
          expr_name::REQUIRE_RESOLVE.into(),
          expr_name::REQUIRE_RESOLVE.into(),
          Some(true),
          start,
          end,
        ))
      }
      expr_name::REQUIRE_RESOLVE_WEAK if should_parse_commonjs_require(parser) => {
        Some(eval::evaluate_to_identifier(
          expr_name::REQUIRE_RESOLVE_WEAK.into(),
          expr_name::REQUIRE_RESOLVE_WEAK.into(),
          Some(true),
          start,
          end,
        ))
      }
      CREATE_REQUIRE_SPECIFIER_TAG if is_current_create_require_tag(parser) => {
        Some(eval::evaluate_to_identifier(
          CREATE_REQUIRE_EVALUATED_TAG.into(),
          CREATE_REQUIRE_EVALUATED_TAG.into(),
          Some(true),
          start,
          end,
        ))
      }
      CREATE_REQUIRE_EVALUATED_TAG => Some(eval::evaluate_to_identifier(
        CREATE_REQUIRE_EVALUATED_TAG.into(),
        CREATE_REQUIRE_EVALUATED_TAG.into(),
        Some(true),
        start,
        end,
      )),
      _ => None,
    }
  }

  fn evaluate_call_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    expr: CallExpression,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if !should_handle_create_require_call(parser, for_name, Some(expr.callee(parser.ast.ast))) {
      return None;
    }
    evaluate_create_require_call_expression(parser, expr)
  }

  fn evaluate_call_expression_member(
    &self,
    parser: &mut JavascriptParser<'p>,
    property: &str,
    expr: CallExpression,
    param: BasicEvaluatedExpression<'p>,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if !is_create_require_namespace_member_param(parser, property, &param) {
      return None;
    }
    evaluate_create_require_call_expression(parser, expr)
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    // same as webpack/tagRequireExpression
    if (should_parse_commonjs_require(parser)
      && (for_name == expr_name::REQUIRE
        || for_name == expr_name::REQUIRE_RESOLVE
        || for_name == expr_name::REQUIRE_RESOLVE_WEAK))
      || should_handle_create_require_specifier(parser, for_name)
      || for_name == CREATED_REQUIRE_IDENTIFIER_TAG
    {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(parser.ast.ast).into(),
        "'function'".into(),
      )));
      Some(true)
    } else {
      None
    }
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
    {
      self.require_handler(parser, CallOrNewExpression::Call(call_expr), None)
    } else if should_handle_create_require_call(
      parser,
      for_name,
      Some(call_expr.callee(parser.ast.ast)),
    ) {
      if let Some(argument) = parse_create_require_argument(parser, call_expr, true) {
        let ast = parser.ast.ast;
        let args = arguments_view(ast, call_expr.arguments(ast));
        let call_span = call_expr.span(ast);
        let clear_call = should_clear_create_require_call(parser, &args);
        if clear_call {
          clear_create_require_call(parser, call_span);
        } else if argument.replace_argument {
          let argument_expr = argument_expression(parser.ast.ast, args.at(0));
          parser.add_presentational_dependency(Arc::new(ConstDependency::new(
            argument_expr.span(parser.ast.ast).into(),
            json_stringify_str(&argument.value).into(),
          )));
        } else {
          walk_create_require_argument_side_effects(
            parser,
            argument_expression(parser.ast.ast, args.at(0)),
          );
        }
        if !clear_call {
          walk_create_require_callee(parser, call_expr);
        }
        walk_create_require_ignored_args(parser, call_expr);
        Some(true)
      } else {
        None
      }
    } else if for_name == expr_name::REQUIRE_RESOLVE && should_parse_commonjs_require(parser) {
      if matches!(parser.javascript_options.require_resolve, Some(false))
        || !Self::should_process_resolve(parser, call_expr)
      {
        return None;
      }

      self.process_resolve(parser, call_expr, false, None);
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE_WEAK && should_parse_commonjs_require(parser) {
      if !Self::should_process_resolve(parser, call_expr) {
        return None;
      }

      self.process_resolve(parser, call_expr, true, None);
      Some(true)
    } else if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      wrap_created_require_with_side_effects(parser, call_expr.span(parser.ast.ast));
      self.require_handler(
        parser,
        CallOrNewExpression::Call(call_expr),
        current_created_require_context(parser),
      )
    } else {
      None
    }
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    new_expr: NewExpression,
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
    {
      self.require_handler(parser, CallOrNewExpression::New(new_expr), None)
    } else if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      wrap_created_require_with_side_effects(parser, new_expr.span(parser.ast.ast));
      self.require_handler(
        parser,
        CallOrNewExpression::New(new_expr),
        current_created_require_context(parser),
      )
    } else {
      None
    }
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    member_expr: MemberExpression,
    callee_members: &[Atom],
    call_expr: CallExpression,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && let Some(argument) = parse_create_require_argument(parser, call_expr, false)
    {
      let ast = parser.ast.ast;
      let args = arguments_view(ast, call_expr.arguments(ast));
      let member_span = member_expr.span(ast);
      let side_effects = create_require_args_side_effects(parser, &args, &argument);
      let unsupported_replacement = create_require_unsupported_member_replacement(&side_effects);
      handle_created_require_member(
        parser,
        member_span,
        require_cache_range(parser.ast.ast, member_expr.into(), member_ranges, members),
        members,
        unsupported_replacement,
      );
      if members
        .first()
        .is_some_and(|member| member.as_ref() == "cache")
      {
        wrap_span_with_side_effects(parser, member_span, &side_effects);
      }
      walk_create_require_ignored_args(parser, call_expr);
      return Some(true);
    }

    if callee_members.is_empty()
      && (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
      && let Some(dep) = self.chain_handler(parser, member_expr, call_expr, members, false)
    {
      parser.add_dependency(BoxDependency::new(dep));
      return Some(true);
    }
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    callee_members: &[Atom],
    inner_call_expr: CallExpression,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && members.len() == 1
      && members[0].as_ref() == "resolve"
    {
      let ast = parser.ast.ast;
      let call_args = arguments_view(ast, call_expr.arguments(ast));
      if matches!(parser.javascript_options.require_resolve, Some(false))
        || call_args.len() != 1
        || call_args.at(0).as_expr(ast).is_none()
      {
        walk_unsupported_create_require_resolve(parser, inner_call_expr, call_expr);
        return Some(true);
      }
      let argument = parse_create_require_argument(parser, inner_call_expr, false)?;
      let ast = parser.ast.ast;
      let inner_args = arguments_view(ast, inner_call_expr.arguments(ast));
      let side_effects = create_require_args_side_effects(parser, &inner_args, &argument);
      wrap_span_with_side_effects(parser, call_expr.span(parser.ast.ast), &side_effects);
      let context = argument.context;
      walk_create_require_ignored_args(parser, inner_call_expr);
      self.process_resolve(parser, call_expr, false, Some(context));
      return Some(true);
    }

    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && let Some(argument) = parse_create_require_argument(parser, inner_call_expr, false)
    {
      let ast = parser.ast.ast;
      let inner_args = arguments_view(ast, inner_call_expr.arguments(ast));
      let side_effects = create_require_args_side_effects(parser, &inner_args, &argument);
      let unsupported_replacement = create_require_unsupported_member_replacement(&side_effects);
      let callee = call_expr.callee(ast);
      let member_span = callee.span(ast);
      handle_created_require_member(
        parser,
        member_span,
        require_cache_range(
          parser.ast.ast,
          callee.as_member_expression(parser.ast.ast)?.into(),
          member_ranges,
          members,
        ),
        members,
        unsupported_replacement,
      );
      if members
        .first()
        .is_some_and(|member| member.as_ref() == "cache")
      {
        wrap_span_with_side_effects(parser, member_span, &side_effects);
      }
      walk_create_require_ignored_args(parser, inner_call_expr);
      let ast = parser.ast.ast;
      parser.walk_arguments(
        call_expr
          .arguments(ast)
          .iter()
          .map(|id| ast.get_node_in_sub_range(id)),
      );
      return Some(true);
    }

    if callee_members.is_empty()
      && (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
      && let Some(member) = call_expr
        .callee(parser.ast.ast)
        .as_member_expression(parser.ast.ast)
      && let Some(dep) = self.chain_handler(parser, member, inner_call_expr, members, true)
    {
      parser.add_dependency(BoxDependency::new(dep));
      let ast = parser.ast.ast;
      parser.walk_arguments(
        call_expr
          .arguments(ast)
          .iter()
          .map(|id| ast.get_node_in_sub_range(id)),
      );
      return Some(true);
    }
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: AssignmentExpression,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser) {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        (0, 0).into(),
        "var require;".into(),
      )));
      return Some(true);
    }

    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG
      || for_name == CREATE_REQUIRE_SPECIFIER_TAG
      || for_name == CREATE_REQUIRE_EVALUATED_TAG
    {
      if matches!(
        expr.operator(parser.ast.ast),
        AssignmentOperator::LogicalOrAssign | AssignmentOperator::NullishAssign
      ) {
        return Some(true);
      }
      let Some(name) = source_for_span(parser, ident.span()) else {
        return Some(true);
      };
      clear_create_require_tag(parser, &name);
      return Some(true);
    }

    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    for (locator, variable_name, mut references) in parser
      .common_js_require_references
      .take_all_require_references()
    {
      // If the require result is assigned to a variable that is also an ESM
      // named export, importers may access arbitrary properties on it. In that
      // case the entire module must be considered referenced.
      if parser.build_info.esm_named_exports.contains(&variable_name) {
        references.push(ReferencedSpecifier::new(vec![]));
      }
      let dep = if let Some(block_idx) = locator.block_idx
        && let Some(block) = parser.get_block_mut(block_idx)
      {
        block.get_dependency_mut(locator.dep_idx)
      } else {
        parser.get_dependency_mut(locator.dep_idx)
      };
      let Some(dep) = dep else {
        continue;
      };
      match locator.dep_type {
        DependencyType::CjsRequire => {
          let dep = dep
            .downcast_mut::<CommonJsRequireDependency>()
            .expect("Failed to downcast to CommonJsRequireDependency");
          dep.set_referenced_specifiers(references);
        }
        DependencyType::CommonJSRequireContext => {
          let dep = dep
            .downcast_mut::<CommonJsRequireContextDependency>()
            .expect("Failed to downcast to CommonJsRequireContextDependency");
          dep.set_referenced_specifiers(references);
        }
        _ => unreachable!(),
      }
    }

    for name in parser.created_require_references.take_exported_locals() {
      let pending_call = parser
        .get_tag_data::<CreatedRequireTagData>(&name, CREATED_REQUIRE_IDENTIFIER_TAG)
        .and_then(|data| data.pending_call);
      if let Some(call_span) = pending_call {
        parser.created_require_references.mark_must_keep(call_span);
      }
    }

    let mut created_require_references = std::mem::take(&mut parser.created_require_references);
    let pending_calls = created_require_references.take_pending();
    parser.created_require_references = created_require_references;
    for (call_span, pending) in pending_calls {
      if pending.must_keep {
        keep_deferred_create_require_call(parser, pending);
      } else {
        clear_create_require_call(parser, call_span);
      }
    }
    None
  }
}
