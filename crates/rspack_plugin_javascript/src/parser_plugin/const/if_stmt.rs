use itertools::Itertools;
use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_core::{
  common::Spanned,
  ecma::ast::{
    DoWhileStmt, ForHead, ForInStmt, ForOfStmt, Ident, IfStmt, LabeledStmt, ObjectPatProp, Pat,
    Stmt, VarDeclKind, VarDeclOrExpr, WhileStmt,
  },
};

use crate::visitors::{JavascriptParser, Statement, VariableDeclarationKind};

/// Collects hoisted `var` and (in non-strict) function/class declaration idents from a statement.
pub fn get_hoisted_declarations<'ast>(
  stmt: Statement<'ast>,
  include_function_declarations: bool,
) -> FxHashSet<&'ast Ident> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![stmt];

  let collect_declaration_from_ident =
    |ident: &'ast Ident, declarations: &mut FxHashSet<&'ast Ident>| {
      declarations.insert(ident);
    };

  let collect_declaration_from_pat =
    |pattern: &'ast Pat, declarations: &mut FxHashSet<&'ast Ident>| {
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
                  collect_declaration_from_ident(&assign.key, declarations);
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
    match node {
      Statement::Block(block) => {
        for s in &block.stmts {
          stmt_stack.push(Statement::from(s));
        }
      }
      Statement::If(r#if) => {
        stmt_stack.push(Statement::from(r#if.cons.as_ref()));
        if let Some(alt) = &r#if.alt {
          stmt_stack.push(Statement::from(alt.as_ref()));
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
        stmt_stack.push(Statement::from(r#for.body.as_ref()));
      }
      Statement::ForIn(ForInStmt { left, body, .. })
      | Statement::ForOf(ForOfStmt { left, body, .. }) => {
        if let ForHead::VarDecl(var_decl) = left {
          for decl in &var_decl.decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(Statement::from(body.as_ref()));
      }
      Statement::DoWhile(DoWhileStmt { body, .. })
      | Statement::While(WhileStmt { body, .. })
      | Statement::Labeled(LabeledStmt { body, .. }) => {
        stmt_stack.push(Statement::from(body.as_ref()));
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

pub fn statement_if(scanner: &mut JavascriptParser, stmt: &IfStmt) -> Option<bool> {
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
    stmt.alt.as_ref().map(|b| b.as_ref())
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
      format!(
        "{{ var {} }}",
        declarations.iter().map(|decl| decl.sym.as_str()).join(", ")
      )
    };

    scanner.add_presentational_dependency(Box::new(ConstDependency::new(
      (
        branch_to_remove.span().real_lo(),
        branch_to_remove.span().real_hi(),
      )
        .into(),
      replacement.into_boxed_str(),
    )))
  }
  Some(boolean)
}
