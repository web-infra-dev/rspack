use std::borrow::Cow;

use either::Either;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ChunkGroupOptions, ConstDependency, DependencyRange,
  GroupOptions,
};
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{
  ArrowExpr, Ast, BlockStmtOrExpr, CallExpr, Expr, FnExpr, GetSpan, UnaryExpr,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{RequireEnsureDependency, RequireEnsureItemDependency},
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{JavascriptParser, Statement},
};

pub struct RequireEnsureDependenciesBlockParserPlugin;

impl JavascriptParserPlugin for RequireEnsureDependenciesBlockParserPlugin {
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    (for_name == "require.ensure").then(|| {
      eval::evaluate_to_string(
        "function".to_string(),
        expr.span(&parser.ast).real_lo(),
        expr.span(&parser.ast).real_hi(),
      )
    })
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == "require.ensure").then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span(&parser.ast).into(),
        "'function'".into(),
      )));
      true
    })
  }

  fn call(&self, parser: &mut JavascriptParser, expr: CallExpr, for_name: &str) -> Option<bool> {
    if for_name != "require.ensure" {
      return None;
    }

    let dependencies_arg = parser
      .ast
      .get_node_in_sub_range(expr.args(&parser.ast).first()?)
      .expr(&parser.ast);
    let dependencies_expr = parser.evaluate_expression(dependencies_arg);
    let dependencies_items = if dependencies_expr.is_array() {
      Cow::Borrowed(dependencies_expr.items())
    } else {
      Cow::Owned(vec![dependencies_expr])
    };

    let success_arg = parser
      .ast
      .get_node_in_sub_range(expr.args(&parser.ast).get(1)?)
      .expr(&parser.ast);
    let success_expr = success_arg.get_function_expr(&parser.ast);
    let error_arg = expr
      .args(&parser.ast)
      .get(2)
      .map(|arg| parser.ast.get_node_in_sub_range(arg));
    let error_expr = error_arg.and_then(|arg| arg.expr(&parser.ast).get_function_expr(&parser.ast));

    let chunk_name = match expr
      .args(&parser.ast)
      .get(3)
      .or_else(|| if error_expr.is_some() { None } else { expr.args(&parser.ast).get(2) }) // !errorExpression
    {
      Some(arg) => match parser.evaluate_expression(parser.ast.get_node_in_sub_range(arg).expr(&parser.ast)).as_string() {
        Some(chunk_name) => Some(chunk_name),
        None => return None,
      },
      None => None,
    };

    if let Some(success_expr) = success_expr.as_ref().and_then(|expr| expr.expressions) {
      parser.walk_expression(success_expr);
    }
    if let Some(error_expr) = error_expr.as_ref().and_then(|expr| expr.expressions) {
      parser.walk_expression(error_expr);
    }

    let error_callback_exists = expr.args(&parser.ast).len() == 4
      || (expr.args(&parser.ast).len() == 3 && chunk_name.is_none());
    let mut deps: Vec<BoxDependency> = vec![Box::new(RequireEnsureDependency::new(
      expr.span(&parser.ast).into(),
      success_arg.span(&parser.ast).into(),
      if error_callback_exists {
        error_arg.as_ref().map(|arg| arg.span(&parser.ast).into())
      } else {
        None
      },
    ))];
    // TODO: Webpack sets `parser.state.current = depBlock`, but rspack doesn't support nested block yet.
    let mut failed = false;
    parser.in_function_scope(true, std::iter::empty(), |parser| {
      for item in dependencies_items.iter() {
        if let Some(item) = item.as_string() {
          deps.push(Box::new(RequireEnsureItemDependency::new(
            item.as_str().into(),
            expr.span(&parser.ast).into(),
          )));
        } else {
          failed = true;
        }
      }
    });
    if failed {
      return None;
    }
    deps.extend(parser.collect_dependencies_for_block(|parser| {
      if let Some(success_expr) = &success_expr {
        match success_expr.func {
          Either::Left(func) => {
            if let Some(body) = func.function(&parser.ast).body(&parser.ast) {
              parser.walk_statement(Statement::Block(body));
            }
          }
          Either::Right(arrow) => match arrow.body(&parser.ast) {
            BlockStmtOrExpr::BlockStmt(body) => parser.walk_statement(Statement::Block(body)),
            BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
          },
        }
      }
    }));

    let mut block = AsyncDependenciesBlock::new(
      *parser.module_identifier,
      Into::<DependencyRange>::into(expr.span(&parser.ast)).to_loc(Some(parser.source())),
      None,
      deps,
      None,
    );
    block.set_group_options(GroupOptions::ChunkGroup(
      ChunkGroupOptions::default().name_optional(chunk_name),
    ));
    parser.add_block(Box::new(block));

    if success_expr.is_none() {
      parser.walk_expression(success_arg);
    }
    match error_expr {
      Some(error_expr) => match error_expr.func {
        Either::Left(func) => {
          if let Some(body) = func.function(&parser.ast).body(&parser.ast) {
            parser.walk_statement(Statement::Block(body));
          }
        }
        Either::Right(arrow) => match arrow.body(&parser.ast) {
          BlockStmtOrExpr::BlockStmt(body) => parser.walk_statement(Statement::Block(body)),
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        },
      },
      None => {
        error_arg.inspect(|error_arg| parser.walk_expression(error_arg.expr(&parser.ast)));
      }
    }

    Some(true)
  }
}

pub(crate) struct FunctionExpression {
  pub(crate) func: Either<FnExpr, ArrowExpr>,
  pub(crate) expressions: Option<Expr>,
  // Used by AMD
  pub(crate) _need_this: Option<bool>,
}

pub(crate) trait GetFunctionExpression {
  fn get_function_expr(self, ast: &Ast) -> Option<FunctionExpression>;
}

impl GetFunctionExpression for Expr {
  fn get_function_expr(self, ast: &Ast) -> Option<FunctionExpression> {
    match self {
      Expr::Fn(fn_expr) => Some(FunctionExpression {
        func: Either::Left(fn_expr),
        expressions: None,
        _need_this: Some(false),
      }),
      Expr::Arrow(arrow_expr) => Some(FunctionExpression {
        func: Either::Right(arrow_expr),
        expressions: None,
        _need_this: Some(false),
      }),
      Expr::Call(call_expr) if call_expr.args(ast).len() == 1 => {
        let first_arg = ast
          .get_node_in_sub_range(call_expr.args(ast).first().expect("should exist"))
          .expr(ast);
        let callee = call_expr.callee(ast);

        if let Some(callee_member_expr) = callee.as_expr().and_then(|expr| expr.as_member())
          && let Some(fn_expr) = callee_member_expr.obj(ast).as_fn()
          && let Some(ident) = callee_member_expr.prop(ast).as_ident()
          && ast.get_utf8(ident.sym(ast)) == "bind"
        {
          return Some(FunctionExpression {
            func: Either::Left(fn_expr),
            expressions: Some(first_arg),
            _need_this: None,
          });
        }

        if let Some(callee_fn_expr) = callee.as_expr().and_then(|expr| expr.as_fn())
          && let Some(body_block_stmt) = callee_fn_expr.function(ast).body(ast)
          && first_arg.is_this()
          && body_block_stmt.stmts(ast).len() == 1
          && let Some(return_stmt) = ast
            .get_node_in_sub_range(body_block_stmt.stmts(ast).get(0).unwrap())
            .as_return()
          && let Some(fn_expr) = return_stmt.arg(ast).and_then(|expr| expr.as_fn())
        {
          return Some(FunctionExpression {
            func: Either::Left(fn_expr),
            expressions: None,
            _need_this: Some(true),
          });
        }

        None
      }
      _ => None,
    }
  }
}
