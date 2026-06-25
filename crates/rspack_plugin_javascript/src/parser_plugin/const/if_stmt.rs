use itertools::Itertools;
use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{
  ForHead, GetSpan, Ident, IfStmt, ObjectPatProp, Pat, Stmt, VarDeclKind, VarDeclOrExpr,
};

use crate::visitors::{JavascriptParser, Statement, VariableDeclarationKind};

/// Collects hoisted `var` and (in non-strict) function/class declaration idents from a statement.
pub fn get_hoisted_declarations<'a>(
  stmt: Statement<'a>,
  include_function_declarations: bool,
) -> FxHashSet<&'a str> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![stmt];

  let collect_declaration_from_ident = |ident: &'a Ident, declarations: &mut FxHashSet<&'a str>| {
    declarations.insert(ident.sym.as_str());
  };

  let collect_declaration_from_pat = |pattern: &'a Pat, declarations: &mut FxHashSet<&'a str>| {
    let mut stack = vec![pattern];
    while let Some(node) = stack.pop() {
      match node {
        Pat::Ident(ident) => collect_declaration_from_ident(&ident.id, declarations),
        Pat::Array(array) => {
          for element in array.elems.iter().flatten() {
            stack.push(element);
          }
        }
        Pat::Assign(assign) => stack.push(&assign.left),
        Pat::Object(object) => {
          for property in &object.props {
            match property {
              ObjectPatProp::KeyValue(key_value) => stack.push(&key_value.value),
              ObjectPatProp::Assign(assign) => {
                collect_declaration_from_ident(&assign.key.id, declarations);
              }
              ObjectPatProp::Rest(rest) => stack.push(&rest.arg),
            }
          }
        }
        Pat::Rest(rest) => stack.push(&rest.arg),
        _ => {}
      }
    }
  };

  while let Some(node) = stmt_stack.pop() {
    #[allow(clippy::collapsible_match)]
    match node {
      Statement::Block(block) => {
        for s in &block.stmts {
          stmt_stack.push(Statement::from(s));
        }
      }
      Statement::If(r#if) => {
        stmt_stack.push(Statement::from(&r#if.cons));
        if let Some(alt) = &r#if.alt {
          stmt_stack.push(Statement::from(alt));
        }
      }
      Statement::For(r#for) => {
        if let Some(init) = &r#for.init
          && let VarDeclOrExpr::VarDecl(var_decl) = init
          && matches!(var_decl.kind, VarDeclKind::Var)
        {
          for decl in &var_decl.decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(Statement::from(&r#for.body));
      }
      Statement::ForIn(stmt) => {
        if let ForHead::VarDecl(var_decl) = &stmt.left
          && matches!(var_decl.kind, VarDeclKind::Var)
        {
          for decl in &var_decl.decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(Statement::from(&stmt.body));
      }
      Statement::ForOf(stmt) => {
        if let ForHead::VarDecl(var_decl) = &stmt.left
          && matches!(var_decl.kind, VarDeclKind::Var)
        {
          for decl in &var_decl.decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(Statement::from(&stmt.body));
      }
      Statement::DoWhile(stmt) => {
        stmt_stack.push(Statement::from(&stmt.body));
      }
      Statement::While(stmt) => {
        stmt_stack.push(Statement::from(&stmt.body));
      }
      Statement::Labeled(stmt) => {
        stmt_stack.push(Statement::from(&stmt.body));
      }
      Statement::Switch(switch) => {
        for case in &switch.cases {
          for consequent in &case.cons {
            stmt_stack.push(Statement::from(consequent));
          }
        }
      }
      Statement::Try(r#try) => {
        for s in &r#try.block.stmts {
          stmt_stack.push(Statement::from(s));
        }
        if let Some(handler) = &r#try.handler {
          for s in &handler.body.stmts {
            stmt_stack.push(Statement::from(s));
          }
        }
        if let Some(finalizer) = &r#try.finalizer {
          for s in &finalizer.stmts {
            stmt_stack.push(Statement::from(s));
          }
        }
      }
      Statement::Fn(fn_decl) if include_function_declarations => {
        if let Some(ident) = fn_decl.ident() {
          collect_declaration_from_ident(ident, &mut declarations);
        }
      }
      Statement::Var(var_decl) => {
        if var_decl.kind() == VariableDeclarationKind::Var {
          for decl in var_decl.declarators() {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
      }
      _ => {}
    }
  }

  declarations
}

pub fn statement_if<'p>(scanner: &mut JavascriptParser<'p>, stmt: &IfStmt<'_>) -> Option<bool> {
  let param = scanner.evaluate_expression(&stmt.test);
  let boolean = param.as_bool()?;
  if !param.could_have_side_effects() {
    scanner.add_presentational_dependency(Box::new(ConstDependency::new(
      param.range().into(),
      boolean.to_string().into_boxed_str(),
    )));
  } else {
    scanner.walk_expression(&stmt.test);
  }

  let branch_to_remove: Option<&Stmt> = if boolean {
    stmt.alt.as_ref()
  } else {
    Some(&stmt.cons)
  };

  if let Some(branch_to_remove) = branch_to_remove {
    let branch_stmt = Statement::from(branch_to_remove);
    let declarations = if scanner.is_strict() {
      get_hoisted_declarations(branch_stmt, false)
    } else {
      get_hoisted_declarations(branch_stmt, true)
    };
    let replacement = if declarations.is_empty() {
      "{}".to_string()
    } else {
      format!("{{ var {} }}", declarations.iter().join(", "))
    };

    scanner.add_presentational_dependency(Box::new(ConstDependency::new(
      {
        let span = branch_to_remove.span();
        (span.real_lo(), span.real_hi()).into()
      },
      replacement.into_boxed_str(),
    )))
  }
  Some(boolean)
}
