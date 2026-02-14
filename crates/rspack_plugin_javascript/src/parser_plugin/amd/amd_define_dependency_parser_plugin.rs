use std::borrow::Cow;

use rspack_core::{
  BoxDependencyTemplate, BuildMetaDefaultObject, BuildMetaExportsType, ContextDependency,
  ContextMode, ContextNameSpaceObject, ContextOptions, Dependency, DependencyCategory,
  RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_util::{SpanExt, atom::Atom};
use rustc_hash::FxHashMap;
use swc_experimental_ecma_ast::{Ast, BlockStmtOrExpr, CallExpr, Callee, Expr, GetSpan, Lit, Pat};

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
    ExportedVariableInfo, JavascriptParser, Statement, context_reg_exp, create_context_dependency,
  },
};

pub struct AMDDefineDependencyParserPlugin;

fn is_unbound_function_expression(expr: Expr) -> bool {
  expr.is_fn() || expr.is_arrow()
}

fn is_bound_function_expression(ast: &Ast, expr: Expr) -> bool {
  if !expr.is_call() {
    return false;
  }

  let call_expr = expr.as_call().expect("expr is supposed to be CallExpr");
  match call_expr.callee(ast) {
    Callee::Super(_) => return false,
    Callee::Import(_) => return false,
    Callee::Expr(callee) => {
      if !callee.is_member() {
        return false;
      }
      let callee_member = callee
        .as_member()
        .expect("callee is supposed to be MemberExpr");
      if callee_member.prop(ast).is_computed() {
        return false;
      }
      if !callee_member.obj(ast).is_fn() {
        return false;
      }
      if !callee_member.prop(ast).is_ident_with(ast, "bind") {
        return false;
      }
    }
  }

  true
}

fn is_callable(ast: &Ast, expr: Expr) -> bool {
  is_unbound_function_expression(expr) || is_bound_function_expression(ast, expr)
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

fn get_lit_str(ast: &Ast, expr: Expr) -> Option<Atom> {
  expr.as_lit().and_then(|lit| match lit {
    Lit::Str(s) => Some(ast.get_wtf8_atom(s.value(ast)).to_atom_lossy().into_owned()),
    _ => None,
  })
}

fn get_ident_name(ast: &Ast, pat: Pat) -> Atom {
  pat
    .as_ident()
    .map_or("".into(), |ident| ast.get_atom(ident.id(ast).sym(ast)))
}

impl AMDDefineDependencyParserPlugin {
  fn process_array(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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
          parser.add_dependency(Box::new(dep));
        }
      }
      let dep = AMDRequireArrayDependency::new(deps, param.range().into());
      parser.add_presentational_dependency(Box::new(dep));
      return Some(true);
    }
    None
  }

  fn process_item(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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

      let dep: BoxDependencyTemplate = if param_str == "require" {
        Box::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::REQUIRE,
        ))
      } else if param_str == "exports" {
        Box::new(RuntimeRequirementsDependency::new(
          range.into(),
          RuntimeGlobals::EXPORTS,
        ))
      } else if param_str == "module" {
        Box::new(RuntimeRequirementsDependency::new(
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
        parser.add_dependency(dep);
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
    call_expr: CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let call_span = call_expr.span(&parser.ast);
    let param_range = param.range();

    let result = create_context_dependency(param, parser);

    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      reg_exp: context_reg_exp(
        &result.reg,
        "",
        Some(call_expr.span(&parser.ast).into()),
        parser,
      ),
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
      phase: None,
    };
    let mut dep = AMDRequireContextDependency::new(options, param_range.into(), parser.in_try);
    *dep.critical_mut() = result.critical;
    parser.add_dependency(Box::new(dep));
    Some(true)
  }

  fn process_call_define(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
  ) -> Option<bool> {
    let mut array: Option<Expr> = None;
    let mut func: Option<Expr> = None;
    let mut obj: Option<Expr> = None;
    let mut named_module: Option<Atom> = None;

    match call_expr.args(&parser.ast).len() {
      1 => {
        let first_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 0)
          .unwrap();

        // We don't support spread syntax in `define()`
        if first_arg.spread(&parser.ast).is_some() {
          return None;
        }

        if is_callable(&parser.ast, first_arg.expr(&parser.ast)) {
          // define(f() {…})
          func = Some(first_arg.expr(&parser.ast));
        } else if first_arg.expr(&parser.ast).is_object() {
          // define({…})
          obj = Some(first_arg.expr(&parser.ast));
        } else {
          // define(expr)
          // unclear if function or object
          func = Some(first_arg.expr(&parser.ast));
          obj = Some(first_arg.expr(&parser.ast));
        }
      }
      2 => {
        let first_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 0)
          .unwrap();
        let second_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 1)
          .unwrap();

        // We don't support spread syntax in `define()`
        if first_arg.spread(&parser.ast).is_some() || second_arg.spread(&parser.ast).is_some() {
          return None;
        }

        if first_arg.expr(&parser.ast).is_lit() {
          // define("…", …)
          named_module = get_lit_str(&parser.ast, first_arg.expr(&parser.ast));

          if is_callable(&parser.ast, second_arg.expr(&parser.ast)) {
            // define("…", f() {…})
            func = Some(second_arg.expr(&parser.ast));
          } else if second_arg.expr(&parser.ast).is_object() {
            // define("…", {…})
            obj = Some(second_arg.expr(&parser.ast));
          } else {
            // define("…", expr)
            // unclear if function or object
            func = Some(second_arg.expr(&parser.ast));
            obj = Some(second_arg.expr(&parser.ast));
          }
        } else {
          // define([…], …)
          array = Some(first_arg.expr(&parser.ast));

          if is_callable(&parser.ast, second_arg.expr(&parser.ast)) {
            // define([…], f() {})
            func = Some(second_arg.expr(&parser.ast));
          } else if second_arg.expr(&parser.ast).is_object() {
            // define([…], {…})
            obj = Some(second_arg.expr(&parser.ast));
          } else {
            // define([…], expr)
            // unclear if function or object
            func = Some(second_arg.expr(&parser.ast));
            obj = Some(second_arg.expr(&parser.ast));
          }
        }
      }
      3 => {
        // define("…", […], …)

        let first_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 0)
          .unwrap();
        let second_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 1)
          .unwrap();
        let third_arg = call_expr
          .args(&parser.ast)
          .get_node(&parser.ast, 2)
          .unwrap();

        // We don't support spread syntax in `define()`
        if first_arg.spread(&parser.ast).is_some()
          || second_arg.spread(&parser.ast).is_some()
          || third_arg.spread(&parser.ast).is_some()
        {
          return None;
        }

        if !first_arg.expr(&parser.ast).is_lit() {
          return None;
        }
        if !second_arg.expr(&parser.ast).is_array() {
          return None;
        }

        named_module = get_lit_str(&parser.ast, first_arg.expr(&parser.ast));
        array = Some(second_arg.expr(&parser.ast));

        if is_callable(&parser.ast, third_arg.expr(&parser.ast)) {
          // define("…", […], f() {})
          func = Some(third_arg.expr(&parser.ast));
        } else if third_arg.expr(&parser.ast).is_object() {
          // define("…", […], {…})
          obj = Some(third_arg.expr(&parser.ast));
        } else {
          // define("…", […], expr)
          // unclear if function or object
          func = Some(third_arg.expr(&parser.ast));
          obj = Some(third_arg.expr(&parser.ast));
        }
      }
      _ => return None,
    }

    {
      // DynamicExports.bailout(parser.state);
      //  TODO: consider how to share this code
      if parser.parser_exports_state.is_some_and(|x| x) {
        parser.build_meta.exports_type = BuildMetaExportsType::Unset;
        parser.build_meta.default_object = BuildMetaDefaultObject::False;
      }
      parser.parser_exports_state = Some(false);
    }

    let mut fn_params: Option<Vec<Pat>> = None;
    let mut fn_params_offset = 0usize;
    if let Some(func) = func {
      if is_unbound_function_expression(func) {
        fn_params = match func {
          Expr::Fn(normal_func) => Some(
            normal_func
              .function(&parser.ast)
              .params(&parser.ast)
              .iter()
              .map(|param| parser.ast.get_node_in_sub_range(param).pat(&parser.ast))
              .collect(),
          ),
          Expr::Arrow(array_func) => Some(
            array_func
              .params(&parser.ast)
              .iter()
              .map(|i| parser.ast.get_node_in_sub_range(i))
              .collect(),
          ),
          _ => None,
        };
      } else if is_bound_function_expression(&parser.ast, func) {
        let call_expr = func
          .as_call()
          .expect("call_expr is supposed to be a CallExpr");
        let object = call_expr
          .callee(&parser.ast)
          .as_expr()
          .expect("call_expr.callee is supposed to be Expr")
          .as_member()
          .expect("call_expr.callee is supposed to be MemberExpr")
          .obj(&parser.ast)
          .as_fn()
          .expect("call_expr.callee.obj is supposed to be FnExpr");

        fn_params = Some(
          object
            .function(&parser.ast)
            .params(&parser.ast)
            .iter()
            .map(|param| parser.ast.get_node_in_sub_range(param).pat(&parser.ast))
            .collect(),
        );

        if !call_expr.args(&parser.ast).is_empty() {
          fn_params_offset = call_expr.args(&parser.ast).len() - 1;
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
            fn_renames.insert(get_ident_name(&parser.ast, *param), name.clone());
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
          fn_renames.insert(
            get_ident_name(&parser.ast, *param),
            RESERVED_NAMES[idx].into(),
          );
          return false;
        }
        true
      });
    }

    if func.is_some_and(is_unbound_function_expression) {
      let in_try = parser.in_try;
      parser.in_function_scope(
        true,
        fn_params
          .expect("fn_params should not be None")
          .into_iter()
          .map(|p| move |_: &Ast| Some(p)),
        |parser| {
          for (name, rename_identifier) in fn_renames.iter() {
            let variable = parser
              .get_variable_info(rename_identifier)
              .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
              .unwrap_or(ExportedVariableInfo::Name(rename_identifier.clone()));
            parser.set_variable(name.clone(), variable);
          }

          parser.in_try = in_try;

          if let Some(func) = func.and_then(|f| f.as_fn()) {
            if let Some(body) = func.function(&parser.ast).body(&parser.ast) {
              parser.detect_mode(body.stmts(&parser.ast));
              let prev = parser.prev_statement;
              parser.pre_walk_statement(Statement::Block(body));
              parser.prev_statement = prev;
              parser.walk_statement(Statement::Block(body));
            }
          } else if let Some(func) = func.and_then(|f| f.as_arrow()) {
            match func.body(&parser.ast) {
              BlockStmtOrExpr::BlockStmt(stmt) => {
                parser.detect_mode(stmt.stmts(&parser.ast));
                let prev = parser.prev_statement;
                parser.pre_walk_statement(Statement::Block(stmt));
                parser.prev_statement = prev;
                parser.walk_statement(Statement::Block(stmt));
              }
              BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
            }
          }
        },
      );
    } else if func.is_some_and(|f| is_bound_function_expression(&parser.ast, f)) {
      let in_try = parser.in_try;

      if let Some(call_expr) = func.and_then(|f| f.as_call()) {
        let object = call_expr
          .callee(&parser.ast)
          .as_expr()
          .and_then(|expr| expr.as_member())
          .and_then(|member_expr| member_expr.obj(&parser.ast).as_fn());

        if let Some(func_expr) = object {
          let params = func_expr
            .function(&parser.ast)
            .params(&parser.ast)
            .iter()
            .map(|param| {
              move |ast: &Ast| {
                let pat = ast.get_node_in_sub_range(param).pat(ast);
                let Some(ident) = pat.as_ident() else {
                  return None;
                };
                (!RESERVED_NAMES.contains(&ast.get_utf8(ident.id(ast).sym(ast)))).then_some(pat)
              }
            });
          parser.in_function_scope(true, params, |parser| {
            for (name, rename_identifier) in fn_renames.iter() {
              let variable = parser
                .get_variable_info(rename_identifier)
                .map(|info| ExportedVariableInfo::VariableInfo(info.id()))
                .unwrap_or(ExportedVariableInfo::Name(rename_identifier.clone()));
              parser.set_variable(name.clone(), variable);
            }

            parser.in_try = in_try;

            if let Some(body) = func_expr.function(&parser.ast).body(&parser.ast) {
              parser.detect_mode(body.stmts(&parser.ast));
              let prev = parser.prev_statement;
              parser.pre_walk_statement(Statement::Block(body));
              parser.prev_statement = prev;
              parser.walk_statement(Statement::Block(body));
            }
          });
        }

        parser.walk_expr_or_spread(call_expr.args(&parser.ast));
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

    let dep = Box::new(AMDDefineDependency::new(
      call_expr.span(&parser.ast).into(),
      array.map(|expr| expr.span(&parser.ast).into()),
      func.map(|expr| expr.span(&parser.ast).into()),
      obj.map(|expr| expr.span(&parser.ast).into()),
      named_module,
    ));

    parser.add_presentational_dependency(dep);

    Some(true)
  }
}

impl JavascriptParserPlugin for AMDDefineDependencyParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "define" {
      self.process_call_define(parser, call_expr)
    } else {
      None
    }
  }

  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
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
