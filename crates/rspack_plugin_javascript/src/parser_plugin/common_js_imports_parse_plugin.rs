#[cfg(windows)]
use std::path::Path;

use rspack_core::{
  ConstDependency, Context, ContextDependency, ContextMode, ContextModulePattern, ContextOptions,
  DependencyCategory, DependencyRange, DependencyType, ModuleType, ReferencedSpecifier,
  RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_error::{Diagnostic, Severity};
use rspack_util::{SpanExt, json_stringify_str};
use swc_atoms::Atom;
use swc_experimental_ecma_ast::{
  AssignExpr, AssignOp, CallExpr, Callee, Expr, ExprOrSpread, GetSpan, Ident, Lit, MemberExpr,
  MemberProp, NewExpr, Span, UnaryExpr, UnaryOp, VarDeclarator,
};
use url::Url;

use super::{
  JavascriptParserPlugin,
  esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  get_url_request,
  url_plugin::is_meta_url,
};
use crate::{
  dependency::{
    CommonJsFullRequireDependency, CommonJsRequireContextDependency, CommonJsRequireDependency,
    RequireHeaderDependency, RequireResolveContextDependency, RequireResolveDependency,
    RequireResolveHeaderDependency, local_module_dependency::LocalModuleDependency,
  },
  magic_comment::try_extract_magic_comment,
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{
    CallHooksName, ExportedVariableInfo, JavascriptParser, TagInfoData, VariableDeclaration,
    VariableDeclarationKind, VariableInfo, VariableInfoFlags, context_reg_exp,
    create_context_dependency, create_traceable_error, expr_name, get_non_optional_part,
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
}

struct CreateRequireArgument {
  value: String,
  context: Context,
  replace_argument: bool,
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
pub fn is_create_require_specifier(parser: &mut JavascriptParser, name: &Atom) -> bool {
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
  callee: Option<&Expr>,
) -> bool {
  should_handle_create_require_specifier(parser, for_name)
    || callee.is_some_and(|callee| is_create_require_namespace_member(parser, callee))
}

#[cold]
#[inline(never)]
fn is_evaluated_create_require(parser: &mut JavascriptParser, expr: &Expr) -> bool {
  let evaluated = parser.evaluate_expression(expr);
  evaluated.is_identifier() && evaluated.identifier() == CREATE_REQUIRE_EVALUATED_TAG
}

#[cold]
#[inline(never)]
pub(crate) fn is_create_require_namespace_member(
  parser: &mut JavascriptParser,
  expr: &Expr,
) -> bool {
  let Some(member_expr) = expr.as_member() else {
    return false;
  };
  let Some(namespace) = member_expr.obj.as_ident() else {
    return false;
  };
  let Some(settings) =
    parser.get_tag_data::<ESMSpecifierData>(&Atom::from(namespace.sym.as_str()), ESM_SPECIFIER_TAG)
  else {
    return false;
  };
  let namespace_import = settings.namespace_import;
  let source = settings.source.clone();
  let Some(member) = static_member_name(member_expr) else {
    return false;
  };
  namespace_import
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
  settings.namespace_import
    && create_require_import_specifier(parser, &settings.source)
      .is_some_and(|specifier| property == specifier.as_ref())
}

#[cold]
#[inline(never)]
fn static_member_name(member_expr: &MemberExpr) -> Option<Atom> {
  match &member_expr.prop {
    MemberProp::Ident(ident) => Some(Atom::from(ident.sym.as_str())),
    MemberProp::Computed(computed) => match &computed.expr {
      Expr::Lit(lit) => match &**lit {
        Lit::Str(str) => Some(Atom::from(str.value.as_wtf8().to_string_lossy().as_ref())),
        _ => None,
      },
      _ => None,
    },
    MemberProp::PrivateName(_) => None,
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
fn evaluate_create_require_argument(parser: &mut JavascriptParser, arg: &Expr) -> Option<String> {
  let evaluated = parser.evaluate_expression(arg);
  if let Some(value) = evaluated.as_string() {
    return Some(value);
  }

  let new_expr = arg.as_new()?;
  if new_expr.callee.as_ident()?.sym.as_str() != "URL"
    || parser.get_variable_info(&Atom::from("URL")).is_some()
  {
    return None;
  }
  if let Some(args) = &new_expr.args
    && !args.is_empty()
    && args[0].spread.is_none()
    && let Some(value) = parser.evaluate_expression(&args[0].expr).as_string()
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
  args: &[ExprOrSpread],
  start: usize,
) -> bool {
  args
    .iter()
    .skip(start)
    .all(|arg| arg.spread.is_none() && is_side_effect_free_ignored_url_arg(parser, &arg.expr))
}

#[cold]
#[inline(never)]
fn is_side_effect_free_ignored_url_arg(parser: &mut JavascriptParser, expr: &Expr) -> bool {
  match expr {
    Expr::Lit(_) => true,
    Expr::Ident(ident) => {
      ident.sym.as_str() == "undefined"
        && parser.get_variable_info(&Atom::from("undefined")).is_none()
    }
    Expr::Unary(unary) if unary.op == UnaryOp::Void => {
      is_side_effect_free_ignored_url_arg(parser, &unary.arg)
    }
    _ => false,
  }
}

#[cold]
#[inline(never)]
fn parse_create_require_argument(
  parser: &mut JavascriptParser,
  call_expr: &CallExpr,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  parse_create_require_argument_from_args(parser, &call_expr.args, call_expr.span, emit_warning)
}

#[cold]
#[inline(never)]
fn parse_create_require_argument_from_args(
  parser: &mut JavascriptParser,
  args: &[ExprOrSpread],
  span: Span,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  if args.is_empty() {
    if emit_warning {
      add_create_require_warning(parser, "module.createRequire requires one argument.", span);
    }
    return None;
  }

  if let Some(spread) = args[0].spread {
    if emit_warning {
      add_create_require_warning(
        parser,
        "module.createRequire does not support spread arguments.",
        spread,
      );
    }
    return None;
  }

  let arg = &args[0].expr;
  let Some(value) = evaluate_create_require_argument(parser, arg) else {
    if emit_warning {
      add_create_require_warning(
        parser,
        "module.createRequire failed parsing argument.",
        arg.span(),
      );
    }
    return None;
  };
  let context = create_require_context_from_path(&value);
  if context.is_none() && emit_warning {
    add_create_require_warning(
      parser,
      "module.createRequire supports only file URLs and absolute paths.",
      arg.span(),
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
  new_expr: &NewExpr,
  emit_warning: bool,
) -> Option<CreateRequireArgument> {
  let args = new_expr.args.as_deref().unwrap_or_default();
  parse_create_require_argument_from_args(parser, args, new_expr.span, emit_warning)
}

#[inline(never)]
fn should_replace_create_require_argument(parser: &mut JavascriptParser, arg: &Expr) -> bool {
  let Some(new_expr) = arg.as_new() else {
    return true;
  };
  if new_expr
    .callee
    .as_ident()
    .is_some_and(|ident| ident.sym.as_str() == "URL")
    && parser.get_variable_info(&Atom::from("URL")).is_none()
  {
    let is_absolute_file_url = is_absolute_file_url_constructor_arg(parser, arg);
    let start = if is_absolute_file_url { 1 } else { 2 };
    let Some(args) = &new_expr.args else {
      return true;
    };
    if is_absolute_file_url
      && let Some(base) = args.get(1)
      && !is_valid_ignored_url_base_arg(parser, base)
    {
      return false;
    }
    ignored_url_args_are_side_effect_free_from(parser, args, start)
  } else {
    true
  }
}

#[inline(never)]
fn is_valid_ignored_url_base_arg(parser: &mut JavascriptParser, base: &ExprOrSpread) -> bool {
  if base.spread.is_some() {
    return false;
  }
  if let Expr::Ident(ident) = &base.expr
    && ident.sym.as_str() == "undefined"
    && parser.get_variable_info(&Atom::from("undefined")).is_none()
  {
    return true;
  }
  if let Expr::Unary(unary) = &base.expr
    && unary.op == UnaryOp::Void
    && is_side_effect_free_ignored_url_arg(parser, &unary.arg)
  {
    return true;
  }
  parser
    .evaluate_expression(&base.expr)
    .as_string()
    .is_some_and(|base| Url::parse(&base).is_ok())
}

#[inline(never)]
fn is_absolute_file_url_constructor_arg(parser: &mut JavascriptParser, arg: &Expr) -> bool {
  let Some(new_expr) = arg.as_new() else {
    return false;
  };
  if new_expr
    .callee
    .as_ident()
    .is_none_or(|ident| ident.sym.as_str() != "URL")
    || parser.get_variable_info(&Atom::from("URL")).is_some()
  {
    return false;
  };
  let Some(args) = &new_expr.args else {
    return false;
  };
  args
    .first()
    .filter(|arg| arg.spread.is_none())
    .and_then(|arg| parser.evaluate_expression(&arg.expr).as_string())
    .is_some_and(|value| value.starts_with("file:/"))
}

#[inline(never)]
fn walk_create_require_callee(parser: &mut JavascriptParser, call_expr: &CallExpr) {
  if let Callee::Expr(callee) = &call_expr.callee {
    parser.walk_expression(callee);
  }
}

fn walk_create_require_ignored_args(parser: &mut JavascriptParser, call_expr: &CallExpr) {
  if call_expr.args.len() > 1 {
    parser.walk_expr_or_spread(&call_expr.args[1..]);
  }
}

#[inline(never)]
fn is_unbound_url_constructor(parser: &mut JavascriptParser, callee: &Expr) -> bool {
  callee
    .as_ident()
    .is_some_and(|ident| ident.sym.as_str() == "URL")
    && parser.get_variable_info(&Atom::from("URL")).is_none()
}

#[inline(never)]
fn walk_create_require_argument_side_effects(parser: &mut JavascriptParser, arg: &Expr) {
  let Some(new_expr) = arg.as_new() else {
    return;
  };
  if !is_unbound_url_constructor(parser, &new_expr.callee) {
    return;
  };
  let Some(args) = &new_expr.args else {
    return;
  };
  if args.len() > 1 {
    parser.walk_expr_or_spread(&args[1..]);
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
fn create_require_url_arg_side_effects(parser: &mut JavascriptParser, arg: &Expr) -> String {
  let Some(new_expr) = arg.as_new() else {
    return String::new();
  };
  if !is_unbound_url_constructor(parser, &new_expr.callee) {
    return String::new();
  };
  let Some(args) = &new_expr.args else {
    return String::new();
  };
  let start = if is_absolute_file_url_constructor_arg(parser, arg) {
    1
  } else {
    2
  };
  let mut side_effects = String::new();
  for arg in args.iter().skip(start) {
    let Some(source) = source_for_span(parser, arg.expr.span()) else {
      continue;
    };
    if arg.spread.is_some() {
      push_spread_side_effect(&mut side_effects, &source);
    } else if !is_side_effect_free_ignored_url_arg(parser, &arg.expr)
      && !arg
        .expr
        .as_member()
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
  parser.add_presentational_dependency(Box::new(ConstDependency::new(
    (span.real_lo(), span.real_lo()).into(),
    side_effects_with_suffix(side_effects, ""),
  )));
  parser.add_presentational_dependency(Box::new(ConstDependency::new(
    (span.real_hi(), span.real_hi()).into(),
    ")".into(),
  )));
}

#[inline(never)]
fn create_require_extra_arg_side_effects(
  parser: &JavascriptParser,
  args: &[ExprOrSpread],
) -> String {
  let mut side_effects = String::new();
  for arg in args.iter().skip(1) {
    let Some(source) = source_for_span(parser, arg.expr.span()) else {
      continue;
    };
    if arg.spread.is_some() {
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
  args: &[ExprOrSpread],
  argument: &CreateRequireArgument,
) -> String {
  let mut side_effects = if argument.replace_argument {
    String::new()
  } else {
    create_require_url_arg_side_effects(parser, &args[0].expr)
  };
  let extra_side_effects = create_require_extra_arg_side_effects(parser, args);
  if !extra_side_effects.is_empty() {
    push_side_effect(&mut side_effects, &extra_side_effects);
  }
  side_effects
}

#[inline(never)]
fn evaluate_created_require<'a>(
  parser: &mut JavascriptParser,
  range: Span,
  args: &[ExprOrSpread],
  argument: CreateRequireArgument,
) -> BasicEvaluatedExpression<'a> {
  let side_effects = create_require_args_side_effects(parser, args, &argument);
  let has_side_effects = !side_effects.is_empty();
  let evaluated_name = Atom::from(range.real_lo().to_string());
  parser.tag_variable(
    evaluated_name.clone(),
    CREATED_REQUIRE_IDENTIFIER_TAG,
    Some(CreatedRequireTagData {
      context: argument.context,
      side_effects,
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
pub(crate) fn evaluate_create_require_new_expression<'a>(
  parser: &mut JavascriptParser,
  for_name: &str,
  callee: Option<&Expr>,
  expr: &'a NewExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  if !should_handle_create_require_call(parser, for_name, callee) {
    return None;
  }
  let argument = parse_create_require_new_argument(parser, expr, false)?;
  Some(evaluate_created_require(
    parser,
    expr.span,
    expr.args.as_deref().unwrap_or_default(),
    argument,
  ))
}

#[inline(never)]
fn evaluate_create_require_call_expression<'a>(
  parser: &mut JavascriptParser,
  expr: &'a CallExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let argument = parse_create_require_argument(parser, expr, false)?;
  Some(evaluate_created_require(
    parser, expr.span, &expr.args, argument,
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

#[cold]
#[inline(never)]
fn tag_created_require_declarator(
  parser: &mut JavascriptParser,
  binding: &Ident,
  args: &[ExprOrSpread],
  argument: CreateRequireArgument,
) {
  let CreateRequireArgument {
    value,
    context,
    replace_argument,
  } = argument;
  let binding_name = Atom::from(binding.sym.as_str());
  parser.define_variable(binding_name.clone());
  parser.tag_variable(
    binding_name,
    CREATED_REQUIRE_IDENTIFIER_TAG,
    Some(CreatedRequireTagData {
      context,
      side_effects: String::new(),
    }),
  );
  if replace_argument {
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      args[0].expr.span().into(),
      json_stringify_str(&value).into(),
    )));
  } else {
    walk_create_require_argument_side_effects(parser, &args[0].expr);
  }
  parser.walk_expr_or_spread(&args[1..]);
}

fn clear_create_require_tag(parser: &mut JavascriptParser, name: &Atom) {
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
      .set(declared_scope, name.clone(), info);
  }
}

#[inline(never)]
fn add_require_cache_dependency(parser: &mut JavascriptParser, range: DependencyRange) {
  parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
    range,
    RuntimeGlobals::MODULE_CACHE,
  )));
}

#[inline(never)]
fn require_cache_range(member_expr: &MemberExpr, member_ranges: &[Span], members: &[Atom]) -> Span {
  if members.len() > 1 {
    member_ranges[1]
  } else {
    member_expr.span()
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
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
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

#[cold]
#[inline(never)]
fn walk_unsupported_create_require_resolve(
  parser: &mut JavascriptParser,
  inner_call_expr: &CallExpr,
  call_expr: &CallExpr,
) {
  walk_create_require_callee(parser, inner_call_expr);
  if inner_call_expr.args.len() == 1 && inner_call_expr.args[0].spread.is_none() {
    let arg = &inner_call_expr.args[0].expr;
    if let Some(value) = evaluate_create_require_argument(parser, arg) {
      if should_replace_create_require_argument(parser, arg) {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          arg.span().into(),
          json_stringify_str(&value).into(),
        )));
      } else {
        walk_create_require_argument_side_effects(parser, arg);
      }
    } else if let Some(new_expr) = arg.as_new()
      && is_unbound_url_constructor(parser, &new_expr.callee)
      && let Some(args) = new_expr.args.as_ref()
      && args.len() > 2
    {
      if get_url_request(parser, new_expr).is_some() {
        parser.walk_expr_or_spread(&args[2..]);
      } else {
        parser.walk_expr_or_spread(args);
      }
    } else {
      parser.walk_expression(arg);
    }
  } else {
    parser.walk_expr_or_spread(&inner_call_expr.args);
  }
  parser.walk_expr_or_spread(&call_expr.args);
}

fn tag_commonjs_require_referenced(
  parser: &mut JavascriptParser,
  require_call: &CallExpr,
  variable_name: Atom,
) {
  let require_span = require_call.span;
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
  call_expr: &CallExpr,
  arg_expr: &Expr,
  referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  request_context: Option<rspack_core::Context>,
) -> CommonJsRequireContextDependency {
  let result = create_context_dependency(param, parser);

  let span = call_expr.span;
  let options = ContextOptions {
    mode: ContextMode::Sync,
    recursive: true,
    pattern: context_reg_exp(&result.reg, "", None, parser).into(),
    category: DependencyCategory::CommonJS,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    replaces: result.replaces,
    start: span.real_lo(),
    end: span.real_hi(),
    referenced_specifiers,
    ..Default::default()
  };
  let range = call_expr.span.into();
  let loc = parser
    .to_dependency_location(range)
    .expect("Should get correct loc");
  let mut dep = CommonJsRequireContextDependency::new(
    options,
    loc,
    range,
    Some(arg_expr.span().into()),
    parser.in_try,
    request_context,
  );
  *dep.critical_mut() = result.critical;
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

  let options = ContextOptions {
    mode: if weak {
      ContextMode::Weak
    } else {
      ContextMode::Sync
    },
    recursive: true,
    pattern: context_reg_exp(&result.reg, "", None, parser).into(),
    category: DependencyCategory::CommonJS,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    replaces: result.replaces,
    start,
    end,
    ..Default::default()
  };
  RequireResolveContextDependency::new(options, range, parser.in_try, request_context)
}

pub(crate) fn is_require_call_expr(parser: &mut JavascriptParser, call: &CallExpr) -> bool {
  if !should_parse_commonjs_require(parser) {
    return false;
  }

  if call.args.len() != 1 {
    return false;
  }
  let Some(callee) = call.callee.as_expr() else {
    return false;
  };

  if let Some(ident) = callee.as_ident() {
    return ident
      .sym
      .call_hooks_name(parser, |_, for_name| {
        (for_name == expr_name::REQUIRE).then_some(true)
      })
      .unwrap_or_default();
  }

  if let Some(member) = callee.as_member() {
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

enum CallOrNewExpr<'a> {
  Call(&'a CallExpr<'a>),
  New(&'a NewExpr<'a>),
}

impl<'a> CallOrNewExpr<'a> {
  pub fn callee(&self) -> Option<&'a Expr<'a>> {
    match self {
      CallOrNewExpr::Call(call_expr) => call_expr.callee.as_expr(),
      CallOrNewExpr::New(new_expr) => Some(&new_expr.callee),
    }
  }

  pub fn args(&self) -> Option<&'a [ExprOrSpread<'a>]> {
    match self {
      CallOrNewExpr::Call(call_expr) => Some(&call_expr.args),
      CallOrNewExpr::New(new_expr) => new_expr.args.as_deref(),
    }
  }

  pub fn span(&self) -> Span {
    match self {
      CallOrNewExpr::Call(call_expr) => call_expr.span,
      CallOrNewExpr::New(new_expr) => new_expr.span,
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

  fn should_process_resolve(parser: &mut JavascriptParser, call_expr: &CallExpr) -> bool {
    let Callee::Expr(expr) = &call_expr.callee else {
      return false;
    };

    let Expr::Member(member_expr) = expr.as_ref() else {
      return false;
    };

    let Expr::Ident(ident) = &member_expr.obj else {
      return false;
    };

    if parser
      .get_variable_info(&Atom::from(ident.sym.as_str()))
      .is_some()
    {
      return false;
    }

    true
  }

  fn process_resolve(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    weak: bool,
    request_context: Option<Context>,
  ) {
    if call_expr.args.len() != 1 {
      return;
    }

    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = &call_expr.args[0]
      && Self::has_ignore_comment(parser, call_expr.span, argument_expr.span())
    {
      return;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    let range = call_expr.callee.span().into();
    let loc = parser.to_dependency_location(range);
    let require_resolve_header_dependency =
      Box::new(RequireResolveHeaderDependency::new(range, loc));

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
    expr: &CallExpr,
  ) -> Option<bool> {
    if expr.args.len() != 1 || expr.args[0].spread.is_some() {
      parser.walk_expr_or_spread(&expr.args);
      return Some(true);
    }
    if matches!(parser.javascript_options.require_resolve, Some(false)) {
      parser.walk_expr_or_spread(&expr.args);
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
        parser.add_dependency(Box::new(RequireResolveDependency::new_contextual(
          param.string().clone(),
          param.range().into(),
          weak,
          parser.in_try,
          context,
        )));
      } else {
        parser.add_dependency(Box::new(RequireResolveDependency::new(
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

    parser.add_dependency(Box::new(dep));
  }

  fn chain_handler(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    call_expr: &CallExpr,
    members: &[Atom],
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    if call_expr.args.len() != 1 {
      return None;
    }
    let arg = &call_expr.args[0];
    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = arg
      && Self::has_ignore_comment(parser, call_expr.span, argument_expr.span())
    {
      return None;
    }
    let param = parser.evaluate_expression(&arg.expr);
    let range = DependencyRange::from(member_expr.span);
    let loc = parser.to_dependency_location(range);
    param.is_string().then(|| {
      CommonJsFullRequireDependency::new(
        param.string().to_owned(),
        members.to_vec(),
        member_expr.span.into(),
        loc,
        is_call,
        parser
          .javascript_options
          .strict_this_context_on_imports
          .unwrap_or(false)
          && !members.is_empty(),
        parser.in_try,
        !parser.is_asi_position(member_expr.span.start),
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
      let dep: Box<dyn rspack_core::Dependency> = if let Some(context) = request_context {
        Box::new(CommonJsRequireDependency::new_contextual(
          param.string().clone(),
          range_expr,
          Some(span.into()),
          parser.in_try,
          context,
          loc,
          referenced_specifiers,
        ))
      } else {
        Box::new(CommonJsRequireDependency::new(
          param.string().clone(),
          range_expr,
          Some(span.into()),
          parser.in_try,
          loc,
          referenced_specifiers,
        ))
      };
      let dep_idx = parser.next_dependency_idx();
      if let Some(require_references) = parser.common_js_require_references.get_require_mut(&span) {
        require_references.dep_locator = Some(RequireDependencyLocator {
          dep_idx,
          block_idx: parser.collecting_dependencies_for_block,
          dep_type: DependencyType::CjsRequire,
        });
      }
      parser.add_dependency(dep);
      true
    })
  }

  fn process_require_context(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
    request_context: Option<Context>,
  ) -> Option<bool> {
    let Some(argument_expr) = call_expr.args.first().map(|expr| &expr.expr) else {
      unreachable!("ensure require includes arguments")
    };
    let referenced_specifiers = parser
      .destructuring_assignment_properties
      .get(&call_expr.span)
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
      .get_require_mut(&call_expr.span)
    {
      require_references.dep_locator = Some(RequireDependencyLocator {
        dep_idx,
        block_idx: parser.collecting_dependencies_for_block,
        dep_type: DependencyType::CommonJSRequireContext,
      });
    }
    parser.add_dependency(Box::new(dep));
    Some(true)
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    expr: CallOrNewExpr,
    request_context: Option<Context>,
  ) -> Option<bool> {
    let callee = expr.callee()?;
    let args = expr.args()?;

    if args.len() != 1 {
      return None;
    }
    if args[0].spread.is_some() {
      return None;
    }

    // Skip adding require() as a dependency when in unreachable code after
    // return/throw (e.g. require("fail") in dead code should not be resolved).
    // We still walk the AST so scope and other code are correct (issue-19514,
    // dead-code-elimination). Mirrors import_parser_plugin's dynamic import check.
    if parser.terminated.is_some() && !parser.is_top_level_scope() {
      return Some(true);
    }

    if let ExprOrSpread {
      spread: None,
      expr: argument_expr,
    } = &args[0]
      && Self::has_ignore_comment(parser, expr.span(), argument_expr.span())
    {
      return Some(true);
    }

    let param = parser.evaluate_expression(&args[0].expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self
          .process_require_item(parser, expr.span(), p, request_context.clone())
          .is_none()
        {
          is_expression = true;
        }
      }
      if !is_expression {
        let range: DependencyRange = callee.span().into();
        let loc = parser.to_dependency_location(range);
        parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(range, loc)));
        return Some(true);
      }
    }
    if param.is_string()
      && let Some(local_module) = parser.get_local_module_mut(param.string())
    {
      local_module.flag_used();
      let span = expr.span();
      let dep = Box::new(LocalModuleDependency::new(
        local_module.clone(),
        Some(span.into()),
        matches!(expr, CallOrNewExpr::New(_)),
      ));
      parser.add_presentational_dependency(dep);
      return Some(true);
    }

    if matches!(parser.javascript_options.require_dynamic, Some(false)) && !param.is_string() {
      return None;
    }

    if self
      .process_require_item(parser, expr.span(), &param, request_context.clone())
      .is_none()
      && let CallOrNewExpr::Call(call_expr) = expr
    {
      self.process_require_context(parser, call_expr, &param, request_context);
    } else {
      let range: DependencyRange = callee.span().into();
      let loc = parser.to_dependency_location(range);
      parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(range, loc)));
    }
    Some(true)
  }

  fn require_as_expression_handler(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    request_context: Option<Context>,
  ) -> Option<bool> {
    if parser.javascript_options.require_as_expression == Some(false) {
      return None;
    }

    let span = ident.span;
    let start = span.real_lo();
    let end = span.real_hi();
    let mut dep = CommonJsRequireContextDependency::new(
      ContextOptions {
        mode: ContextMode::Sync,
        recursive: true,
        pattern: ContextModulePattern::None,
        request: ".".to_string(),
        context: ".".to_string(),
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
      *dep.critical_mut() = Some(Diagnostic::from(error));
    }
    parser.add_dependency(Box::new(dep));
    Some(true)
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for CommonJsImportsParserPlugin {
  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &Expr,
  ) -> Option<bool> {
    if !should_parse_commonjs_require(parser) {
      return None;
    }

    if let Some(call) = expr.as_call()
      && is_require_call_expr(parser, call)
    {
      return Some(true);
    }
    if let Some(ident) = expr.as_ident()
      && let Some(name_info) = parser.get_name_info_from_variable(&Atom::from(ident.sym.as_str()))
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
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !should_parse_commonjs_require(parser) {
      return None;
    }

    if declaration.kind() != VariableDeclarationKind::Var
      && let Some(init) = &declarator.init
      && let Some(call) = init.as_call()
      && let Some(binding) = declarator.name.as_ident()
      && is_require_call_expr(parser, call)
    {
      let name = Atom::from(binding.id.sym.as_str());
      parser.define_variable(name.clone());
      tag_commonjs_require_referenced(parser, call, name);
    }
    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    declarator: &VarDeclarator,
    _stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !parser.javascript_options.is_create_require_enabled() {
      return None;
    }

    let init = declarator.init.as_ref()?;
    if let Some(init) = init.as_ident()
      && let Some(context) = parser
        .get_tag_data::<CreatedRequireTagData>(
          &Atom::from(init.sym.as_str()),
          CREATED_REQUIRE_IDENTIFIER_TAG,
        )
        .map(|data| data.context.clone())
      && let Some(binding) = declarator.name.as_ident()
    {
      let name = Atom::from(binding.id.sym.as_str());
      parser.define_variable(name.clone());
      parser.tag_variable(
        name,
        CREATED_REQUIRE_IDENTIFIER_TAG,
        Some(CreatedRequireTagData {
          context,
          side_effects: String::new(),
        }),
      );
      return Some(true);
    }

    if let Some(init) = init.as_ident()
      && is_create_require_specifier(parser, &Atom::from(init.sym.as_str()))
      && let Some(binding) = declarator.name.as_ident()
    {
      let name = Atom::from(binding.id.sym.as_str());
      parser.define_variable(name.clone());
      tag_create_require(parser, name);
    }

    let binding = declarator.name.as_ident()?;

    if is_create_require_namespace_member(parser, init) {
      let name = Atom::from(binding.id.sym.as_str());
      parser.define_variable(name.clone());
      tag_create_require(parser, name);
    }

    if let Some(call) = init.as_call()
      && let Some(callee) = call.callee.as_expr()
      && (is_evaluated_create_require(parser, callee)
        || is_create_require_namespace_member(parser, callee))
      && let Some(argument) = parse_create_require_argument(parser, call, false)
    {
      tag_created_require_declarator(parser, &binding.id, &call.args, argument);
      walk_create_require_callee(parser, call);
      return Some(true);
    }

    if let Some(init) = init.as_new()
      && (is_evaluated_create_require(parser, &init.callee)
        || is_create_require_namespace_member(parser, &init.callee))
      && let Some(argument) = parse_create_require_new_argument(parser, init, false)
      && let Some(args) = init.args.as_deref()
    {
      tag_created_require_declarator(parser, &binding.id, args, argument);
      parser.walk_expression(&init.callee);
      return Some(true);
    }

    if parser
      .get_tag_data::<CreatedRequireTagData>(
        &Atom::from(binding.id.sym.as_str()),
        CREATED_REQUIRE_IDENTIFIER_TAG,
      )
      .is_some()
    {
      parser.define_variable(Atom::from(binding.id.sym.as_str()));
      parser.walk_expression(init);
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
    if for_name == COMMONJS_REQUIRE_TAG && should_parse_commonjs_require(parser) {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let data = RequireTagData::downcast(tag_info.data.clone()?);
      if let Some(keys) = parser.destructuring_assignment_properties.get(&ident.span) {
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
      let context = current_created_require_context(parser);
      return self.require_as_expression_handler(parser, ident, context);
    }

    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      handle_created_require_member(
        parser,
        _expr.span(),
        require_cache_range(_expr, member_ranges, members),
        members,
        "undefined".into(),
      );
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
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      let ids = get_non_optional_part(members, members_optionals);
      if members.is_empty() {
        wrap_created_require_with_side_effects(parser, expr.span());
        return self.require_handler(
          parser,
          CallOrNewExpr::Call(expr),
          current_created_require_context(parser),
        );
      }
      if members.len() == 1 && members[0].as_ref() == "resolve" {
        return self.process_created_require_resolve_call(parser, expr);
      }
      if ids.len() != members.len() {
        parser.walk_expr_or_spread(&expr.args);
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
    parser.walk_expr_or_spread(&expr.args);
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

  fn rename(&self, parser: &mut JavascriptParser<'p>, expr: &Expr, for_name: &str) -> Option<bool> {
    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      if let Some(ident) = expr.as_ident() {
        let context = current_created_require_context(parser);
        self.require_as_expression_handler(parser, ident, context)?;
      }
      parser.walk_expression(expr);
      Some(false)
    } else if for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser) {
      if parser.javascript_options.require_alias.unwrap_or_default() {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          expr.span().into(),
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
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    ((should_parse_commonjs_require(parser)
      && (for_name == expr_name::REQUIRE
        || for_name == expr_name::REQUIRE_RESOLVE
        || for_name == expr_name::REQUIRE_RESOLVE_WEAK))
      || should_handle_create_require_specifier(parser, for_name)
      || for_name == CREATED_REQUIRE_IDENTIFIER_TAG)
      .then(|| {
        eval::evaluate_to_string(
          "function".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        )
      })
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
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
    expr: &'a CallExpr<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>>
  where
    'p: 'a,
  {
    if !should_handle_create_require_call(parser, for_name, expr.callee.as_expr()) {
      return None;
    }
    evaluate_create_require_call_expression(parser, expr)
  }

  fn evaluate_call_expression_member(
    &self,
    parser: &mut JavascriptParser<'p>,
    property: &str,
    expr: &'a CallExpr<'a>,
    param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>>
  where
    'p: 'a,
  {
    if !is_create_require_namespace_member_param(parser, property, &param) {
      return None;
    }
    evaluate_create_require_call_expression(parser, expr)
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &UnaryExpr,
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
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
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
    {
      self.require_handler(parser, CallOrNewExpr::Call(call_expr), None)
    } else if should_handle_create_require_call(parser, for_name, call_expr.callee.as_expr()) {
      if let Some(argument) = parse_create_require_argument(parser, call_expr, true) {
        if argument.replace_argument {
          parser.add_presentational_dependency(Box::new(ConstDependency::new(
            call_expr.args[0].expr.span().into(),
            json_stringify_str(&argument.value).into(),
          )));
        } else {
          walk_create_require_argument_side_effects(parser, &call_expr.args[0].expr);
        }
        walk_create_require_callee(parser, call_expr);
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
      wrap_created_require_with_side_effects(parser, call_expr.span());
      self.require_handler(
        parser,
        CallOrNewExpr::Call(call_expr),
        current_created_require_context(parser),
      )
    } else {
      None
    }
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    new_expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
    {
      self.require_handler(parser, CallOrNewExpr::New(new_expr), None)
    } else if for_name == CREATED_REQUIRE_IDENTIFIER_TAG {
      wrap_created_require_with_side_effects(parser, new_expr.span);
      self.require_handler(
        parser,
        CallOrNewExpr::New(new_expr),
        current_created_require_context(parser),
      )
    } else {
      None
    }
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    member_expr: &MemberExpr,
    callee_members: &[Atom],
    call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && let Some(argument) = parse_create_require_argument(parser, call_expr, false)
    {
      let side_effects = create_require_args_side_effects(parser, &call_expr.args, &argument);
      let unsupported_replacement = create_require_unsupported_member_replacement(&side_effects);
      handle_created_require_member(
        parser,
        member_expr.span(),
        require_cache_range(member_expr, member_ranges, members),
        members,
        unsupported_replacement,
      );
      if members
        .first()
        .is_some_and(|member| member.as_ref() == "cache")
      {
        wrap_span_with_side_effects(parser, member_expr.span(), &side_effects);
      }
      walk_create_require_ignored_args(parser, call_expr);
      return Some(true);
    }

    if callee_members.is_empty()
      && (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
      && let Some(dep) = self.chain_handler(parser, member_expr, call_expr, members, false)
    {
      parser.add_dependency(Box::new(dep));
      return Some(true);
    }
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    callee_members: &[Atom],
    inner_call_expr: &CallExpr,
    members: &[Atom],
    member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && members.len() == 1
      && members[0].as_ref() == "resolve"
    {
      if matches!(parser.javascript_options.require_resolve, Some(false))
        || call_expr.args.len() != 1
        || call_expr.args[0].spread.is_some()
      {
        walk_unsupported_create_require_resolve(parser, inner_call_expr, call_expr);
        return Some(true);
      }
      let argument = parse_create_require_argument(parser, inner_call_expr, false)?;
      let side_effects = create_require_args_side_effects(parser, &inner_call_expr.args, &argument);
      wrap_span_with_side_effects(parser, call_expr.span(), &side_effects);
      let context = argument.context;
      walk_create_require_ignored_args(parser, inner_call_expr);
      self.process_resolve(parser, call_expr, false, Some(context));
      return Some(true);
    }

    if callee_members.is_empty()
      && should_handle_create_require_specifier(parser, for_name)
      && let Some(argument) = parse_create_require_argument(parser, inner_call_expr, false)
    {
      let side_effects = create_require_args_side_effects(parser, &inner_call_expr.args, &argument);
      let unsupported_replacement = create_require_unsupported_member_replacement(&side_effects);
      let member_span = call_expr.callee.span();
      handle_created_require_member(
        parser,
        member_span,
        require_cache_range(
          call_expr.callee.as_expr()?.as_member()?,
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
      parser.walk_expr_or_spread(&call_expr.args);
      return Some(true);
    }

    if callee_members.is_empty()
      && (for_name == expr_name::REQUIRE || for_name == expr_name::MODULE_REQUIRE)
      && should_parse_commonjs_require(parser)
      && let Some(callee) = call_expr.callee.as_expr()
      && let Some(member) = callee.as_member()
      && let Some(dep) = self.chain_handler(parser, member, inner_call_expr, members, true)
    {
      parser.add_dependency(Box::new(dep));
      parser.walk_expr_or_spread(&call_expr.args);
      return Some(true);
    }
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &AssignExpr,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE && should_parse_commonjs_require(parser) {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (0, 0).into(),
        "var require;".into(),
      )));
      return Some(true);
    }

    if for_name == CREATED_REQUIRE_IDENTIFIER_TAG
      || for_name == CREATE_REQUIRE_SPECIFIER_TAG
      || for_name == CREATE_REQUIRE_EVALUATED_TAG
    {
      if matches!(expr.op, AssignOp::OrAssign | AssignOp::NullishAssign) {
        return Some(true);
      }
      clear_create_require_tag(parser, &Atom::from(ident.sym.as_str()));
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
    None
  }
}
