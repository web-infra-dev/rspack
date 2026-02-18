use itertools::Itertools;
use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{
  Ast, BlockStmt, ForHead, GetSpan, Ident, IfStmt, ObjectPatProp, Pat, Stmt, TypedSubRange,
  VarDecl, VarDeclKind, VarDeclOrExpr, VarDeclarator,
};

use crate::visitors::JavascriptParser;

fn get_hoisted_declarations(
  ast: &Ast,
  branch: Stmt,
  include_function_declarations: bool,
) -> FxHashSet<Ident> {
  let mut declarations = FxHashSet::default();
  let mut stmt_stack = vec![branch];

  let collect_declaration_from_ident = |ident: Ident, declarations: &mut FxHashSet<Ident>| {
    declarations.insert(ident);
  };

  let collect_declaration_from_pat =
    |ast: &Ast, pattern: Pat, declarations: &mut FxHashSet<Ident>| {
      let mut stack = vec![pattern];
      while let Some(node) = stack.pop() {
        match node {
          Pat::Ident(ident) => collect_declaration_from_ident(ident.id(ast), declarations),
          Pat::Array(array) => {
            for element in array.elems(ast).iter() {
              let element = ast.get_node_in_sub_range(element);
              if let Some(element) = element {
                stack.push(element);
              }
            }
          }
          Pat::Assign(assign) => stack.push(assign.left(ast)),
          Pat::Object(object) => {
            for property in object.props(ast).iter() {
              let property = ast.get_node_in_sub_range(property);
              match property {
                ObjectPatProp::KeyValue(key_value) => stack.push(key_value.value(ast)),
                ObjectPatProp::Assign(assign) => {
                  collect_declaration_from_ident(assign.key(ast).id(ast), declarations)
                }
                ObjectPatProp::Rest(rest) => stack.push(rest.arg(ast)),
              }
            }
          }
          Pat::Rest(rest) => stack.push(rest.arg(ast)),
          _ => {}
        }
      }
    };

  fn get_var_kind_decls(ast: &Ast, decl: VarDecl) -> Option<TypedSubRange<VarDeclarator>> {
    matches!(decl.kind(ast), VarDeclKind::Var).then_some(decl.decls(ast))
  }

  fn collect_from_block_stm(ast: &Ast, block: BlockStmt, stack: &mut Vec<Stmt>) {
    for stmt in block.stmts(ast).iter() {
      let stmt = ast.get_node_in_sub_range(stmt);
      stack.push(stmt);
    }
  }

  while let Some(node) = stmt_stack.pop() {
    match node {
      Stmt::Block(block) => {
        collect_from_block_stm(ast, block, &mut stmt_stack);
      }
      Stmt::If(r#if) => {
        stmt_stack.push(r#if.cons(ast));
        if let Some(alt) = r#if.alt(ast) {
          stmt_stack.push(alt);
        }
      }
      Stmt::For(r#for) => {
        if let Some(init) = r#for.init(ast)
          && let VarDeclOrExpr::VarDecl(var_decl) = init
          && let Some(decls) = get_var_kind_decls(ast, var_decl)
        {
          for decl in decls.iter() {
            let decl = ast.get_node_in_sub_range(decl);
            collect_declaration_from_pat(ast, decl.name(ast), &mut declarations);
          }
        }
        stmt_stack.push(r#for.body(ast));
      }
      Stmt::ForIn(for_in) => {
        let left = for_in.left(ast);
        let body = for_in.body(ast);
        if let ForHead::VarDecl(var_decl) = left {
          for decl in var_decl.decls(ast).iter() {
            let decl = ast.get_node_in_sub_range(decl);
            collect_declaration_from_pat(ast, decl.name(ast), &mut declarations);
          }
        }
        stmt_stack.push(body);
      }
      Stmt::ForOf(for_of) => {
        let left = for_of.left(ast);
        let body = for_of.body(ast);
        if let ForHead::VarDecl(var_decl) = left {
          for decl in var_decl.decls(ast).iter() {
            let decl = ast.get_node_in_sub_range(decl);
            collect_declaration_from_pat(ast, decl.name(ast), &mut declarations);
          }
        }
        stmt_stack.push(body);
      }
      Stmt::DoWhile(do_while) => stmt_stack.push(do_while.body(ast)),
      Stmt::While(while_stmt) => stmt_stack.push(while_stmt.body(ast)),
      Stmt::Labeled(labeled_stmt) => stmt_stack.push(labeled_stmt.body(ast)),
      Stmt::Switch(switch) => {
        for cs in switch.cases(ast).iter() {
          let cs = ast.get_node_in_sub_range(cs);
          for consequent in cs.cons(ast).iter() {
            let consequent = ast.get_node_in_sub_range(consequent);
            stmt_stack.push(consequent);
          }
        }
      }
      Stmt::Try(r#try) => {
        collect_from_block_stm(ast, r#try.block(ast), &mut stmt_stack);
        if let Some(handler) = r#try.handler(ast) {
          collect_from_block_stm(ast, handler.body(ast), &mut stmt_stack);
        }
        if let Some(finalizer) = r#try.finalizer(ast) {
          collect_from_block_stm(ast, finalizer, &mut stmt_stack);
        }
      }
      Stmt::Decl(decl) if decl.as_fn().is_some() && include_function_declarations => {
        let r#fn = decl
          .as_fn()
          .expect("decl is `FunctionDeclaration` in `if_guard`");
        collect_declaration_from_ident(r#fn.ident(ast), &mut declarations);
      }
      Stmt::Decl(decl) if decl.as_var().is_some() => {
        let Some(var) = decl.as_var() else {
          continue;
        };
        let Some(decls) = get_var_kind_decls(ast, var) else {
          continue;
        };
        for decl in decls.iter() {
          let decl = ast.get_node_in_sub_range(decl);
          collect_declaration_from_pat(ast, decl.name(ast), &mut declarations);
        }
      }
      _ => {}
    }
  }

  declarations
}

pub fn statement_if(scanner: &mut JavascriptParser, stmt: IfStmt) -> Option<bool> {
  let param = scanner.evaluate_expression(stmt.test(&scanner.ast));
  let boolean = param.as_bool()?;
  if !param.could_have_side_effects() {
    scanner.add_presentational_dependency(Box::new(ConstDependency::new(
      param.range().into(),
      boolean.to_string().into_boxed_str(),
    )));
  } else {
    scanner.walk_expression(stmt.test(&scanner.ast));
  }

  let branch_to_remove = if boolean {
    stmt.alt(&scanner.ast)
  } else {
    Some(stmt.cons(&scanner.ast))
  };

  if let Some(branch_to_remove) = branch_to_remove {
    let declarations = if scanner.is_strict() {
      get_hoisted_declarations(&scanner.ast, branch_to_remove, false)
    } else {
      get_hoisted_declarations(&scanner.ast, branch_to_remove, true)
    };
    let replacement = if declarations.is_empty() {
      "{}".to_string()
    } else {
      format!(
        "{{ var {} }}",
        declarations
          .iter()
          .map(|decl| scanner.ast.get_utf8(decl.sym(&scanner.ast)))
          .join(", ")
      )
    };

    scanner.add_presentational_dependency(Box::new(ConstDependency::new(
      (
        branch_to_remove.span(&scanner.ast).real_lo(),
        branch_to_remove.span(&scanner.ast).real_hi(),
      )
        .into(),
      replacement.into_boxed_str(),
    )))
  }
  Some(boolean)
}
