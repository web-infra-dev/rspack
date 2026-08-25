use std::{borrow::Cow, sync::Arc};

use rspack_core::{
  BoxDependency, BuildMetaDefaultObject, ContextDependency, ContextMode, ContextOptions,
  Dependency, DependencyCategory, DependencyCodeGenerationRef, RuntimeGlobals,
  RuntimeRequirementsDependency, get_context,
};
use rspack_util::{SpanExt, atom::Atom};
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
  ArrowFunctionBodyData, Ast, BindingPattern, CallExpression, Expr, ExprData, FormalParameters,
  GetSpan, PropertyKeyData,
};

use crate::{
  JavascriptParserPlugin,
  dependency::{
    AMDRequireContextDependency,
    amd_define_dependency::AMDDefineDependency,
    amd_require_array_dependency::{AMDRequireArrayDependency, AMDRequireArrayItem},
    amd_require_item_dependency::AMDRequireItemDependency,
    local_module_dependency::LocalModuleDependency,
  },
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ExportedVariableInfo, JavascriptParser, PatRef, context_reg_exp, create_context_dependency,
  },
};

pub struct AMDDefineDependencyParserPlugin;

fn formal_parameter_patterns<'a>(
  ast: &'a Ast<'a>,
  params: FormalParameters,
) -> impl Iterator<Item = BindingPattern> + 'a {
  params
    .items(ast)
    .iter()
    .map(|id| ast.get_node_in_sub_range(id))
    .filter_map(|item| item.as_formal_parameter(ast))
    .filter_map(|parameter| parameter.pattern(ast).as_binding_pattern(ast))
    .chain(params.rest(ast).map(BindingPattern::BindingRestElement))
}

fn is_unbound_function_expression(ast: &Ast<'_>, expr: Expr) -> bool {
  expr.as_function(ast).is_some() || expr.as_arrow_function_expression(ast).is_some()
}

fn is_bound_function_expression(ast: &Ast<'_>, expr: Expr) -> bool {
  let Some(call_expr) = expr.as_call_expression(ast) else {
    return false;
  };
  let Some(callee_member) = call_expr.callee(ast).as_member_expression(ast) else {
    return false;
  };
  if callee_member.computed(ast) || callee_member.object(ast).as_function(ast).is_none() {
    return false;
  }
  let PropertyKeyData::IdentifierName(property) =
    ast.property_key_data(callee_member.property(ast))
  else {
    return false;
  };
  if ast.get_utf8(property.name(ast)) != "bind" {
    return false;
  }

  true
}

fn is_callable(ast: &Ast<'_>, expr: Expr) -> bool {
  is_unbound_function_expression(ast, expr) || is_bound_function_expression(ast, expr)
}

/// `define('ui/foo/bar', ['./baz', '../qux'], ...);`
/// - 'ui/foo/baz'
/// - 'ui/qux'
fn lookup<'a>(parent: &str, module: &'a str) -> Cow<'a, str> {
  if module.starts_with('.') {
    let mut path: Vec<&str> = parent.split('/').collect();
    path.pop();

    for seg in module.split('/') {
      if seg == ".." {
        path.pop();
      } else if seg != "." {
        path.push(seg);
      }
    }

    path.join("/").into()
  } else {
    module.into()
  }
}

const REQUIRE: &str = "require";
const MODULE: &str = "module";
const EXPORTS: &str = "exports";
const RESERVED_NAMES: [&str; 3] = [REQUIRE, EXPORTS, MODULE];

fn get_lit_str(ast: &Ast<'_>, expr: Expr) -> Option<Atom> {
  let string = expr.as_string_literal(ast)?;
  Some(Atom::new(
    ast.get_wtf8(string.value(ast)).to_string_lossy().as_ref(),
  ))
}

fn is_literal(ast: &Ast<'_>, expr: Expr) -> bool {
  matches!(
    ast.expr_data(expr),
    ExprData::StringLiteral(_)
      | ExprData::NumericLiteral(_)
      | ExprData::BigIntLiteral(_)
      | ExprData::BooleanLiteral(_)
      | ExprData::NullLiteral(_)
      | ExprData::RegExpLiteral(_)
  )
}

fn get_ident_name(ast: &Ast<'_>, pat: &PatRef) -> Atom {
  pat
    .as_pat()
    .as_binding_identifier(ast)
    .map_or("".into(), |ident| Atom::new(ast.get_utf8(ident.name(ast))))
}

impl AMDDefineDependencyParserPlugin {
  fn process_array(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
    identifiers: &mut FxHashMap<usize, Atom>, // param index => "require" | "module" | "exports"
    named_module: &Option<Atom>,
  ) -> Option<bool> {
    if param.is_array() {
      let items = param.items();
      for (idx, item) in items.iter().enumerate() {
        if item.is_string() {
          let item = item.string().as_str();
          if RESERVED_NAMES.contains(&item) {
            identifiers.insert(idx, item.into());
          }
        }
        let result = self.process_item(parser, call_expr, item, named_module);
        if result.is_none() {
          self.process_context(parser, call_expr, item);
        }
      }
      return Some(true);
    } else if param.is_const_array() {
      let mut deps: Vec<AMDRequireArrayItem> = vec![];
      let array = param.array();
      for (i, request) in array.iter().enumerate() {
        if request == "require" {
          identifiers.insert(i, REQUIRE.into());
          deps.push(AMDRequireArrayItem::Require);
        } else if request == "exports" {
          identifiers.insert(i, EXPORTS.into());
          deps.push(AMDRequireArrayItem::String(request.into()));
        } else if request == "module" {
          identifiers.insert(i, MODULE.into());
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
          parser.add_dependency(BoxDependency::new(dep));
        }
      }
      let dep = AMDRequireArrayDependency::new(deps, param.range().into());
      parser.add_presentational_dependency(Arc::new(dep));
      return Some(true);
    }
    None
  }

  fn process_item(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    param: &BasicEvaluatedExpression,
    named_module: &Option<Atom>,
  ) -> Option<bool> {
    if param.is_conditional() {
      let options = param.options();

      for option in options.iter() {
        let result = self.process_item(parser, call_expr, option, &None);
        if result.is_none() {
          self.process_context(parser, call_expr, option);
        }
      }

      return Some(true);
    } else if param.is_string() {
      let param_str = param.string();
      let range = param.range();

      let dep: DependencyCodeGenerationRef = if param_str == "require" {
        Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::REQUIRE,
        ))
      } else if param_str == "exports" {
        Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::EXPORTS,
        ))
      } else if param_str == "module" {
        Arc::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::MODULE,
        ))
      } else if let Some(local_module) = parser.get_local_module_mut(
        &named_module
          .as_ref()
          .map(|parent| lookup(parent, param_str))
          .unwrap_or(param_str.into()),
      ) {
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
        parser.add_dependency(BoxDependency::new(dep));
        return Some(true);
      };
      // TODO: how to implement this?
      // dep.loc = /** @type {DependencyLocation} */ (expr.loc);
      parser.add_presentational_dependency(dep);
      return Some(true);
    }
    None
  }

  fn process_context(
    &self,
    parser: &mut JavascriptParser,
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
    parser.add_dependency(BoxDependency::new(dep));
    Some(true)
  }

  fn process_call_define(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = call_expr.arguments(ast);
    let mut array: Option<Expr> = None;
    let mut func: Option<Expr> = None;
    let mut obj: Option<Expr> = None;
    let mut named_module: Option<Atom> = None;

    match args.len() {
      1 => {
        // We don't support spread syntax in `define()`.
        let first_arg = args.get_node(ast, 0)?.as_expr(ast)?;

        if is_callable(ast, first_arg) {
          // define(f() {…})
          func = Some(first_arg);
        } else if first_arg.as_object_expression(ast).is_some() {
          // define({…})
          obj = Some(first_arg);
        } else {
          // define(expr)
          // unclear if function or object
          func = Some(first_arg);
          obj = Some(first_arg);
        }
      }
      2 => {
        let first_arg = args.get_node(ast, 0)?.as_expr(ast)?;
        let second_arg = args.get_node(ast, 1)?.as_expr(ast)?;

        if is_literal(ast, first_arg) {
          // define("…", …)
          named_module = get_lit_str(ast, first_arg);

          if is_callable(ast, second_arg) {
            // define("…", f() {…})
            func = Some(second_arg);
          } else if second_arg.as_object_expression(ast).is_some() {
            // define("…", {…})
            obj = Some(second_arg);
          } else {
            // define("…", expr)
            // unclear if function or object
            func = Some(second_arg);
            obj = Some(second_arg);
          }
        } else {
          // define([…], …)
          array = Some(first_arg);

          if is_callable(ast, second_arg) {
            // define([…], f() {})
            func = Some(second_arg);
          } else if second_arg.as_object_expression(ast).is_some() {
            // define([…], {…})
            obj = Some(second_arg);
          } else {
            // define([…], expr)
            // unclear if function or object
            func = Some(second_arg);
            obj = Some(second_arg);
          }
        }
      }
      3 => {
        // define("…", […], …)

        let first_arg = args.get_node(ast, 0)?.as_expr(ast)?;
        let second_arg = args.get_node(ast, 1)?.as_expr(ast)?;
        let third_arg = args.get_node(ast, 2)?.as_expr(ast)?;

        if !is_literal(ast, first_arg) {
          return None;
        }
        second_arg.as_array_expression(ast)?;

        named_module = get_lit_str(ast, first_arg);
        array = Some(second_arg);

        if is_callable(ast, third_arg) {
          // define("…", […], f() {})
          func = Some(third_arg);
        } else if third_arg.as_object_expression(ast).is_some() {
          // define("…", […], {…})
          obj = Some(third_arg);
        } else {
          // define("…", […], expr)
          // unclear if function or object
          func = Some(third_arg);
          obj = Some(third_arg);
        }
      }
      _ => return None,
    }

    {
      // DynamicExports.bailout(parser.state);
      //  TODO: consider how to share this code
      if parser.parser_exports_state.is_some_and(|x| x) {
        parser.build_meta.clear_exports_type();
        parser
          .build_meta
          .set_default_object(BuildMetaDefaultObject::False);
      }
      parser.parser_exports_state = Some(false);
    }

    let mut fn_params: Option<Vec<PatRef>> = None;
    let mut fn_params_offset = 0usize;
    if let Some(func) = func {
      if is_unbound_function_expression(ast, func) {
        fn_params = match ast.expr_data(func) {
          ExprData::Function(normal_func) => Some(
            formal_parameter_patterns(ast, normal_func.params(ast))
              .map(PatRef::Borrowed)
              .collect(),
          ),
          ExprData::ArrowFunctionExpression(arrow_func) => Some(
            formal_parameter_patterns(ast, arrow_func.params(ast))
              .map(PatRef::Borrowed)
              .collect(),
          ),
          _ => None,
        };
      } else if is_bound_function_expression(ast, func) {
        let call_expr = func
          .as_call_expression(ast)
          .expect("call_expr is supposed to be a CallExpression");
        let object = call_expr
          .callee(ast)
          .as_member_expression(ast)
          .expect("call_expr.callee is supposed to be MemberExpression")
          .object(ast)
          .as_function(ast)
          .expect("call_expr.callee.obj is supposed to be Function");

        fn_params = Some(
          formal_parameter_patterns(ast, object.params(ast))
            .map(PatRef::Borrowed)
            .collect(),
        );

        if !call_expr.arguments(ast).is_empty() {
          fn_params_offset = call_expr.arguments(ast).len() - 1;
        }
      }
    }

    // TODO: ensure all fn_params are identifiers

    let mut fn_renames = FxHashMap::default();
    if let Some(array) = array {
      let mut identifiers = FxHashMap::default();
      let param = parser.evaluate_expression(array);
      let result = self.process_array(parser, call_expr, &param, &mut identifiers, &named_module);
      if !result.is_some_and(|b| b) {
        return None;
      }
      if let Some(fn_params) = &mut fn_params {
        let mut i = 0usize;
        fn_params.retain(|param| {
          if i < fn_params_offset {
            return false;
          }
          let idx = i - fn_params_offset;
          i += 1;
          if let Some(name) = identifiers.get(&idx) {
            fn_renames.insert(get_ident_name(ast, param), name.clone());
            return false;
          }
          true
        });
      }
    } else if let Some(fn_params) = &mut fn_params {
      let mut i = 0usize;
      fn_params.retain(|param| {
        if i < fn_params_offset {
          return false;
        }
        let idx = i - fn_params_offset;
        i += 1;
        if idx < RESERVED_NAMES.len() {
          fn_renames.insert(get_ident_name(ast, param), RESERVED_NAMES[idx].into());
          return false;
        }
        true
      });
    }

    if func.is_some_and(|func| is_unbound_function_expression(ast, func)) {
      let in_try = parser.in_try;
      parser.in_function_scope(
        true,
        fn_params.expect("fn_params should not be None").into_iter(),
        |parser| {
          for (name, rename_identifier) in fn_renames.iter() {
            let variable = parser
              .get_variable_info(rename_identifier)
              .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
              .unwrap_or(ExportedVariableInfo::Name(rename_identifier.clone()));
            parser.set_variable(name.clone(), variable);
          }

          parser.in_try = in_try;

          match func.map(|func| parser.ast.ast.expr_data(func)) {
            Some(ExprData::Function(function)) => {
              parser.walk_function_body(function.body(parser.ast.ast));
            }
            Some(ExprData::ArrowFunctionExpression(function)) => {
              match parser
                .ast
                .ast
                .arrow_function_body_data(function.body(parser.ast.ast))
              {
                ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
                ArrowFunctionBodyData::Expr(expr) => parser.walk_expression(expr),
              }
            }
            _ => unreachable!(),
          }
        },
      );
    } else if func.is_some_and(|func| is_bound_function_expression(ast, func)) {
      let in_try = parser.in_try;

      if let Some(call_expr) = func.and_then(|f| f.as_call_expression(ast)) {
        let object = call_expr
          .callee(ast)
          .as_member_expression(ast)
          .and_then(|member_expr| member_expr.object(ast).as_function(ast));

        if let Some(func_expr) = object {
          let params = formal_parameter_patterns(ast, func_expr.params(ast));
          parser.in_function_scope(
            true,
            params.map(PatRef::Borrowed).filter(|pat| {
              pat
                .as_pat()
                .as_binding_identifier(ast)
                .is_some_and(|ident| !RESERVED_NAMES.contains(&ast.get_utf8(ident.name(ast))))
            }),
            |parser| {
              for (name, rename_identifier) in fn_renames.iter() {
                let variable = parser
                  .get_variable_info(rename_identifier)
                  .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
                  .unwrap_or(ExportedVariableInfo::Name(rename_identifier.clone()));
                parser.set_variable(name.clone(), variable);
              }

              parser.in_try = in_try;

              parser.walk_function_body(func_expr.body(parser.ast.ast));
            },
          );
        }

        parser.walk_arguments(
          call_expr
            .arguments(parser.ast.ast)
            .iter()
            .map(|id| parser.ast.ast.get_node_in_sub_range(id)),
        );
      }
    } else if let Some(expr) = func {
      parser.walk_expression(expr);
    } else if let Some(expr) = obj {
      parser.walk_expression(expr);
    }

    if let Some(name) = &named_module {
      let dep_idx = parser.next_presentational_dependency_idx();
      parser.add_local_module(name, dep_idx);
    }

    let dep = Arc::new(AMDDefineDependency::new(
      call_expr.span(ast).into(),
      array.map(|expr| expr.span(ast).into()),
      func.map(|expr| expr.span(ast).into()),
      obj.map(|expr| expr.span(ast).into()),
      named_module,
    ));

    parser.add_presentational_dependency(dep);

    Some(true)
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for AMDDefineDependencyParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "define" {
      self.process_call_define(parser, call_expr)
    } else {
      None
    }
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    for local_module in std::mem::take(&mut parser.local_modules) {
      let dep_idx = local_module.amd_dep_idx();
      if let Some(dep) = parser.get_presentational_dependency_mut(dep_idx)
        && let Some(dep) = dep.as_any_mut().downcast_mut::<AMDDefineDependency>()
      {
        dep.set_local_module(local_module);
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_lookup() {
    assert_eq!(lookup("ui/foo", "./bar"), "ui/bar");
    assert_eq!(lookup("ui/foo", "../bar"), "bar");
    assert_eq!(lookup("ui/foo", "bar"), "bar");
  }
}
