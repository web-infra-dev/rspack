use std::sync::LazyLock;

use rspack_core::SideEffectsBailoutItemWithSpan;
use swc_core::common::comments::{CommentKind, Comments};
use swc_experimental_ecma_ast::{
  Class, ClassMember, Decl, Expr, Function, ModuleDecl, Pat, PropName, Spanned, VarDecl,
  VarDeclOrExpr,
};
use swc_experimental_ecma_semantic::ScopeId;
use swc_experimental_ecma_utils::{ExprCtx, ExprExt};

use crate::{
  ClassExt, JavascriptParserPlugin,
  visitors::{JavascriptParser, Statement, VariableDeclaration},
};

static PURE_COMMENTS: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new("^\\s*(#|@)__PURE__\\s*$").expect("Should create the regex"));

pub struct SideEffectsParserPlugin {
  unresolved_scope_id: ScopeId,
}

impl SideEffectsParserPlugin {
  pub fn new(unresolved_scope_id: ScopeId) -> Self {
    Self {
      unresolved_scope_id,
    }
  }
}

impl JavascriptParserPlugin for SideEffectsParserPlugin {
  fn module_declaration(&self, parser: &mut JavascriptParser, decl: ModuleDecl) -> Option<bool> {
    match decl {
      ModuleDecl::ExportDefaultExpr(expr) => {
        if !is_pure_expression(
          parser,
          expr.expr(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr.span(&parser.ast),
            String::from("ExportDefaultExpr"),
          ));
        }
      }
      ModuleDecl::ExportDecl(decl) => {
        if !is_pure_decl(
          parser,
          decl.decl(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            decl.decl(&parser.ast).span(&parser.ast),
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
    self.analyze_stmt_side_effects(stmt, parser);
    None
  }
}

fn is_pure_call_expr(
  parser: &mut JavascriptParser,
  expr: Expr,
  unresolved_scope_id: ScopeId,
  comments: Option<&dyn Comments>,
) -> bool {
  let Expr::Call(call_expr) = expr else {
    unreachable!();
  };
  let callee = call_expr.callee(&parser.ast);
  let pure_flag = comments
    .and_then(|comments| {
      if let Some(comment_list) = comments.get_leading(callee.span(&parser.ast).lo) {
        return Some(comment_list.iter().any(|comment| {
          comment.kind == CommentKind::Block && PURE_COMMENTS.is_match(&comment.text)
        }));
      }
      None
    })
    .unwrap_or(false);
  if !pure_flag {
    !expr.may_have_side_effects(ExprCtx {
      ast: &parser.ast,
      semantic: &parser.semantic,
      in_strict: false,
      is_unresolved_ref_safe: false,
      remaining_depth: 4,
    })
  } else {
    call_expr.args(&parser.ast).iter().all(|arg| {
      let arg = parser.ast.get_node_in_sub_range(arg);
      if arg.spread(&parser.ast).is_some() {
        false
      } else {
        is_pure_expression(parser, arg.expr(&parser.ast), unresolved_scope_id, comments)
      }
    })
  }
}

impl SideEffectsParserPlugin {
  fn analyze_stmt_side_effects(&self, stmt: Statement, parser: &mut JavascriptParser) {
    if parser.side_effects_item.is_some() {
      return;
    }
    match stmt {
      Statement::If(if_stmt) => {
        if !is_pure_expression(
          parser,
          if_stmt.test(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            if_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::While(while_stmt) => {
        if !is_pure_expression(
          parser,
          while_stmt.test(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            while_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::DoWhile(do_while_stmt) => {
        if !is_pure_expression(
          parser,
          do_while_stmt.test(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            do_while_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::For(for_stmt) => {
        let pure_init = match for_stmt.init(&parser.ast) {
          Some(init) => match init {
            VarDeclOrExpr::VarDecl(decl) => {
              is_pure_var_decl(parser, decl, self.unresolved_scope_id, parser.comments)
            }
            VarDeclOrExpr::Expr(expr) => {
              is_pure_expression(parser, expr, self.unresolved_scope_id, parser.comments)
            }
          },
          None => true,
        };

        if !pure_init {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
          return;
        }

        let pure_test = match for_stmt.test(&parser.ast) {
          Some(test) => is_pure_expression(parser, test, self.unresolved_scope_id, parser.comments),
          None => true,
        };

        if !pure_test {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
          return;
        }

        let pure_update = match for_stmt.update(&parser.ast) {
          Some(expr) => is_pure_expression(parser, expr, self.unresolved_scope_id, parser.comments),
          None => true,
        };

        if !pure_update {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            for_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::Expr(expr_stmt) => {
        if !is_pure_expression(
          parser,
          expr_stmt.expr(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            expr_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::Switch(switch_stmt) => {
        if !is_pure_expression(
          parser,
          switch_stmt.discriminant(&parser.ast),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            switch_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::Class(class_stmt) => {
        if !is_pure_class(
          parser,
          class_stmt.class(),
          self.unresolved_scope_id,
          parser.comments,
        ) {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            class_stmt.span(&parser.ast),
            String::from("Statement"),
          ));
        }
      }
      Statement::Var(var_stmt) => match var_stmt {
        VariableDeclaration::VarDecl(var_decl) => {
          if !is_pure_var_decl(parser, var_decl, self.unresolved_scope_id, parser.comments) {
            parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
              var_stmt.span(&parser.ast),
              String::from("Statement"),
            ));
          }
        }
        VariableDeclaration::UsingDecl(_) => {
          parser.side_effects_item = Some(SideEffectsBailoutItemWithSpan::new(
            var_stmt.span(&parser.ast),
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
          stmt.span(&parser.ast),
          String::from("Statement"),
        ))
      }
    };
  }
}

pub fn is_pure_pat<'a>(
  parser: &mut JavascriptParser,
  pat: Pat,
  unresolved_scope_id: ScopeId,
  comments: Option<&'a dyn Comments>,
) -> bool {
  match pat {
    Pat::Ident(_) => true,
    Pat::Array(array_pat) => array_pat.elems(&parser.ast).iter().all(|ele| {
      let ele = parser.ast.get_node_in_sub_range(ele);
      if let Some(pat) = ele {
        is_pure_pat(parser, pat, unresolved_scope_id, comments)
      } else {
        true
      }
    }),
    Pat::Rest(_) => true,
    Pat::Invalid(_) | Pat::Assign(_) | Pat::Object(_) => false,
    Pat::Expr(expr) => is_pure_expression(parser, expr, unresolved_scope_id, comments),
  }
}

pub fn is_pure_function<'a>(
  parser: &mut JavascriptParser,
  function: Function,
  unresolved_scope_id: ScopeId,
  comments: Option<&'a dyn Comments>,
) -> bool {
  if !function.params(&parser.ast).iter().all(|param| {
    is_pure_pat(
      parser,
      parser.ast.get_node_in_sub_range(param).pat(&parser.ast),
      unresolved_scope_id,
      comments,
    )
  }) {
    return false;
  }

  true
}

pub fn is_pure_expression<'a>(
  parser: &mut JavascriptParser,
  expr: Expr,
  unresolved_scope_id: ScopeId,
  comments: Option<&'a dyn Comments>,
) -> bool {
  pub fn _is_pure_expression<'a>(
    parser: &mut JavascriptParser,
    expr: Expr,
    unresolved_scope_id: ScopeId,
    comments: Option<&'a dyn Comments>,
  ) -> bool {
    let drive = parser.plugin_drive.clone();
    if let Some(res) = drive.is_pure(parser, expr) {
      return res;
    }

    match expr {
      Expr::Call(_) => is_pure_call_expr(parser, expr, unresolved_scope_id, comments),
      Expr::Paren(_) => unreachable!(),
      Expr::Seq(seq_expr) => seq_expr.exprs(&parser.ast).iter().all(|expr| {
        is_pure_expression(
          parser,
          parser.ast.get_node_in_sub_range(expr),
          unresolved_scope_id,
          comments,
        )
      }),
      _ => !expr.may_have_side_effects(ExprCtx {
        ast: &parser.ast,
        semantic: &parser.semantic,
        is_unresolved_ref_safe: true,
        in_strict: false,
        remaining_depth: 4,
      }),
    }
  }
  _is_pure_expression(parser, expr, unresolved_scope_id, comments)
}

pub fn is_pure_class_member<'a>(
  parser: &mut JavascriptParser,
  member: ClassMember,
  unresolved_scope_id: ScopeId,
  comments: Option<&'a dyn Comments>,
) -> bool {
  let is_key_pure = match member.class_key(&parser.ast) {
    Some(PropName::Ident(_ident)) => true,
    Some(PropName::Str(_)) => true,
    Some(PropName::Num(_)) => true,
    Some(PropName::Computed(computed)) => is_pure_expression(
      parser,
      computed.expr(&parser.ast),
      unresolved_scope_id,
      comments,
    ),
    Some(PropName::BigInt(_)) => true,
    None => true,
  };
  if !is_key_pure {
    return false;
  }
  let is_static = member.is_static(&parser.ast);
  let is_value_pure = match member {
    ClassMember::Constructor(_) => true,
    ClassMember::Method(_) => true,
    ClassMember::PrivateMethod(_) => true,
    ClassMember::ClassProp(prop) => {
      if let Some(value) = prop.value(&parser.ast) {
        is_pure_expression(parser, value, unresolved_scope_id, comments)
      } else {
        true
      }
    }
    ClassMember::PrivateProp(prop) => {
      if let Some(value) = prop.value(&parser.ast) {
        is_pure_expression(parser, value, unresolved_scope_id, comments)
      } else {
        true
      }
    }
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
  stmt: Decl,
  unresolved_scope_id: ScopeId,
  comments: Option<&dyn Comments>,
) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(
      parser,
      class.class(&parser.ast),
      unresolved_scope_id,
      comments,
    ),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(parser, var, unresolved_scope_id, comments),
    Decl::Using(_) => false,
  }
}

pub fn is_pure_class(
  parser: &mut JavascriptParser,
  class: Class,
  unresolved_scope_id: ScopeId,
  comments: Option<&dyn Comments>,
) -> bool {
  if let Some(super_class) = class.super_class(&parser.ast)
    && !is_pure_expression(parser, super_class, unresolved_scope_id, comments)
  {
    return false;
  }
  let is_pure_key = |parser: &mut JavascriptParser, key: PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(computed) => is_pure_expression(
        parser,
        computed.expr(&parser.ast),
        unresolved_scope_id,
        comments,
      ),
    }
  };

  class.body(&parser.ast).iter().all(|item| -> bool {
    let item = parser.ast.get_node_in_sub_range(item);
    match item {
      ClassMember::Constructor(_) => class.super_class(&parser.ast).is_none(),
      ClassMember::Method(method) => is_pure_key(parser, method.key(&parser.ast)),
      ClassMember::PrivateMethod(method) => is_pure_expression(
        parser,
        Expr::PrivateName(method.key(&parser.ast)),
        unresolved_scope_id,
        comments,
      ),
      ClassMember::ClassProp(prop) => {
        is_pure_key(parser, prop.key(&parser.ast))
          && (!prop.is_static(&parser.ast)
            || if let Some(value) = prop.value(&parser.ast) {
              is_pure_expression(parser, value, unresolved_scope_id, comments)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(
          parser,
          Expr::PrivateName(prop.key(&parser.ast)),
          unresolved_scope_id,
          comments,
        ) && (!prop.is_static(&parser.ast)
          || if let Some(value) = prop.value(&parser.ast) {
            is_pure_expression(parser, value, unresolved_scope_id, comments)
          } else {
            true
          })
      }
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => false, // TODO: support is pure analyze for statements
      ClassMember::AutoAccessor(_) => false,
    }
  })
}

fn is_pure_var_decl<'a>(
  parser: &mut JavascriptParser,
  var: VarDecl,
  unresolved_scope_id: ScopeId,
  comments: Option<&'a dyn Comments>,
) -> bool {
  var.decls(&parser.ast).iter().all(|decl| {
    let decl = parser.ast.get_node_in_sub_range(decl);
    if let Some(init) = decl.init(&parser.ast) {
      is_pure_expression(parser, init, unresolved_scope_id, comments)
    } else {
      true
    }
  })
}
