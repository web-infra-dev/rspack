use std::sync::LazyLock;

use rspack_core::SideEffectsBailoutItemWithSpan;
use swc_core::{
  common::{
    Mark, Spanned, SyntaxContext,
    comments::{CommentKind, Comments},
  },
  ecma::{
    ast::{
      Class, ClassMember, Decl, Expr, Function, ModuleDecl, Pat, PropName, VarDecl, VarDeclOrExpr,
    },
    utils::{ExprCtx, ExprExt},
  },
};

use crate::{
  ClassExt, JavascriptParserPlugin,
  visitors::{JavascriptParser, Statement, VariableDeclaration},
};

static PURE_COMMENTS: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new("^\\s*(#|@)__PURE__\\s*$").expect("Should create the regex"));

pub struct SideEffectsParserPlugin {
  unresolve_ctxt: SyntaxContext,
}

impl SideEffectsParserPlugin {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolve_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
    }
  }
}

impl JavascriptParserPlugin for SideEffectsParserPlugin {
  fn module_declaration(&self, parser: &mut JavascriptParser, decl: &ModuleDecl) -> Option<bool> {
    match decl {
      ModuleDecl::ExportDefaultExpr(expr) => {
        if !is_pure_expression(&expr.expr, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr.span,
            String::from("ExportDefaultExpr"),
          ));
        }
      }
      ModuleDecl::ExportDecl(decl) => {
        if !is_pure_decl(&decl.decl, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            decl.decl.span(),
            String::from("Decl"),
          ));
        }
      }
      _ => {}
    };
    None
  }
  fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    if !parser.is_top_level_scope() {
      return None;
    }
    self.analyze_stmt_side_effects(&stmt, parser);
    None
  }
}

fn is_pure_call_expr(
  expr: &Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };
  let callee = &call_expr.callee;
  let pure_flag = comments
    .and_then(|comments| {
      if let Some(comment_list) = comments.get_leading(callee.span().lo) {
        return Some(comment_list.iter().any(|comment| {
          comment.kind == CommentKind::Block && PURE_COMMENTS.is_match(&comment.text)
        }));
      }
      None
    })
    .unwrap_or(false);
  if !pure_flag {
    !expr.may_have_side_effects(ExprCtx {
      unresolved_ctxt,
      in_strict: false,
      is_unresolved_ref_safe: false,
      remaining_depth: 4,
    })
  } else {
    call_expr.args.iter().all(|arg| {
      if arg.spread.is_some() {
        false
      } else {
        is_pure_expression(&arg.expr, unresolved_ctxt, comments)
      }
    })
  }
}

impl SideEffectsParserPlugin {
  fn analyze_stmt_side_effects(&self, stmt: &Statement, parser: &mut JavascriptParser) {
    if parser.side_effects_item.is_some() {
      return;
    }
    match stmt {
      Statement::If(if_stmt) => {
        if !is_pure_expression(&if_stmt.test, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            if_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::While(while_stmt) => {
        if !is_pure_expression(&while_stmt.test, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            while_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::DoWhile(do_while_stmt) => {
        if !is_pure_expression(&do_while_stmt.test, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            do_while_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::For(for_stmt) => {
        let pure_init = match for_stmt.init {
          Some(ref init) => match init {
            VarDeclOrExpr::VarDecl(decl) => {
              is_pure_var_decl(decl, self.unresolve_ctxt, parser.comments)
            }
            VarDeclOrExpr::Expr(expr) => {
              is_pure_expression(expr, self.unresolve_ctxt, parser.comments)
            }
          },
          None => true,
        };

        if !pure_init {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
            String::from("Statement"),
          ));
          return;
        }

        let pure_test = match &for_stmt.test {
          Some(test) => is_pure_expression(test, self.unresolve_ctxt, parser.comments),
          None => true,
        };

        if !pure_test {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
            String::from("Statement"),
          ));
          return;
        }

        let pure_update = match for_stmt.update {
          Some(ref expr) => is_pure_expression(expr, self.unresolve_ctxt, parser.comments),
          None => true,
        };

        if !pure_update {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Expr(expr_stmt) => {
        if !is_pure_expression(&expr_stmt.expr, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Switch(switch_stmt) => {
        if !is_pure_expression(
          &switch_stmt.discriminant,
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            switch_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Class(class_stmt) => {
        if !is_pure_class(class_stmt.class(), self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            class_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Var(var_stmt) => match var_stmt {
        VariableDeclaration::VarDecl(var_decl) => {
          if !is_pure_var_decl(var_decl, self.unresolve_ctxt, parser.comments) {
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              var_stmt.span(),
              String::from("Statement"),
            ));
          }
        }
        VariableDeclaration::UsingDecl(_) => {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            var_stmt.span(),
            String::from("Statement"),
          ));
        }
      },
      Statement::Empty(_) => {}
      Statement::Labeled(_) => {}
      Statement::Block(_) => {}
      Statement::Fn(_) => {}
      _ => {
        parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
          stmt.span(),
          String::from("Statement"),
        ))
      }
    };
  }
}

pub fn is_pure_pat<'a>(
  pat: &'a Pat,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => array_pat.elems.iter().all(|ele| {
      if let Some(pat) = ele {
        is_pure_pat(pat, unresolved_ctxt, comments)
      } else {
        true
      }
    }),
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(expr, unresolved_ctxt, comments),
  }
}

pub fn is_pure_function<'a>(
  function: &'a Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  if !function
    .params
    .iter()
    .all(|param| is_pure_pat(&param.pat, unresolved_ctxt, comments))
  {
    return false;
  }

  true
}

pub fn is_pure_expression<'a>(
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a dyn Comments>,
  ) -> bool {
    match expr {
      Expr::Call(_) => is_pure_call_expr(expr, unresolved_ctxt, comments),
      Expr::Paren(_) => unreachable!(),
      _ => !expr.may_have_side_effects(ExprCtx {
        unresolved_ctxt,
        is_unresolved_ref_safe: true,
        in_strict: false,
        remaining_depth: 4,
      }),
    }
  }
  _is_pure_expression(expr, unresolved_ctxt, comments)
}

pub fn is_pure_class_member<'a>(
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  let is_key_pure = match member.class_key() {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => {
      is_pure_expression(&computed.expr, unresolved_ctxt, comments)
    }
    Some(PropName::BigInt(_)) => true,
    None => true,
  };
  if !is_key_pure {
    return false;
  }
  let is_static = member.is_static();
  let is_value_pure = match member {
    ClassMember::Constructor(_) => true,
    ClassMember::Method(_) => true,
    ClassMember::PrivateMethod(_) => true,
    ClassMember::ClassProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(value, unresolved_ctxt, comments)
      } else {
        true
      }
    }
    ClassMember::PrivateProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(value, unresolved_ctxt, comments)
      } else {
        true
      }
    }
    ClassMember::TsIndexSignature(_) => unreachable!(),
    ClassMember::Empty(_) => true,
    ClassMember::StaticBlock(_) => false,
    ClassMember::AutoAccessor(_) => false,
  };
  if is_static && !is_value_pure {
    return false;
  }
  true
}

pub fn is_pure_decl(
  stmt: &Decl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(&class.class, unresolved_ctxt, comments),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(var, unresolved_ctxt, comments),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

pub fn is_pure_class(
  class: &Class,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if let Some(ref super_class) = class.super_class
    && !is_pure_expression(super_class, unresolved_ctxt, comments)
  {
    return false;
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => is_pure_expression(&computed.expr, unresolved_ctxt, comments),
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => is_pure_key(&method.key),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        &Expr::PrivateName(method.key.clone()),
        unresolved_ctxt,
        comments,
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(&prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt, comments)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          &Expr::PrivateName(prop.key.clone()),
          unresolved_ctxt,
          comments,
        ) && (!prop.is_static
          || if let Some(ref value) = prop.value {
            is_pure_expression(value, unresolved_ctxt, comments)
          } else {
            true
          })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(_) => true,
    }
  })
}

fn is_pure_var_decl<'a>(
  var: &'a VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(init, unresolved_ctxt, comments)
    } else {
      true
    }
  })
}
