use std::borrow::Cow;

use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ConstDependency, ContextDependency, ContextMode,
  ContextNameSpaceObject, ContextOptions, Dependency, DependencyCategory, RuntimeGlobals, SpanExt,
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

use crate::{
  dependency::{
    amd_define_dependency::AMDDefineDependency,
    amd_require_array_dependency::{AMDRequireArrayDependency, AMDRequireArrayItem},
    amd_require_item_dependency::AMDRequireItemDependency,
    local_module_dependency::LocalModuleDependency,
    AMDRequireContextDependency,
  },
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    context_reg_exp, create_context_dependency, scope_info::FreeName, JavascriptParser, Statement,
  },
  JavascriptParserPlugin,
};

pub struct AMDDefineDependencyParserPlugin;

fn is_unbound_function_expression(expr: &Expr) -> bool {
  expr.is_fn_expr() || expr.is_arrow()
}

fn is_bound_function_expression(expr: &Expr) -> bool {
  if !expr.is_call() {
    return false;
  }

  let call_expr = expr.as_call().expect("expr is supposed to be CallExpr");
  match &call_expr.callee {
    Callee::Super(_) => return false,
    Callee::Import(_) => return false,
    Callee::Expr(callee) => {
      if !callee.is_member() {
        return false;
      }
      let callee_member = callee
        .as_member()
        .expect("callee is supposed to be MemberExpr");
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

  true
}

fn is_callable(expr: &Expr) -> bool {
  is_unbound_function_expression(expr) || is_bound_function_expression(expr)
}

/// define('ui/foo/bar', ['./baz', '../qux'], ...);
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
    .map(|ident| ident.sym.clone())
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
    } else if param.is_const_array() {
      let mut deps: Vec<AMDRequireArrayItem> = vec![];
      let array = param.array();
      for (i, request) in array.iter().enumerate() {
        if request == "require" {
          identifiers.insert(i, REQUIRE);
          deps.push(AMDRequireArrayItem::String(
            RuntimeGlobals::REQUIRE.name().into(),
          ));
        } else if request == "exports" {
          identifiers.insert(i, EXPORTS);
          deps.push(AMDRequireArrayItem::String(request.into()));
        } else if request == "module" {
          identifiers.insert(i, MODULE);
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
          parser.dependencies.push(Box::new(dep));
        }
      }
      let range = param.range();
      let dep = AMDRequireArrayDependency::new(deps, (range.0, range.1 - 1));
      parser.presentational_dependencies.push(Box::new(dep));
      return Some(true);
    }
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
      } else if let Some(local_module) = parser.get_local_module_mut(
        &named_module
          .as_ref()
          .map(|parent| lookup(parent, param_str))
          .unwrap_or(param_str.into()),
      ) {
        local_module.flag_used();
        let dep = Box::new(LocalModuleDependency::new(
          local_module.clone(),
          Some((range.0, range.1)),
          false,
        ));
        parser.presentational_dependencies.push(dep);
        return Some(true);
      } else {
        let mut dep = Box::new(AMDRequireItemDependency::new(
          Atom::new(param_str.as_str()),
          Some(range),
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
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let call_span = call_expr.span();
    let param_range = (param.range().0, param.range().1 - 1);

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
    parser.dependencies.push(Box::new(dep));
    Some(true)
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
          Expr::Arrow(array_func) => Some(array_func.params.iter().map(Cow::Borrowed).collect()),
          _ => None,
        };
      } else if is_bound_function_expression(func) {
        let call_expr = func
          .as_call()
          .expect("call_expr is supposed to be a CallExpr");
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

        if !call_expr.args.is_empty() {
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
            fn_renames.insert(get_ident_name(param), name);
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
          fn_renames.insert(get_ident_name(param), RESERVED_NAMES[idx]);
          return false;
        }
        true
      });
    }

    if func.is_some_and(is_unbound_function_expression) {
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
    } else if func.is_some_and(is_bound_function_expression) {
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
    } else if let Some(expr) = func {
      parser.walk_expression(expr);
    } else if let Some(expr) = obj {
      parser.walk_expression(expr);
    }

    let local_module = named_module
      .as_ref()
      .map(|name| parser.add_local_module(name.as_str()));

    let dep = Box::new(AMDDefineDependency::new(
      (call_expr.span.real_lo(), call_expr.span.real_hi()),
      array.map(|expr| span_to_range(expr.span())),
      func.map(|expr| span_to_range(expr.span())),
      obj.map(|expr| span_to_range(expr.span())),
      named_module,
      local_module,
    ));

    parser.presentational_dependencies.push(dep);

    Some(true)
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

  /**
   * unlike js, it's hard to share the LocalModule instance in Rust.
   * so the AMDDefineDependency will get a clone of LocalModule in parser.local_modules.
   * synchronize the used flag to the AMDDefineDependency's local_module at the end of the parse.
   */
  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    for dep in parser.presentational_dependencies.iter_mut() {
      if let Some(define_dep) = dep.as_any_mut().downcast_mut::<AMDDefineDependency>()
        && let Some(local_module) = define_dep.get_local_module_mut()
        && parser
          .local_modules
          .get(local_module.get_idx())
          .is_some_and(|m| m.is_used())
      {
        local_module.flag_used();
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
