use itertools::Itertools;
use rspack_core::{ConstDependency, SpanExt};
use rustc_hash::FxHashSet;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{BlockStmt, DoWhileStmt, ForHead, ForInStmt, ForOfStmt, Ident, IfStmt};
use swc_core::ecma::ast::{LabeledStmt, ObjectPatProp, Pat, Stmt, VarDeclOrExpr, WhileStmt};
use swc_core::ecma::ast::{VarDecl, VarDeclKind, VarDeclarator};

use crate::visitors::JavascriptParser;

fn get_hoisted_declarations<'a>(
  branch: &'a Stmt,
  include_function_declarations: bool,
) -> FxHashSet<&'a Ident> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![branch];

  let collect_declaration_from_ident =
    |ident: &'a Ident, declarations: &mut FxHashSet<&'a Ident>| {
      declarations.insert(ident);
    };

  let collect_declaration_from_pat = |pattern: &'a Pat, declarations: &mut FxHashSet<&'a Ident>| {
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
            match &property {
              ObjectPatProp::KeyValue(key_value) => stack.push(&key_value.value),
              ObjectPatProp::Assign(assign) => {
                collect_declaration_from_ident(&assign.key, declarations)
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

  fn get_var_kind_decls(decl: &VarDecl) -> Option<&Vec<VarDeclarator>> {
    matches!(decl.kind, VarDeclKind::Var).then_some(&decl.decls)
  }

  fn collect_from_block_stm<'b>(block: &'b BlockStmt, stack: &mut Vec<&'b Stmt>) {
    for stmt in &block.stmts {
      stack.push(stmt);
    }
  }

  while let Some(node) = stmt_stack.pop() {
    match node {
      Stmt::Block(block) => {
        collect_from_block_stm(block, &mut stmt_stack);
      }
      Stmt::If(r#if) => {
        stmt_stack.push(&r#if.cons);
        if let Some(alt) = &r#if.alt {
          stmt_stack.push(alt);
        }
      }
      Stmt::For(r#for) => {
        if let Some(init) = &r#for.init
          && let VarDeclOrExpr::VarDecl(var_decl) = &init
          && let Some(decls) = get_var_kind_decls(var_decl)
        {
          for decl in decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(&r#for.body);
      }
      Stmt::ForIn(ForInStmt { left, body, .. }) | Stmt::ForOf(ForOfStmt { left, body, .. }) => {
        if let ForHead::VarDecl(var_decl) = &left {
          for decl in &var_decl.decls {
            collect_declaration_from_pat(&decl.name, &mut declarations);
          }
        }
        stmt_stack.push(body);
      }
      Stmt::DoWhile(DoWhileStmt { body, .. })
      | Stmt::While(WhileStmt { body, .. })
      | Stmt::Labeled(LabeledStmt { body, .. }) => {
        stmt_stack.push(body);
      }
      Stmt::Switch(switch) => {
        for cs in &switch.cases {
          for consequent in &cs.cons {
            stmt_stack.push(consequent);
          }
        }
      }
      Stmt::Try(r#try) => {
        collect_from_block_stm(&r#try.block, &mut stmt_stack);
        if let Some(handler) = &r#try.handler {
          collect_from_block_stm(&handler.body, &mut stmt_stack);
        }
        if let Some(finalizer) = &r#try.finalizer {
          collect_from_block_stm(finalizer, &mut stmt_stack);
        }
      }
      Stmt::Decl(decl)
        if let Some(r#fn) = decl.as_fn_decl()
          && include_function_declarations =>
      {
        collect_declaration_from_ident(&r#fn.ident, &mut declarations);
      }
      Stmt::Decl(decl)
        if let Some(var) = decl.as_var()
          && let Some(decls) = get_var_kind_decls(var) =>
      {
        for decl in decls {
          collect_declaration_from_pat(&decl.name, &mut declarations);
        }
      }
      _ => {}
    }
  }

  declarations
}

pub fn statement_if(scanner: &mut JavascriptParser, stmt: &IfStmt) -> Option<bool> {
  let param = scanner.evaluate_expression(&stmt.test);
  let Some(boolean) = param.as_bool() else {
    return None;
  };
  if !param.could_have_side_effects() {
    scanner
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        param.range().0,
        param.range().1 - 1,
        boolean.to_string().into_boxed_str(),
        None,
      )));
  } else {
    scanner.walk_expression(&stmt.test);
  }

  let branch_to_remove = if boolean {
    stmt.alt.as_ref()
  } else {
    Some(&stmt.cons)
  };

  if let Some(branch_to_remove) = branch_to_remove {
    let declarations = if scanner.is_strict() {
      get_hoisted_declarations(branch_to_remove, false)
    } else {
      get_hoisted_declarations(branch_to_remove, true)
    };
    let replacement = if declarations.is_empty() {
      "{}".to_string()
    } else {
      format!(
        "{{ var {} }}",
        declarations.iter().map(|decl| decl.sym.as_str()).join(", ")
      )
    };

    scanner
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        branch_to_remove.span().real_lo(),
        branch_to_remove.span().hi().0 - 1,
        replacement.into_boxed_str(),
        None,
      )))
  }
  Some(boolean)
}
