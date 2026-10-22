use std::sync::Arc;

use itertools::Itertools;
use rspack_core::ConstDependency;
use rspack_util::{SpanExt, swc::AstSubRangeExt};
use rustc_hash::FxHashSet;
use swc_next_ecma_ast::{
  Ast, BindingPattern, BindingPatternData, GetSpan, IfStatement, NodeKindData, Stmt, VariableKind,
};

use crate::visitors::JavascriptParser;

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
        stack.extend(ast.nodes(array.elements(ast)).flatten());
        if let Some(rest) = array.rest(ast) {
          stack.push(rest.argument(ast));
        }
      }
      BindingPatternData::ObjectPattern(object) => {
        stack.extend(
          ast
            .nodes(object.properties(ast))
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
    for declarator in ast.nodes(declaration.declarators(ast)) {
      collect_declaration_from_pattern(ast, declarator.id(ast), declarations);
    }
  }
}

/// Collects hoisted `var` and (in non-strict mode) function declaration identifiers.
pub fn get_hoisted_declarations<'a>(
  ast: &'a Ast<'_>,
  stmt: Stmt,
  include_function_declarations: bool,
) -> FxHashSet<&'a str> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![stmt];

  while let Some(statement) = stmt_stack.pop() {
    match ast.kind_data(statement.node_id()) {
      NodeKindData::BlockStatement(block) => {
        stmt_stack.extend(ast.nodes(block.body(ast)));
      }
      NodeKindData::IfStatement(statement) => {
        stmt_stack.push(statement.consequent(ast));
        if let Some(alternate) = statement.alternate(ast) {
          stmt_stack.push(alternate);
        }
      }
      NodeKindData::ForStatement(statement) => {
        if let Some(declaration) = statement
          .init(ast)
          .and_then(|init| init.as_variable_declaration(ast))
        {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::ForInStatement(statement) => {
        if let Some(declaration) = statement.left(ast).as_variable_declaration(ast) {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::ForOfStatement(statement) => {
        if let Some(declaration) = statement.left(ast).as_variable_declaration(ast) {
          collect_variable_declaration(ast, declaration, &mut declarations);
        }
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::DoWhileStatement(statement) => {
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::WhileStatement(statement) => {
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::LabeledStatement(statement) => {
        stmt_stack.push(statement.body(ast));
      }
      NodeKindData::SwitchStatement(statement) => {
        for case in ast.nodes(statement.cases(ast)) {
          stmt_stack.extend(ast.nodes(case.consequent(ast)));
        }
      }
      NodeKindData::TryStatement(statement) => {
        stmt_stack.extend(ast.nodes(statement.block(ast).body(ast)));
        if let Some(handler) = statement.handler(ast) {
          stmt_stack.extend(ast.nodes(handler.body(ast).body(ast)));
        }
        if let Some(finalizer) = statement.finalizer(ast) {
          stmt_stack.extend(ast.nodes(finalizer.body(ast)));
        }
      }
      NodeKindData::Function(function) if include_function_declarations => {
        if let Some(identifier) = function.id(ast) {
          declarations.insert(ast.get_utf8(identifier.name(ast)));
        }
      }
      NodeKindData::VariableDeclaration(declaration)
        if declaration.kind(ast) == VariableKind::Var =>
      {
        for declarator in ast.nodes(declaration.declarators(ast)) {
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
    let declarations = get_hoisted_declarations(ast, branch_to_remove, !scanner.is_strict());
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
