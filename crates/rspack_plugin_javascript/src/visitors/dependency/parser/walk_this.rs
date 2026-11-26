use rspack_util::swc::AstSubRangeExt;
use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
  Ast, DeclData, Expr, ExprData, Function, PropertyDefinition, StaticBlock, Stmt, StmtData,
  SubRange, ThisExpression, VariableKind, Visit, VisitWith,
};

use super::JavascriptParser;
use crate::Atom;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TopLevelFunction {
  Function(Function),
  Arrow,
}

struct ThisExpressionFinder<'a> {
  ast: &'a Ast<'a>,
  found: bool,
}

impl<'a> Visit<'a> for ThisExpressionFinder<'a> {
  fn ast(&self) -> &Ast<'a> {
    self.ast
  }

  fn visit_stmt(&mut self, node: Stmt) {
    if !self.found {
      node.visit_children_with(self);
    }
  }

  fn visit_expr(&mut self, node: Expr) {
    if !self.found {
      node.visit_children_with(self);
    }
  }

  fn visit_this_expression(&mut self, _node: ThisExpression) {
    self.found = true;
  }

  fn visit_function(&mut self, _node: Function) {}

  fn visit_property_definition(&mut self, node: PropertyDefinition) {
    node.decorators(self.ast).visit_with(self);
    node.key(self.ast).visit_with(self);
  }

  fn visit_static_block(&mut self, _node: StaticBlock) {}
}

pub fn is_this_free_function(ast: &Ast, function: Function) -> bool {
  let mut finder = ThisExpressionFinder { ast, found: false };
  function.params(ast).visit_with(&mut finder);
  if !finder.found {
    function.body(ast).visit_with(&mut finder);
  }
  !finder.found
}

pub fn is_this_free_function_expression(ast: &Ast, expr: Expr) -> bool {
  match ast.expr_data(expr) {
    ExprData::ArrowFunctionExpression(_) => true,
    ExprData::Function(function) => is_this_free_function(ast, function),
    _ => false,
  }
}

fn collect_top_level_functions(
  ast: &Ast,
  statements: SubRange<Stmt>,
) -> FxHashMap<Atom, TopLevelFunction> {
  let mut functions = FxHashMap::default();
  for statement in ast.nodes(statements) {
    let declaration = match ast.stmt_data(statement) {
      StmtData::Declaration(declaration) => declaration,
      StmtData::ExportNamedDeclaration(export) => match export.declaration(ast) {
        Some(declaration) => declaration,
        None => continue,
      },
      _ => continue,
    };
    match ast.decl_data(declaration) {
      DeclData::Function(function) => {
        if let Some(id) = function.id(ast) {
          functions.insert(
            Atom::from(ast.get_utf8(id.name(ast))),
            TopLevelFunction::Function(function),
          );
        }
      }
      DeclData::VariableDeclaration(variable) if variable.kind(ast) == VariableKind::Const => {
        for declarator in ast.nodes(variable.declarators(ast)) {
          let Some(id) = declarator.id(ast).as_binding_identifier(ast) else {
            continue;
          };
          let Some(init) = declarator.init(ast) else {
            continue;
          };
          let function = match ast.expr_data(init) {
            ExprData::ArrowFunctionExpression(_) => TopLevelFunction::Arrow,
            ExprData::Function(function) => TopLevelFunction::Function(function),
            _ => continue,
          };
          functions.insert(Atom::from(ast.get_utf8(id.name(ast))), function);
        }
      }
      _ => {}
    }
  }
  functions
}

impl JavascriptParser<'_> {
  pub(crate) fn is_this_free_top_level_function(&mut self, name: &Atom) -> bool {
    let parsed = self.ast;
    let ast = parsed.ast;
    let functions = self
      .top_level_functions
      .get_or_insert_with(|| collect_top_level_functions(ast, parsed.program.body(ast)));
    match functions.get(name) {
      Some(TopLevelFunction::Arrow) => true,
      Some(TopLevelFunction::Function(function)) => is_this_free_function(ast, *function),
      None => false,
    }
  }
}
