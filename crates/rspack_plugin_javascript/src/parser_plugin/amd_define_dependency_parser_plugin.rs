use std::borrow::Cow;

use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ConstDependency, RuntimeGlobals, SpanExt,
};
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap;
use swc_core::{
  common::{Span, Spanned},
  ecma::{
    ast::{BlockStmtOrExpr, CallExpr, Callee, Expr, Lit, Pat},
    utils::ExprExt,
  },
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{
    amd_define_dependency::AmdDefineDependency,
    amd_require_item_dependency::AMDRequireItemDependency,
    local_module_dependency::LocalModuleDependency,
  },
  utils::eval::BasicEvaluatedExpression,
  visitors::{scope_info::FreeName, JavascriptParser, Statement},
};

pub struct AMDDefineDependencyParserPlugin;

fn is_unbound_function_expression(expr: &Expr) -> bool {
  expr.is_fn_expr() || expr.is_arrow()
}

fn is_bound_function_expression(expr: &Expr) -> bool {
  if !expr.is_call() {
    return false;
  }

  let call_expr = expr.as_call().unwrap();
  match &call_expr.callee {
    Callee::Super(_) => return false,
    Callee::Import(_) => return false,
    Callee::Expr(callee) => {
      if !callee.is_member() {
        return false;
      }
      let callee_member = callee.as_member().unwrap();
      if callee_member.prop.is_computed() {
        return false;
      }
      if !callee_member.obj.is_fn_expr() {
        return false;
      }
      if !callee_member.prop.is_ident_with("bind") {
        return false;
      }
    }
  }

  return true;
}

fn is_callable(expr: &Expr) -> bool {
  is_unbound_function_expression(expr) || is_bound_function_expression(expr)
}

/**
 * lookup
 *
 * define('ui/foo/bar', ['./baz', '../qux'], ...);
 * - 'ui/foo/baz'
 * - 'ui/qux'
 */
fn resolve_mod_name(mod_name: &Option<Atom>, dep_name: &str) -> Atom {
  if let Some(mod_name) = mod_name
    && dep_name.starts_with('.')
  {
    let mut path: Vec<&str> = mod_name.split('/').collect();
    path.pop();

    for seg in dep_name.split('.') {
      if seg == ".." {
        path.pop();
      } else if seg != "." {
        path.push(seg);
      }
    }

    path.join("/").into()
  } else {
    dep_name.into()
  }
}

const REQUIRE: &str = "require";
const MODULE: &str = "module";
const EXPORTS: &str = "exports";
const RESERVED_NAMES: [&str; 3] = [REQUIRE, MODULE, EXPORTS];

fn span_to_range(span: Span) -> (u32, u32) {
  (span.real_lo(), span.real_hi())
}

fn get_lit_str(expr: &Expr) -> Option<Atom> {
  expr.as_lit().and_then(|lit| match lit {
    Lit::Str(s) => Some(s.value.clone()),
    _ => None,
  })
}

fn get_ident_name(pat: &Pat) -> Atom {
  pat
    .as_ident()
    .and_then(|ident| Some(ident.sym.clone()))
    .unwrap_or("".into())
}

impl AMDDefineDependencyParserPlugin {
  fn process_array(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
    identifiers: &mut FxHashMap<usize, &'static str>, // param index => "require" | "module" | "exports"
    named_module: &Option<Atom>,
  ) -> Option<bool> {
    if param.is_array() {
      let items = param.items();
      for (idx, item) in items.iter().enumerate() {
        if item.is_string() {
          let item = item.string();
          if let Some(i) = RESERVED_NAMES.iter().position(|s| s == item) {
            identifiers.insert(idx, RESERVED_NAMES[i]);
          }
        }
        let result = self.process_item(parser, call_expr, item, named_module);
        if result.is_none() {
          self.process_context(parser, call_expr, item);
        }
      }
      return Some(true);
    }
    // currently, there is no ConstArray in rspack
    // TODO: check if `param` is a const string array
    None
  }

  fn process_item(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
    named_module: &Option<Atom>,
  ) -> Option<bool> {
    if param.is_conditional() {
      let options = param.options();

      for option in options.iter() {
        let result = self.process_item(parser, call_expr, option, &None);
        if result.is_none() {
          self.process_context(parser, call_expr, param);
        }
      }

      return Some(true);
    } else if param.is_string() {
      let param_str = param.string();
      let range = {
        let (l, h) = param.range();
        (l, h - 1)
      };

      let dep = if param_str == "require" {
        Box::new(ConstDependency::new(
          range.0,
          range.1,
          RuntimeGlobals::REQUIRE.name().into(),
          Some(RuntimeGlobals::REQUIRE),
        ))
      } else if param_str == "exports" {
        Box::new(ConstDependency::new(
          range.0,
          range.1,
          EXPORTS.into(),
          Some(RuntimeGlobals::EXPORTS),
        ))
      } else if param_str == "module" {
        Box::new(ConstDependency::new(
          range.0,
          range.1,
          MODULE.into(),
          Some(RuntimeGlobals::MODULE),
        ))
      } else if let Some(local_module) =
        parser.get_local_module(&resolve_mod_name(named_module, param_str))
      {
        let dep = Box::new(LocalModuleDependency::new(
          local_module,
          Some((range.0, range.1)),
          false,
        ));
        parser.presentational_dependencies.push(dep);
        return Some(true);
      } else {
        let mut dep = Box::new(AMDRequireItemDependency::new(
          Atom::new(param_str.as_str()),
          range,
        ));
        dep.set_optional(parser.in_try);
        parser.dependencies.push(dep);
        return Some(true);
      };
      // TODO: how to implement this?
      // dep.loc = /** @type {DependencyLocation} */ (expr.loc);
      parser.presentational_dependencies.push(dep);
      return Some(true);
    }
    None
  }

  fn process_context(
    &self,
    _parser: &mut JavascriptParser,
    _call_expr: &CallExpr,
    _param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    // TODO: support amd context dep
    None
  }

  fn process_call_define(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    let mut array: Option<&Expr> = None;
    let mut func: Option<&Expr> = None;
    let mut obj: Option<&Expr> = None;
    let mut named_module: Option<Atom> = None;

    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];

        // We don't support spread syntax in `define()`
        if first_arg.spread.is_some() {
          return None;
        }

        if is_callable(&first_arg.expr) {
          // define(f() {…})
          func = Some(&first_arg.expr);
        } else if first_arg.expr.is_object() {
          // define({…})
          obj = Some(&first_arg.expr);
        } else {
          // define(expr)
          // unclear if function or object
          func = Some(&first_arg.expr);
          obj = Some(&first_arg.expr);
        }
      }
      2 => {
        let first_arg = &call_expr.args[0];
        let second_arg = &call_expr.args[1];

        // We don't support spread syntax in `define()`
        if first_arg.spread.is_some() || second_arg.spread.is_some() {
          return None;
        }

        if first_arg.expr.is_lit() {
          // define("…", …)
          named_module = get_lit_str(&first_arg.expr);

          if is_callable(&second_arg.expr) {
            // define("…", f() {…})
            func = Some(&second_arg.expr);
          } else if second_arg.expr.is_object() {
            // define("…", {…})
            obj = Some(&second_arg.expr);
          } else {
            // define("…", expr)
            // unclear if function or object
            func = Some(&second_arg.expr);
            obj = Some(&second_arg.expr);
          }
        } else {
          // define([…], …)
          if !first_arg.expr.is_array() {
            return None;
          }

          array = Some(&first_arg.expr);

          if is_callable(&second_arg.expr) {
            // define([…], f() {})
            func = Some(&second_arg.expr);
          } else if second_arg.expr.is_object() {
            // define([…], {…})
            obj = Some(&second_arg.expr);
          } else {
            // define([…], expr)
            // unclear if function or object
            func = Some(&second_arg.expr);
            obj = Some(&second_arg.expr);
          }
        }
      }
      3 => {
        // define("…", […], …)

        let first_arg = &call_expr.args[0];
        let second_arg = &call_expr.args[1];
        let third_arg = &call_expr.args[2];

        // We don't support spread syntax in `define()`
        if first_arg.spread.is_some() || second_arg.spread.is_some() || third_arg.spread.is_some() {
          return None;
        }

        if !first_arg.expr.is_lit() {
          return None;
        }
        if !second_arg.expr.is_array_lit() {
          return None;
        }

        named_module = get_lit_str(&first_arg.expr);
        array = Some(&second_arg.expr);

        if is_callable(&third_arg.expr) {
          // define("…", […], f() {})
          func = Some(&third_arg.expr);
        } else if third_arg.expr.is_object() {
          // define("…", […], {…})
          obj = Some(&third_arg.expr);
        } else {
          // define("…", […], expr)
          // unclear if function or object
          func = Some(&third_arg.expr);
          obj = Some(&third_arg.expr);
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

    let mut fn_params: Option<Vec<Cow<'_, Pat>>> = None;
    let mut fn_params_offset = 0usize;
    if let Some(func) = func {
      if is_unbound_function_expression(func) {
        fn_params = match func {
          Expr::Fn(normal_func) => Some(
            normal_func
              .function
              .params
              .iter()
              .map(|param| Cow::Borrowed(&param.pat))
              .collect(),
          ),
          Expr::Arrow(array_func) => Some(
            array_func
              .params
              .iter()
              .map(|param| Cow::Borrowed(param))
              .collect(),
          ),
          _ => None,
        };
      } else if is_bound_function_expression(func) {
        let call_expr = func.as_call().unwrap();
        let object = &call_expr
          .callee
          .as_expr()
          .expect("call_expr.callee is supposed to be Expr")
          .as_member()
          .expect("call_expr.callee is supposed to be MemberExpr")
          .obj
          .as_fn_expr()
          .expect("call_expr.callee.obj is supposed to be FnExpr");

        fn_params = Some(
          object
            .function
            .params
            .iter()
            .map(|param| Cow::Borrowed(&param.pat))
            .collect(),
        );

        if call_expr.args.len() > 0 {
          fn_params_offset = call_expr.args.len() - 1;
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
          if let Some(&name) = identifiers.get(&idx) {
            fn_renames.insert(get_ident_name(&param), name);
            return false;
          }
          return true;
        });
      }
    } else {
      if let Some(fn_params) = &mut fn_params {
        let mut i = 0usize;
        fn_params.retain(|param| {
          if i < fn_params_offset {
            return false;
          }
          let idx = i - fn_params_offset;
          i += 1;
          if idx < RESERVED_NAMES.len() {
            fn_renames.insert(get_ident_name(&param), RESERVED_NAMES[idx]);
            return false;
          }
          return true;
        });
      }
    }

    if func.is_some_and(|f| is_unbound_function_expression(f)) {
      let in_try = parser.in_try;
      parser.in_function_scope(
        true,
        fn_params.expect("fn_params should not be None").into_iter(),
        |parser| {
          for (name, &rename_identifier) in fn_renames.iter() {
            let variable = parser
              .get_variable_info(rename_identifier)
              .and_then(|info| info.free_name.as_ref())
              .and_then(|free_name| match free_name {
                FreeName::String(s) => Some(s.to_string()),
                FreeName::True => None,
              })
              .unwrap_or(rename_identifier.to_string());
            parser.set_variable(name.to_string(), variable);
          }

          parser.in_try = in_try;

          if let Some(func) = func.and_then(|f| f.as_fn_expr()) {
            if let Some(body) = &func.function.body {
              parser.detect_mode(&body.stmts);
              let prev = parser.prev_statement;
              parser.pre_walk_statement(Statement::Block(body));
              parser.prev_statement = prev;
              parser.walk_statement(Statement::Block(body));
            }
          } else if let Some(func) = func.and_then(|f| f.as_arrow()) {
            match &*func.body {
              BlockStmtOrExpr::BlockStmt(stmt) => {
                parser.detect_mode(&stmt.stmts);
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
    } else if func.is_some_and(|f| is_bound_function_expression(f)) {
      let in_try = parser.in_try;

      if let Some(call_expr) = func.and_then(|f| f.as_call()) {
        let object = call_expr
          .callee
          .as_expr()
          .and_then(|expr| expr.as_member())
          .and_then(|member_expr| member_expr.obj.as_fn_expr());

        if let Some(func_expr) = object {
          parser.in_function_scope(
            true,
            func_expr
              .function
              .params
              .iter()
              .map(|param| Cow::Borrowed(&param.pat))
              .filter(|pat| {
                pat
                  .as_ident()
                  .is_some_and(|ident| !RESERVED_NAMES.contains(&ident.sym.as_str()))
              }),
            |parser| {
              for (name, &rename_identifier) in fn_renames.iter() {
                let variable = parser
                  .get_variable_info(rename_identifier)
                  .and_then(|info| info.free_name.as_ref())
                  .and_then(|free_name| match free_name {
                    FreeName::String(s) => Some(s.to_string()),
                    FreeName::True => None,
                  })
                  .unwrap_or(rename_identifier.to_string());
                parser.set_variable(name.to_string(), variable);
              }

              parser.in_try = in_try;

              if let Some(body) = &func_expr.function.body {
                parser.detect_mode(&body.stmts);
                let prev = parser.prev_statement;
                parser.pre_walk_statement(Statement::Block(body));
                parser.prev_statement = prev;
                parser.walk_statement(Statement::Block(body));
              }
            },
          );
        }

        parser.walk_expr_or_spread(&call_expr.args);
      }
    } else {
      if let Some(expr) = func {
        parser.walk_expression(expr);
      } else if let Some(expr) = obj {
        parser.walk_expression(expr);
      }
    }

    let local_module = named_module
      .as_ref()
      .and_then(|name| Some(parser.add_local_module(name.as_str())));

    let dep = Box::new(AmdDefineDependency::new(
      (call_expr.span.real_lo(), call_expr.span.real_hi()),
      array.and_then(|expr| Some(span_to_range(expr.span()))),
      func.and_then(|expr| Some(span_to_range(expr.span()))),
      obj.and_then(|expr| Some(span_to_range(expr.span()))),
      named_module,
      local_module,
    ));

    parser.presentational_dependencies.push(dep);

    return Some(true);
  }
}

impl JavascriptParserPlugin for AMDDefineDependencyParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "define" {
      self.process_call_define(parser, call_expr)
    } else {
      None
    }
  }
}
