use std::sync::Arc;

use either::Either;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ChunkGroupOptions, ConstDependency, DependencyRange,
  GroupOptions,
};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  ArrowFunctionBodyData, ArrowFunctionExpression, Ast, CallExpression, Expr, ExprData, Function,
  GetSpan, PropertyKeyData, StmtData, UnaryExpression,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::{RequireEnsureDependency, RequireEnsureItemDependency},
  utils::eval::{self, BasicEvaluatedExpression},
  visitors::JavascriptParser,
};

pub struct RequireEnsureDependenciesBlockParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for RequireEnsureDependenciesBlockParserPlugin {
  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    (for_name == "require.ensure").then(|| {
      let span = expr.span(_parser.ast.ast);
      eval::evaluate_to_string("function".to_string(), span.real_lo(), span.real_hi())
    })
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == "require.ensure").then(|| {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(parser.ast.ast).into(),
        "'function'".into(),
      )));
      true
    })
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != "require.ensure" {
      return None;
    }

    let ast = parser.ast.ast;
    let arguments = expr.arguments(ast);
    let dependencies_arg = arguments.get_node(ast, 0)?.as_expr(ast)?;
    let dependencies_expr = parser.evaluate_expression(dependencies_arg);
    let dependencies_items = if dependencies_expr.is_array() {
      Either::Left(dependencies_expr.items().iter())
    } else {
      Either::Right(std::iter::once(&dependencies_expr))
    };

    let success_argument = arguments.get_node(ast, 1)?;
    let success_arg = success_argument.as_expr(ast)?;
    let success_expr = success_arg.get_function_expr(ast);
    let error_arg = arguments.get_node(ast, 2);
    let error_expr = error_arg
      .and_then(|arg| arg.as_expr(ast))
      .and_then(|expr| expr.get_function_expr(ast));

    let chunk_name = match expr
      .arguments(ast)
      .get_node(ast, 3)
      .or_else(|| if error_expr.is_some() { None } else { arguments.get_node(ast, 2) }) // !errorExpression
    {
      Some(arg) => match arg
        .as_expr(ast)
        .and_then(|expr| parser.evaluate_expression(expr).as_string())
      {
        Some(chunk_name) => Some(chunk_name),
        None => return None,
      },
      None => None,
    };

    if let Some(success_expr) = success_expr.and_then(|expr| expr.expressions) {
      parser.walk_expression(success_expr);
    }
    if let Some(error_expr) = error_expr.and_then(|expr| expr.expressions) {
      parser.walk_expression(error_expr);
    }

    let error_callback_exists =
      arguments.len() == 4 || (arguments.len() == 3 && chunk_name.is_none());
    let mut deps: Vec<BoxDependency> = vec![BoxDependency::new(RequireEnsureDependency::new(
      expr.span(ast).into(),
      success_arg.span(ast).into(),
      if error_callback_exists {
        error_arg.map(|arg| arg.span(ast).into())
      } else {
        None
      },
    ))];
    // TODO: Webpack sets `parser.state.current = depBlock`, but rspack doesn't support nested block yet.
    let mut failed = false;
    parser.in_function_scope(true, std::iter::empty(), |_| {
      for item in dependencies_items {
        if let Some(item) = item.as_string() {
          deps.push(BoxDependency::new(RequireEnsureItemDependency::new(
            item.as_str().into(),
            expr.span(ast).into(),
          )));
        } else {
          failed = true;
        }
      }
    });
    if failed {
      return None;
    }
    deps = parser.collect_dependencies_for_block(parser.next_block_idx(), deps, |parser| {
      if let Some(success_expr) = success_expr {
        let old_terminated = parser.terminated;
        match success_expr.func {
          Either::Left(func) => {
            let body = func.body(parser.ast.ast);
            parser.walk_function_body(body);
          }
          Either::Right(arrow) => match parser
            .ast
            .ast
            .arrow_function_body_data(arrow.body(parser.ast.ast))
          {
            ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
            ArrowFunctionBodyData::Expr(expr) => parser.walk_expression(expr),
          },
        }
        parser.terminated = old_terminated;
      }
    });

    let range = DependencyRange::from(expr.span(ast));
    let loc = parser.to_dependency_location(range);
    let mut block = AsyncDependenciesBlock::new(*parser.module_identifier, loc, None, deps, None);
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
          let body = func.body(parser.ast.ast);
          parser.walk_function_body(body);
        }
        Either::Right(arrow) => match parser
          .ast
          .ast
          .arrow_function_body_data(arrow.body(parser.ast.ast))
        {
          ArrowFunctionBodyData::FunctionBody(body) => parser.walk_function_body(body),
          ArrowFunctionBodyData::Expr(expr) => parser.walk_expression(expr),
        },
      },
      None => {
        error_arg.inspect(|error_arg| parser.walk_arguments(std::iter::once(*error_arg)));
      }
    }

    Some(true)
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionExpression {
  pub(crate) func: Either<Function, ArrowFunctionExpression>,
  pub(crate) expressions: Option<Expr>,
  // Used by AMD
  pub(crate) _need_this: Option<bool>,
}

pub(crate) trait GetFunctionExpression {
  fn get_function_expr(self, ast: &Ast<'_>) -> Option<FunctionExpression>;
}

impl GetFunctionExpression for Expr {
  fn get_function_expr(self, ast: &Ast<'_>) -> Option<FunctionExpression> {
    match ast.expr_data(self) {
      ExprData::Function(function) => Some(FunctionExpression {
        func: Either::Left(function),
        expressions: None,
        _need_this: Some(false),
      }),
      ExprData::ArrowFunctionExpression(arrow) => Some(FunctionExpression {
        func: Either::Right(arrow),
        expressions: None,
        _need_this: Some(false),
      }),
      ExprData::CallExpression(call) if call.arguments(ast).len() == 1 => {
        let first_arg = call.arguments(ast).get_node(ast, 0)?.as_expr(ast)?;
        let callee = call.callee(ast);

        if let Some(member) = callee.as_member_expression(ast)
          && let Some(function) = member.object(ast).as_function(ast)
          && let PropertyKeyData::IdentifierName(identifier) =
            ast.property_key_data(member.property(ast))
          && ast.get_utf8(identifier.name(ast)) == "bind"
        {
          return Some(FunctionExpression {
            func: Either::Left(function),
            expressions: Some(first_arg),
            _need_this: None,
          });
        }

        if let Some(callee_function) = callee.as_function(ast)
          && first_arg.is_this_expression(ast)
          && callee_function.body(ast).body(ast).len() == 1
          && let Some(statement) = callee_function.body(ast).body(ast).get_node(ast, 0)
          && let StmtData::ReturnStatement(return_statement) = ast.stmt_data(statement)
          && let Some(function) = return_statement
            .argument(ast)
            .and_then(|expr| expr.as_function(ast))
        {
          return Some(FunctionExpression {
            func: Either::Left(function),
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
