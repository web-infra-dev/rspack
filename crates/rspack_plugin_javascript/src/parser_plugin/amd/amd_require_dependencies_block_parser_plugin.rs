use std::{borrow::Cow, iter};

use either::Either;
use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ConstDependency, ContextDependency, ContextMode,
  ContextNameSpaceObject, ContextOptions, Dependency, DependencyCategory, DependencyRange,
  RuntimeGlobals, SharedSourceMap,
};
use rspack_error::{Error, Severity};
use rspack_util::{SpanExt, atom::Atom};
use swc_core::{
  common::Spanned,
  ecma::ast::{BlockStmtOrExpr, CallExpr, ExprOrSpread, Pat},
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
    JavascriptParser, Statement, context_reg_exp, create_context_dependency, create_traceable_error,
  },
};

fn is_reserved_param(pat: &Pat) -> bool {
  const RESERVED_NAMES: [&str; 3] = ["require", "module", "exports"];
  pat
    .as_ident()
    .is_some_and(|ident| RESERVED_NAMES.contains(&ident.id.sym.as_str()))
}

pub struct AMDRequireDependenciesBlockParserPlugin;

impl JavascriptParserPlugin for AMDRequireDependenciesBlockParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
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
    call_expr: &CallExpr,
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
          deps.push(AMDRequireArrayItem::String(
            parser
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::REQUIRE),
          ));
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
          block_deps.push(Box::new(dep));
        }
      }
      let range = param.range();
      let dep = AMDRequireArrayDependency::new(deps, range.into());
      parser.add_presentational_dependency(Box::new(dep));
      return Some(true);
    }
    None
  }

  fn process_item(
    &self,
    parser: &mut JavascriptParser,
    block_deps: &mut Vec<BoxDependency>,
    call_expr: &CallExpr,
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
        let dep = Box::new(ConstDependency::new(
          range.into(),
          parser
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE)
            .into(),
          Some(RuntimeGlobals::REQUIRE),
        ));
        parser.add_presentational_dependency(dep);
      } else if param_str == "module" {
        let dep = Box::new(ConstDependency::new(
          range.into(),
          "module".into(),
          Some(RuntimeGlobals::MODULE),
        ));
        parser.add_presentational_dependency(dep);
      } else if param_str == "exports" {
        let dep = Box::new(ConstDependency::new(
          range.into(),
          "exports".into(),
          Some(RuntimeGlobals::EXPORTS),
        ));
        parser.add_presentational_dependency(dep);
      } else if let Some(local_module) = parser.get_local_module_mut(param_str) {
        local_module.flag_used();
        let dep = Box::new(LocalModuleDependency::new(
          local_module.clone(),
          Some(range.into()),
          false,
        ));
        parser.add_presentational_dependency(dep);
        return Some(true);
      } else {
        let mut dep = Box::new(AMDRequireItemDependency::new(
          Atom::new(param_str.as_str()),
          Some(range.into()),
        ));
        dep.set_optional(parser.in_try);
        block_deps.push(dep);
      }

      return Some(true);
    }
    None
  }

  fn process_context(
    &self,
    parser: &mut JavascriptParser,
    block_deps: &mut Vec<BoxDependency>,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let call_span = call_expr.span();
    let param_range = param.range();

    let result = create_context_dependency(param, parser);

    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      reg_exp: context_reg_exp(&result.reg, "", Some(call_expr.span().into()), parser),
      include: None,
      exclude: None,
      category: DependencyCategory::Amd,
      request: format!("{}{}{}", result.context, result.query, result.fragment),
      context: result.context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      replaces: result.replaces,
      start: call_span.real_lo(),
      end: call_span.real_hi(),
      referenced_exports: None,
      attributes: None,
    };
    let mut dep = AMDRequireContextDependency::new(options, param_range.into(), parser.in_try);
    *dep.critical_mut() = result.critical;
    block_deps.push(Box::new(dep));
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
      return Some(param.string().to_string());
    }
    None
  }

  fn process_function_argument(
    &self,
    parser: &mut JavascriptParser,
    func_arg: &ExprOrSpread,
  ) -> bool {
    let mut bind_this = true;

    if let Some(func_expr) = func_arg.expr.get_function_expr() {
      match func_expr.func {
        Either::Left(func) => {
          if let Some(body) = &func.function.body {
            let params = func
              .function
              .params
              .iter()
              .filter(|param| !is_reserved_param(&param.pat))
              .map(|param| Cow::Borrowed(&param.pat));
            parser.in_function_scope(true, params, |parser| {
              parser.walk_statement(Statement::Block(body));
            });
          }
        }
        Either::Right(arrow) => {
          let params = arrow
            .params
            .iter()
            .filter(|param| !is_reserved_param(param))
            .map(Cow::Borrowed);
          parser.in_function_scope(true, params, |parser| match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(body) => parser.walk_statement(Statement::Block(body)),
            BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
          });
        }
      }

      if let Some(bind_expr) = func_expr.expressions {
        parser.walk_expression(bind_expr);
      }

      if func_expr._need_this.is_some_and(|x| !x) {
        bind_this = false;
      }
    } else {
      parser.walk_expression(&func_arg.expr);
    }

    bind_this
  }

  fn process_call_require(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    if call_expr.args.is_empty() {
      return None;
    }
    // TODO: check if args includes spread

    // require(['dep1', 'dep2'], callback, errorCallback);

    let first_arg = call_expr.args.first().expect("first arg cannot be None");
    let callback_arg = call_expr.args.get(1);
    let error_callback_arg = call_expr.args.get(2);

    let param = parser.evaluate_expression(&first_arg.expr);

    let mut dep = Box::new(AMDRequireDependency::new(
      call_expr.span.into(),
      Some(first_arg.expr.span().into()),
      callback_arg.map(|arg| arg.expr.span().into()),
      error_callback_arg.map(|arg| arg.expr.span().into()),
    ));

    let source_map: SharedSourceMap = parser.source().clone();
    let block_loc = Into::<DependencyRange>::into(call_expr.span).to_loc(Some(source_map.as_ref()));

    if call_expr.args.len() == 1 {
      let mut block_deps: Vec<BoxDependency> = vec![dep];
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

    if call_expr.args.len() == 2 || call_expr.args.len() == 3 {
      let mut block_deps: Vec<BoxDependency> = vec![];

      let mut result = None;
      parser.in_function_scope(true, iter::empty(), |parser| {
        result = self.process_array(parser, &mut block_deps, call_expr, &param);
      });

      if !result.is_some_and(|x| x) {
        let dep = Box::new(UnsupportedDependency::new(
          "unsupported".into(),
          call_expr.span.into(),
        ));
        parser.add_presentational_dependency(dep);
        let mut error: Error = create_traceable_error(
          "UnsupportedFeatureWarning".into(),
          "Cannot statically analyse 'require(…, …)'".into(),
          parser.source.to_string(),
          call_expr.span.into(),
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

      block_deps.insert(0, dep);
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
