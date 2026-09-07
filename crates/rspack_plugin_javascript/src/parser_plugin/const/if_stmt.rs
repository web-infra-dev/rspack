use std::sync::Arc;

use itertools::Itertools;
use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
  Ast, BindingPattern, BindingPatternData, GetSpan, IfStatement, VariableKind,
};

use crate::visitors::{JavascriptParser, Statement, VariableDeclarationKind};

fn collect_declaration_from_pattern<'a>(
  ast: &'a Ast<'_>,
  pattern: BindingPattern,
  declarations: &mut FxHashSet<&'a str>,
) {
  let mut stack = vec![pattern];
  while let Some(pattern) = stack.pop() {
    match ast.binding_pattern_data(pattern) {
      BindingPatternData::BindingIdentifier(identifier) => {
        declarations.insert(ast.get_utf8(identifier.name(ast)));
      }
      BindingPatternData::AssignmentPattern(assignment) => stack.push(assignment.left(ast)),
      BindingPatternData::BindingRestElement(rest) => stack.push(rest.argument(ast)),
      BindingPatternData::ArrayPattern(array) => {
        stack.extend(
          array
            .elements(ast)
            .iter()
            .filter_map(|id| ast.get_node_in_sub_range(id)),
        );
        if let Some(rest) = array.rest(ast) {
          stack.push(rest.argument(ast));
        }
      }
      BindingPatternData::ObjectPattern(object) => {
        stack.extend(
          object
            .properties(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .map(|property| property.value(ast)),
        );
        if let Some(rest) = object.rest(ast) {
          stack.push(rest.argument(ast));
        }
      }
      BindingPatternData::SimpleAssignmentTarget(_) => {}
    }
  }
}

fn collect_variable_declaration<'a>(
  ast: &'a Ast<'_>,
  declaration: swc_next_ecma_ast::VariableDeclaration,
  declarations: &mut FxHashSet<&'a str>,
) {
  if declaration.kind(ast) == VariableKind::Var {
    for declarator in declaration
      .declarators(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
    {
      collect_declaration_from_pattern(ast, declarator.id(ast), declarations);
    }
  }
}

/// Collects hoisted `var` and (in non-strict mode) function declaration identifiers.
pub fn get_hoisted_declarations<'a>(
  ast: &'a Ast<'_>,
  stmt: Statement,
  include_function_declarations: bool,
) -> FxHashSet<&'a str> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![stmt];

  while let Some(statement) = stmt_stack.pop() {
    match statement {
      Statement::Block(block) => {
        stmt_stack.extend(
          block
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .map(|statement| Statement::from_stmt(ast, statement)),
        );
      }
      Statement::If(statement) => {
        stmt_stack.push(Statement::from_stmt(ast, statement.consequent(ast)));
        if let Some(alternate) = statement.alternate(ast) {
          stmt_stack.push(Statement::from_stmt(ast, alternate));
        }
      }
      Statement::For(statement) => {
        if let Some(declaration) = statement
          .init(ast)
          .and_then(|init| init.as_variable_declaration(ast))
        {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::ForIn(statement) => {
        if let Some(declaration) = statement.left(ast).as_variable_declaration(ast) {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::ForOf(statement) => {
        if let Some(declaration) = statement.left(ast).as_variable_declaration(ast) {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::DoWhile(statement) => {
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::While(statement) => {
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::Labeled(statement) => {
        stmt_stack.push(Statement::from_stmt(ast, statement.body(ast)));
      }
      Statement::Switch(statement) => {
        for case in statement
          .cases(ast)
          .iter()
          .map(|id| ast.get_node_in_sub_range(id))
        {
          stmt_stack.extend(
            case
              .consequent(ast)
              .iter()
              .map(|id| ast.get_node_in_sub_range(id))
              .map(|statement| Statement::from_stmt(ast, statement)),
          );
        }
      }
      Statement::Try(statement) => {
        stmt_stack.extend(
          statement
            .block(ast)
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .map(|statement| Statement::from_stmt(ast, statement)),
        );
        if let Some(handler) = statement.handler(ast) {
          stmt_stack.extend(
            handler
              .body(ast)
              .body(ast)
              .iter()
              .map(|id| ast.get_node_in_sub_range(id))
              .map(|statement| Statement::from_stmt(ast, statement)),
          );
        }
        if let Some(finalizer) = statement.finalizer(ast) {
          stmt_stack.extend(
            finalizer
              .body(ast)
              .iter()
              .map(|id| ast.get_node_in_sub_range(id))
              .map(|statement| Statement::from_stmt(ast, statement)),
          );
        }
      }
      Statement::Fn(function) if include_function_declarations => {
        if let Some(identifier) = function.ident(ast) {
          declarations.insert(ast.get_utf8(identifier.name(ast)));
        }
      }
      Statement::Var(declaration) if declaration.kind(ast) == VariableDeclarationKind::Var => {
        for declarator in declaration.declarators(ast) {
          collect_declaration_from_pattern(ast, declarator.id(ast), &mut declarations);
        }
      }
      _ => {}
    }
  }

  declarations
}

pub fn statement_if<'p>(
  scanner: &mut JavascriptParser<'p>,
  statement: IfStatement,
) -> Option<bool> {
  let ast = scanner.ast.ast;
  let param = scanner.evaluate_expression(statement.test(ast));
  let boolean = param.as_bool()?;
  if !param.could_have_side_effects() {
    scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
      param.range().into(),
      boolean.to_string().into_boxed_str(),
    )));
  } else {
    scanner.walk_expression(statement.test(ast));
  }

  let branch_to_remove = if boolean {
    statement.alternate(ast)
  } else {
    Some(statement.consequent(ast))
  };

  if let Some(branch_to_remove) = branch_to_remove {
    let branch_statement = Statement::from_stmt(ast, branch_to_remove);
    let declarations = get_hoisted_declarations(ast, branch_statement, !scanner.is_strict());
    let replacement = if declarations.is_empty() {
      "{}".to_string()
    } else {
      format!("{{ var {} }}", declarations.iter().join(", "))
    };

    let span = branch_to_remove.span(ast);
    scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
      (span.real_lo(), span.real_hi()).into(),
      replacement.into_boxed_str(),
    )))
  }
  Some(boolean)
}
