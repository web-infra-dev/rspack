use std::{iter, sync::Arc};

use either::Either;
use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ContextDependency, ContextMode, ContextOptions,
  Dependency, DependencyCategory, DependencyRange, RuntimeGlobals, RuntimeRequirementsDependency,
  get_context,
};
use rspack_error::{Error, Severity};
use rspack_intern::Atom;
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  Argument, ArrowFunctionBodyData, Ast, BindingPattern, CallExpression, FormalParameters, GetSpan,
};

use crate::{
  JavascriptParserPlugin,
  dependency::{
    AMDRequireContextDependency,
    amd_require_array_dependency::{AMDRequireArrayDependency, AMDRequireArrayItem},
    amd_require_dependency::AMDRequireDependency,
    amd_require_item_dependency::AMDRequireItemDependency,
    local_module_dependency::LocalModuleDependency,
    unsupported_dependency::UnsupportedDependency,
  },
  parser_plugin::require_ensure_dependencies_block_parse_plugin::GetFunctionExpression,
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    JavascriptParser, PatRef, context_reg_exp, create_context_dependency, create_traceable_error,
  },
};

fn formal_parameter_patterns(ast: &Ast<'_>, params: FormalParameters) -> Vec<BindingPattern> {
  let mut patterns = params
    .items(ast)
    .iter()
    .map(|id| ast.get_node_in_sub_range(id))
    .filter_map(|item| item.as_formal_parameter(ast))
    .filter_map(|parameter| parameter.pattern(ast).as_binding_pattern(ast))
    .collect::<Vec<_>>();
  if let Some(rest) = params.rest(ast) {
    patterns.push(BindingPattern::BindingRestElement(rest));
  }
  patterns
}

fn is_reserved_param(ast: &Ast<'_>, pat: BindingPattern) -> bool {
  const RESERVED_NAMES: [&str; 3] = ["require", "module", "exports"];
  pat
    .as_binding_identifier(ast)
    .is_some_and(|ident| RESERVED_NAMES.contains(&ast.get_utf8(ident.name(ast))))
}

pub struct AMDRequireDependenciesBlockParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for AMDRequireDependenciesBlockParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require" {
      self.process_call_require(parser, call_expr)
    } else {
      None
    }
  }
}

impl AMDRequireDependenciesBlockParserPlugin {
  fn process_array(
    &self,
    parser: &mut JavascriptParser,
    block_deps: &mut Vec<BoxDependency>,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    if param.is_array() {
      for item in param.items().iter() {
        let result = self.process_item(parser, block_deps, call_expr, item);
        if result.is_none() {
          self.process_context(parser, block_deps, call_expr, item);
        }
      }
      return Some(true);
    } else if param.is_const_array() {
      let mut deps: Vec<AMDRequireArrayItem> = vec![];
      let array = param.array();
      for request in array.iter() {
        if request == "require" {
          deps.push(AMDRequireArrayItem::Require);
        } else if request == "exports" || request == "module" {
          deps.push(AMDRequireArrayItem::String(request.into()));
        } else if let Some(local_module) = parser.get_local_module_mut(request) {
          local_module.flag_used();
          deps.push(AMDRequireArrayItem::LocalModuleDependency {
            local_module_variable_name: local_module.variable_name(),
          })
        } else {
          let mut dep = AMDRequireItemDependency::new(request.as_str().into(), None);
          dep.set_optional(parser.in_try);
          deps.push(AMDRequireArrayItem::AMDRequireItemDependency { dep_id: *dep.id() });
          block_deps.push(BoxDependency::new(dep));
        }
      }
      let range = param.range();
      let dep = AMDRequireArrayDependency::new(deps, range.into());
      parser.add_presentational_dependency(Arc::new(dep));
      return Some(true);
    }
    None
  }

  fn process_item(
    &self,
    parser: &mut JavascriptParser,
    block_deps: &mut Vec<BoxDependency>,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    if param.is_conditional() {
      let options = param.options();

      for option in options.iter() {
        let result = self.process_item(parser, block_deps, call_expr, option);
        if result.is_none() {
          self.process_context(parser, block_deps, call_expr, option);
        }
      }

      return Some(true);
    } else if param.is_string() {
      let param_str = param.string();
      let range = param.range();

      if param_str == "require" {
        let dep = Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::REQUIRE,
        ));
        parser.add_presentational_dependency(dep);
      } else if param_str == "module" {
        let dep = Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::MODULE,
        ));
        parser.add_presentational_dependency(dep);
      } else if param_str == "exports" {
        let dep = Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::EXPORTS,
        ));
        parser.add_presentational_dependency(dep);
      } else if let Some(local_module) = parser.get_local_module_mut(param_str) {
        local_module.flag_used();
        let dep = Arc::new(LocalModuleDependency::new(
          local_module.clone(),
          Some(range.into()),
          false,
        ));
        parser.add_presentational_dependency(dep);
        return Some(true);
      } else {
        let mut dep =
          AMDRequireItemDependency::new(Atom::new(param_str.as_str()), Some(range.into()));
        dep.set_optional(parser.in_try);
        block_deps.push(BoxDependency::new(dep));
      }

      return Some(true);
    }
    None
  }

  fn process_context(
    &self,
    parser: &mut JavascriptParser,
    block_deps: &mut Vec<BoxDependency>,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let call_span = call_expr.span(parser.ast.ast);
    let param_range = param.range();

    let result = create_context_dependency(param, parser);
    let request = result.request();

    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      pattern: context_reg_exp(&result.reg, "", Some(call_span.into()), parser).into(),
      category: DependencyCategory::Amd,
      request,
      context: get_context(parser.resource_data).to_string(),
      compiler_context: parser.compiler_options.context.clone(),
      replaces: result.replaces,
      start: call_span.real_lo(),
      end: call_span.real_hi(),
      ..Default::default()
    };
    let dep = AMDRequireContextDependency::new(options, param_range.into(), parser.in_try);
    dep.set_critical(result.critical);
    block_deps.push(BoxDependency::new(dep));
    Some(true)
  }

  fn process_array_for_request_string(&self, param: &BasicEvaluatedExpression) -> Option<String> {
    if param.is_array() {
      let mut result = param
        .items()
        .iter()
        .map(|item| self.process_item_for_request_string(item));
      if result.all(|item| item.is_some()) {
        return Some(result.map(|item| item.expect("")).join(" "));
      }
    }
    None
  }

  #[allow(clippy::only_used_in_recursion)]
  fn process_item_for_request_string(&self, param: &BasicEvaluatedExpression) -> Option<String> {
    if param.is_conditional() {
      let mut result = param
        .options()
        .iter()
        .map(|item| self.process_item_for_request_string(item));
      if result.all(|item| item.is_some()) {
        return Some(result.map(|item| item.expect("")).join("|"));
      }
    } else if param.is_string() {
      return Some(param.string().clone());
    }
    None
  }

  fn process_function_argument(&self, parser: &mut JavascriptParser, func_arg: Argument) -> bool {
    let mut bind_this = true;
    let ast = parser.ast.ast;
    let Some(func_arg_expr) = func_arg.as_expr(ast) else {
      parser.walk_arguments(iter::once(func_arg));
      return bind_this;
    };

    if let Some(func_expr) = func_arg_expr.get_function_expr(ast) {
      match func_expr.func {
        Either::Left(func) => {
          let params = formal_parameter_patterns(ast, func.params(ast));
          parser.in_function_scope(
            true,
            params
              .into_iter()
              .filter(|param| !is_reserved_param(ast, *param))
              .map(PatRef::Borrowed),
            |parser| parser.walk_function_body(func.body(parser.ast.ast)),
          );
        }
        Either::Right(arrow) => {
          let params = formal_parameter_patterns(ast, arrow.params(ast));
          parser.in_function_scope(
            true,
            params
              .into_iter()
              .filter(|param| !is_reserved_param(ast, *param))
              .map(PatRef::Borrowed),
            |parser| match parser
              .ast
              .ast
              .arrow_function_body_data(arrow.body(parser.ast.ast))
            {
              ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
              ArrowFunctionBodyData::Expr(expr) => parser.walk_expression(expr),
            },
          );
        }
      }

      if let Some(bind_expr) = func_expr.expressions {
        parser.walk_expression(bind_expr);
      }

      if func_expr._need_this.is_some_and(|x| !x) {
        bind_this = false;
      }
    } else {
      parser.walk_expression(func_arg_expr);
    }

    bind_this
  }

  fn process_call_require(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = call_expr
      .arguments(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .collect::<Vec<_>>();
    if args.is_empty() {
      return None;
    }
    // TODO: check if args includes spread

    // require(['dep1', 'dep2'], callback, errorCallback);

    let first_arg = *args.first().expect("first arg cannot be None");
    let callback_arg = args.get(1).copied();
    let error_callback_arg = args.get(2).copied();
    let first_arg_expr = first_arg.as_expr(ast)?;

    let param = parser.evaluate_expression(first_arg_expr);
    let call_span = call_expr.span(ast);

    let mut dep = AMDRequireDependency::new(
      call_span.into(),
      Some(first_arg_expr.span(ast).into()),
      callback_arg
        .and_then(|arg| arg.as_expr(ast))
        .map(|expr| expr.span(ast).into()),
      error_callback_arg
        .and_then(|arg| arg.as_expr(ast))
        .map(|expr| expr.span(ast).into()),
    );

    let range = DependencyRange::from(call_span);
    let block_loc = parser.to_dependency_location(range);

    if args.len() == 1 {
      let mut block_deps: Vec<BoxDependency> = vec![BoxDependency::new(dep)];
      let mut result = None;
      parser.in_function_scope(true, iter::empty(), |parser| {
        result = self.process_array(parser, &mut block_deps, call_expr, &param);
      });
      if result.is_some_and(|x| x) {
        let dep_block = Box::new(AsyncDependenciesBlock::new(
          *parser.module_identifier,
          block_loc,
          None,
          block_deps,
          self.process_array_for_request_string(&param),
        ));
        parser.add_block(dep_block);
        return Some(true);
      } else {
        return None;
      }
    }

    if args.len() == 2 || args.len() == 3 {
      let mut block_deps: Vec<BoxDependency> = vec![];

      let mut result = None;
      parser.in_function_scope(true, iter::empty(), |parser| {
        result = self.process_array(parser, &mut block_deps, call_expr, &param);
      });

      if !result.is_some_and(|x| x) {
        let dep = Arc::new(UnsupportedDependency::new(
          "unsupported".into(),
          call_span.into(),
        ));
        parser.add_presentational_dependency(dep);
        let mut error: Error = create_traceable_error(
          "UnsupportedFeatureWarning".into(),
          "Cannot statically analyse 'require(…, …)'".into(),
          parser.source.to_string(),
          call_span.into(),
        );
        error.severity = Severity::Warning;
        error.hide_stack = Some(true);
        parser.add_warning(error.into());
        return Some(true);
      }

      dep.function_bind_this =
        self.process_function_argument(parser, callback_arg.expect("2nd arg cannot be None"));

      if let Some(error_callback_arg) = error_callback_arg {
        dep.error_callback_bind_this = self.process_function_argument(parser, error_callback_arg);
      }

      block_deps.insert(0, BoxDependency::new(dep));
      let dep_block = Box::new(AsyncDependenciesBlock::new(
        *parser.module_identifier,
        block_loc,
        None,
        block_deps,
        self.process_array_for_request_string(&param),
      ));
      parser.add_block(dep_block);

      return Some(true);
    }

    None
  }
}
