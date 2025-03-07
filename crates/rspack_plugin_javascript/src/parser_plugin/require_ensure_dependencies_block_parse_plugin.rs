use std::borrow::Cow;

use either::Either;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ChunkGroupOptions, ConstDependency, DependencyRange,
  GroupOptions, SharedSourceMap, SpanExt,
};
use swc_core::{
  common::Spanned,
  ecma::ast::{ArrowExpr, BlockStmtOrExpr, CallExpr, Expr, FnExpr, UnaryExpr},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{RequireEnsureDependency, RequireEnsureItemDependency},
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::{expr_matcher::is_require_ensure, JavascriptParser, Statement},
};

pub struct RequireEnsureDependenciesBlockParserPlugin;

impl JavascriptParserPlugin for RequireEnsureDependenciesBlockParserPlugin {
  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    (for_name == "require.ensure").then(|| {
      eval::evaluate_to_string(
        "function".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      )
    })
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == "require.ensure").then(|| {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span.real_hi(),
          "'function'".into(),
          None,
        )));
      true
    })
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, _for_name: &str) -> Option<bool> {
    if expr
      .callee
      .as_expr()
      .map_or(true, |expr| !is_require_ensure(&**expr))
    {
      return None;
    }

    let dependencies_arg = &expr.args.first()?.expr;
    let dependencies_expr = parser.evaluate_expression(dependencies_arg);
    let dependencies_items = if dependencies_expr.is_array() {
      Cow::Borrowed(dependencies_expr.items())
    } else {
      Cow::Owned(vec![dependencies_expr])
    };

    let success_arg = &expr.args.get(1)?.expr;
    let success_expr = success_arg.get_function_expr();
    let error_arg = expr.args.get(2);
    let error_expr = error_arg
      .as_ref()
      .and_then(|arg| arg.expr.get_function_expr());

    let chunk_name = match expr
      .args
      .get(3)
      .or(error_expr.as_ref().and(None)) // !errorExpression
      .or(expr.args.get(2))
    {
      Some(arg) => {
        let chunk_name_expr = parser.evaluate_expression(&arg.expr);
        match chunk_name_expr.as_string() {
          Some(chunk_name_expr) => Some(chunk_name_expr),
          None => return None,
        }
      }
      None => None,
    };

    if let Some(success_expr) = success_expr.as_ref().and_then(|expr| expr.expressions) {
      parser.walk_expression(success_expr);
    }
    if let Some(error_expr) = error_expr.as_ref().and_then(|expr| expr.expressions) {
      parser.walk_expression(error_expr);
    }

    let error_callback_exists =
      expr.args.len() == 4 || (expr.args.len() == 3 && chunk_name.is_none());
    let mut deps: Vec<BoxDependency> = vec![Box::new(RequireEnsureDependency::new(
      expr.span.into(),
      success_arg.span().into(),
      if error_callback_exists {
        error_arg.as_ref().map(|arg| arg.span().into())
      } else {
        None
      },
    ))];
    // TODO: Webpack sets `parser.state.current = depBlock`, but rspack doesn't support nested block yet.
    let mut failed = false;
    parser.in_function_scope(true, std::iter::empty(), |_| {
      for item in dependencies_items.iter() {
        if let Some(item) = item.as_string() {
          deps.push(Box::new(RequireEnsureItemDependency::new(
            item.as_str().into(),
            expr.span.into(),
          )));
        } else {
          failed = true;
        }
      }
    });
    if failed {
      return None;
    }
    let old_deps = std::mem::take(&mut parser.dependencies);

    if let Some(success_expr) = &success_expr {
      match success_expr.func {
        Either::Left(func) => {
          if let Some(body) = &func.function.body {
            parser.walk_statement(Statement::Block(body));
          }
        }
        Either::Right(arrow) => match &*arrow.body {
          BlockStmtOrExpr::BlockStmt(body) => parser.walk_statement(Statement::Block(body)),
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        },
      }
    }
    deps.extend(std::mem::replace(&mut parser.dependencies, old_deps));

    let source_map: SharedSourceMap = parser.source_map.clone();
    let mut block = AsyncDependenciesBlock::new(
      *parser.module_identifier,
      Into::<DependencyRange>::into(expr.span).to_loc(Some(&source_map)),
      None,
      deps,
      None,
    );
    block.set_group_options(GroupOptions::ChunkGroup(
      ChunkGroupOptions::default().name_optional(chunk_name),
    ));
    parser.blocks.push(Box::new(block));

    if success_expr.is_none() {
      parser.walk_expression(success_arg);
    }
    match error_expr {
      Some(error_expr) => match error_expr.func {
        Either::Left(func) => {
          if let Some(body) = &func.function.body {
            parser.walk_statement(Statement::Block(body));
          }
        }
        Either::Right(arrow) => match &*arrow.body {
          BlockStmtOrExpr::BlockStmt(body) => parser.walk_statement(Statement::Block(body)),
          BlockStmtOrExpr::Expr(expr) => parser.walk_expression(expr),
        },
      },
      None => {
        error_arg.inspect(|error_arg| parser.walk_expression(&error_arg.expr));
      }
    }

    Some(true)
  }
}

pub(crate) struct FunctionExpression<'a> {
  pub(crate) func: Either<&'a FnExpr, &'a ArrowExpr>,
  pub(crate) expressions: Option<&'a Expr>,
  // Used by AMD
  pub(crate) _need_this: Option<bool>,
}

pub(crate) trait GetFunctionExpression {
  fn get_function_expr(&self) -> Option<FunctionExpression>;
  fn inner_paren(&self) -> &Self;
}

impl GetFunctionExpression for Expr {
  fn get_function_expr(&self) -> Option<FunctionExpression> {
    match self.inner_paren() {
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
      Expr::Call(call_expr) if call_expr.args.len() == 1 => {
        let first_arg = &call_expr.args.first().expect("should exist").expr;
        let callee = &call_expr.callee;

        if let Some(callee_member_expr) = callee
          .as_expr()
          .and_then(|expr| expr.inner_paren().as_member())
          && let Some(fn_expr) = callee_member_expr.obj.inner_paren().as_fn_expr()
          && let Some(ident) = &callee_member_expr.prop.as_ident()
          && ident.sym == "bind"
        {
          return Some(FunctionExpression {
            func: Either::Left(fn_expr),
            expressions: Some(first_arg),
            _need_this: None,
          });
        }

        if let Some(callee_fn_expr) = callee
          .as_expr()
          .and_then(|expr| expr.inner_paren().as_fn_expr())
          && let Some(body_block_stmt) = &callee_fn_expr.function.body
          && first_arg.inner_paren().is_this()
          && body_block_stmt.stmts.len() == 1
          && let Some(return_stmt) = &body_block_stmt.stmts[0].as_return_stmt()
          && let Some(fn_expr) = return_stmt
            .arg
            .as_ref()
            .and_then(|expr| expr.inner_paren().as_fn_expr())
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

  fn inner_paren(&self) -> &Self {
    let mut cur = self;
    while let Some(inner) = cur.as_paren() {
      cur = &inner.expr;
    }
    cur
  }
}
