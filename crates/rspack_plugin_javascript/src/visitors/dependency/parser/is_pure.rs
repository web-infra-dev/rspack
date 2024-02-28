use once_cell::sync::Lazy;
use swc_core::common::comments::CommentKind;
use swc_core::common::{BytePos, Spanned};
use swc_core::ecma::ast::{Class, ClassMember, Decl, Expr, Ident, Stmt, VarDecl};

use super::JavascriptParser;
use crate::parser_plugin::{is_logic_op, JavascriptParserPlugin};

impl JavascriptParser<'_> {
  pub fn is_pure_stmt(&mut self, stmt: &Stmt, comments_start_pos: BytePos) -> bool {
    match stmt {
      Stmt::Decl(decl) => return self.is_pure_decl(decl, comments_start_pos),
      Stmt::Expr(expr) => return self.is_pure_expr(&expr.expr, comments_start_pos),
      Stmt::Block(_)
      | Stmt::Empty(_)
      | Stmt::Debugger(_)
      | Stmt::With(_)
      | Stmt::Return(_)
      | Stmt::Labeled(_)
      | Stmt::Break(_)
      | Stmt::Continue(_)
      | Stmt::If(_)
      | Stmt::Switch(_)
      | Stmt::Throw(_)
      | Stmt::Try(_)
      | Stmt::While(_)
      | Stmt::DoWhile(_)
      | Stmt::For(_)
      | Stmt::ForIn(_)
      | Stmt::ForOf(_) => (),
    };
    true
  }

  pub fn is_pure_decl(&mut self, decl: &Decl, comments_start_pos: BytePos) -> bool {
    match decl {
      Decl::Class(class_decl) => self.is_pure_class(&class_decl.class, comments_start_pos),
      Decl::Var(var) => self.is_pure_var_decl(var),
      Decl::Fn(_) | Decl::Using(_) => true,
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!()
      }
    }
  }

  pub fn is_pure_expr(&mut self, expr: &Expr, comments_start_pos: BytePos) -> bool {
    match expr {
      Expr::Bin(bin) if is_logic_op(bin.op) => {
        return self.is_pure_expr(&bin.left, comments_start_pos)
          && self.is_pure_expr(&bin.right, bin.left.span().hi())
      }

      Expr::Cond(condition) => {
        return self.is_pure_expr(&condition.test, comments_start_pos)
          && self.is_pure_expr(&condition.cons, condition.test.span().hi)
          && self.is_pure_expr(&condition.alt, condition.cons.span().hi)
      }
      Expr::Call(call) => {
        let pure_flag = (call.span.lo.0 - comments_start_pos.0 > 12)
          .then(|| self.has_pure_flag_comment(call.span.lo))
          .unwrap_or_default();
        let res = if !pure_flag {
          false
        } else if call.args.is_empty() {
          true
        } else {
          let mut comments_start_pos = call.callee.span().hi;
          call.args.iter().all(|arg| {
            if arg.spread.is_some() {
              false
            } else {
              let pure_flag = self.is_pure_expr(&arg.expr, comments_start_pos);
              comments_start_pos = arg.span().hi;
              pure_flag
            }
          })
        };
        return res;
      }
      Expr::Seq(seq) => {
        let mut comments_start_pos = comments_start_pos;
        return seq.exprs.iter().all(|expr| {
          let pure_flag = self.is_pure_expr(expr, comments_start_pos);
          comments_start_pos = expr.span().hi();
          pure_flag
        });
      }
      Expr::Class(class_expr) => return self.is_pure_class(&class_expr.class, comments_start_pos),
      Expr::Paren(paren_expr) => return self.is_pure_expr(&paren_expr.expr, comments_start_pos),
      Expr::Ident(ident) => return self.is_pure_identifier(ident),
      Expr::Lit(_) | Expr::Tpl(_) | Expr::This(_) | Expr::Fn(_) | Expr::Arrow(_) => return true,
      Expr::Bin(_)
      | Expr::Assign(_)
      | Expr::Member(_)
      | Expr::SuperProp(_)
      | Expr::New(_)
      | Expr::TaggedTpl(_)
      | Expr::Yield(_)
      | Expr::MetaProp(_)
      | Expr::Await(_)
      | Expr::Array(_)
      | Expr::Object(_)
      | Expr::Unary(_)
      | Expr::Update(_)
      | Expr::PrivateName(_)
      | Expr::OptChain(_)
      | Expr::Invalid(_) => (),
      Expr::JSXMember(_)
      | Expr::JSXNamespacedName(_)
      | Expr::JSXEmpty(_)
      | Expr::JSXElement(_)
      | Expr::JSXFragment(_)
      | Expr::TsTypeAssertion(_)
      | Expr::TsConstAssertion(_)
      | Expr::TsNonNull(_)
      | Expr::TsAs(_)
      | Expr::TsInstantiation(_)
      | Expr::TsSatisfies(_) => unreachable!(),
    };
    let evaluated = self.evaluate_expression(expr);
    !evaluated.could_have_side_effects()
  }

  fn is_pure_identifier(&mut self, ident: &Ident) -> bool {
    if let Some(result) = self.plugin_drive.clone().is_pure_identifier(self, ident) {
      result
    } else {
      true
    }
  }

  fn is_pure_var_decl(&mut self, var: &VarDecl) -> bool {
    var.decls.iter().all(|decl| {
      if let Some(ref init) = decl.init {
        self.is_pure_expr(init, decl.span.lo)
      } else {
        true
      }
    })
  }

  pub fn is_pure_class(&mut self, class: &Class, comments_start_pos: BytePos) -> bool {
    if let Some(ref super_class) = class.super_class
      && !self.is_pure_expr(super_class, comments_start_pos)
    {
      return false;
    }

    class.body.iter().all(|item| match item {
      ClassMember::Constructor(_) => class.super_class.is_none(),
      ClassMember::Method(method) => !method
        .key
        .as_computed()
        .map(|key| !self.is_pure_expr(&key.expr, method.span.lo))
        .unwrap_or_default(),
      ClassMember::PrivateMethod(_) => true,
      ClassMember::ClassProp(prop) => {
        if prop
          .key
          .as_computed()
          .map(|key| !self.is_pure_expr(&key.expr, prop.span.lo))
          .unwrap_or_default()
        {
          false
        } else {
          !(prop.is_static
            && prop
              .value
              .as_ref()
              .map(|value| !self.is_pure_expr(value, prop.key.span().lo))
              .unwrap_or_default())
        }
      }
      ClassMember::PrivateProp(prop) => {
        !(prop.is_static
          && prop
            .value
            .as_ref()
            .map(|value| !self.is_pure_expr(value, prop.key.span.lo))
            .unwrap_or_default())
      }
      ClassMember::StaticBlock(_) => false,
      ClassMember::Empty(_) | ClassMember::AutoAccessor(_) => true,
      ClassMember::TsIndexSignature(_) => unreachable!(),
    })
  }

  fn has_pure_flag_comment(&self, lo: BytePos) -> bool {
    self
      .comments
      .and_then(|comments| {
        let list = comments.leading.get(&lo)?;
        let last_comment = list.last()?;
        match last_comment.kind {
          CommentKind::Line => None,
          CommentKind::Block => Some(PURE_COMMENTS.is_match(&last_comment.text)),
        }
      })
      .unwrap_or_default()
  }
}

static PURE_COMMENTS: Lazy<regex::Regex> =
  Lazy::new(|| regex::Regex::new("^\\s*(#|@)__PURE__\\s*$").expect("Should create the regex"));
