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
        if !is_pure_expression(parser, &expr.expr, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr.span,
            String::from("ExportDefaultExpr"),
          ));
        }
      }
      ModuleDecl::ExportDecl(decl) => {
        if !is_pure_decl(parser, &decl.decl, self.unresolve_ctxt, parser.comments) {
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
  parser: &mut JavascriptParser,
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
        is_pure_expression(parser, &arg.expr, unresolved_ctxt, comments)
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
        if !is_pure_expression(parser, &if_stmt.test, self.unresolve_ctxt, parser.comments) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            if_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::While(while_stmt) => {
        if !is_pure_expression(
          parser,
          &while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            while_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::DoWhile(do_while_stmt) => {
        if !is_pure_expression(
          parser,
          &do_while_stmt.test,
          self.unresolve_ctxt,
          parser.comments,
        ) {
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
              is_pure_var_decl(parser, decl, self.unresolve_ctxt, parser.comments)
            }
            VarDeclOrExpr::Expr(expr) => {
              is_pure_expression(parser, expr, self.unresolve_ctxt, parser.comments)
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
          Some(test) => is_pure_expression(parser, test, self.unresolve_ctxt, parser.comments),
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
          Some(ref expr) => is_pure_expression(parser, expr, self.unresolve_ctxt, parser.comments),
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
        if !is_pure_expression(
          parser,
          &expr_stmt.expr,
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Switch(switch_stmt) => {
        if !is_pure_expression(
          parser,
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
        if !is_pure_class(
          parser,
          class_stmt.class(),
          self.unresolve_ctxt,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            class_stmt.span(),
            String::from("Statement"),
          ));
        }
      }
      Statement::Var(var_stmt) => match var_stmt {
        VariableDeclaration::VarDecl(var_decl) => {
          if !is_pure_var_decl(parser, var_decl, self.unresolve_ctxt, parser.comments) {
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
  parser: &mut JavascriptParser,
  pat: &'a Pat,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => array_pat.elems.iter().all(|ele| {
      if let Some(pat) = ele {
        is_pure_pat(parser, pat, unresolved_ctxt, comments)
      } else {
        true
      }
    }),
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(parser, expr, unresolved_ctxt, comments),
  }
}

pub fn is_pure_function<'a>(
  parser: &mut JavascriptParser,
  function: &'a Function,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  if !function
    .params
    .iter()
    .all(|param| is_pure_pat(parser, &param.pat, unresolved_ctxt, comments))
  {
    return false;
  }

  true
}

pub fn is_pure_expression<'a>(
  parser: &mut JavascriptParser,
  expr: &'a Expr,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  pub(crate) fn _is_pure_expression<'a>(
    parser: &mut JavascriptParser,
    expr: &'a Expr,
    unresolved_ctxt: SyntaxContext,
    comments: Option<&'a dyn Comments>,
  ) -> bool {
    let drive = parser.plugin_drive.clone();
    if let Some(res) = drive.is_pure(parser, expr) {
      return res;
    }

    match expr {
      Expr::Call(_) => is_pure_call_expr(parser, expr, unresolved_ctxt, comments),
      Expr::Paren(_) => unreachable!(),
      Expr::Seq(seq_expr) => seq_expr
        .exprs
        .iter()
        .all(|expr| is_pure_expression(parser, expr, unresolved_ctxt, comments)),
      _ => !expr.may_have_side_effects(ExprCtx {
        unresolved_ctxt,
        is_unresolved_ref_safe: true,
        in_strict: false,
        remaining_depth: 4,
      }),
    }
  }
  _is_pure_expression(parser, expr, unresolved_ctxt, comments)
}

pub fn is_pure_class_member<'a>(
  parser: &mut JavascriptParser,
  member: &'a ClassMember,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  let is_key_pure = match member.class_key() {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => {
      is_pure_expression(parser, &computed.expr, unresolved_ctxt, comments)
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
        is_pure_expression(parser, value, unresolved_ctxt, comments)
      } else {
        true
      }
    }
    ClassMember::PrivateProp(prop) => {
      if let Some(ref value) = prop.value {
        is_pure_expression(parser, value, unresolved_ctxt, comments)
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
  parser: &mut JavascriptParser,
  stmt: &Decl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(parser, &class.class, unresolved_ctxt, comments),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(parser, var, unresolved_ctxt, comments),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

pub fn is_pure_class(
  parser: &mut JavascriptParser,
  class: &Class,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&dyn Comments>,
) -> bool {
  if let Some(ref super_class) = class.super_class
    && !is_pure_expression(parser, super_class, unresolved_ctxt, comments)
  {
    return false;
  }
  let is_pure_key = |parser: &mut JavascriptParser, key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => {
        is_pure_expression(parser, &computed.expr, unresolved_ctxt, comments)
      }
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => is_pure_key(parser, &method.key),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        parser,
        &Expr::PrivateName(method.key.clone()),
        unresolved_ctxt,
        comments,
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(parser, &prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(parser, value, unresolved_ctxt, comments)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          parser,
          &Expr::PrivateName(prop.key.clone()),
          unresolved_ctxt,
          comments,
        ) && (!prop.is_static
          || if let Some(ref value) = prop.value {
            is_pure_expression(parser, value, unresolved_ctxt, comments)
          } else {
            true
          })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => false, // TODO: support is pure analyze for statements
      ClassMember::AutoAccessor(_) => false,
    }
  })
}

fn is_pure_var_decl<'a>(
  parser: &mut JavascriptParser,
  var: &'a VarDecl,
  unresolved_ctxt: SyntaxContext,
  comments: Option<&'a dyn Comments>,
) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(parser, init, unresolved_ctxt, comments)
    } else {
      true
    }
  })
}
