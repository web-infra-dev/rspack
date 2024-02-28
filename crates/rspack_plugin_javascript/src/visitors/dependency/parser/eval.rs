use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::Expr;

use super::{AllowedMemberTypes, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo};
use crate::parser_plugin::JavascriptParserPlugin;
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::scope_info::FreeName;

impl JavascriptParser<'_> {
  pub fn evaluate_expression(&mut self, expr: &Expr) -> BasicEvaluatedExpression {
    match self.evaluating(expr) {
      Some(evaluated) => evaluated,
      None => BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0),
    }
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating(&mut self, expr: &Expr) -> Option<BasicEvaluatedExpression> {
    match expr {
      Expr::Tpl(tpl) => eval::eval_tpl_expression(self, tpl),
      Expr::Lit(lit) => eval::eval_lit_expr(lit),
      Expr::Cond(cond) => eval::eval_cond_expression(self, cond),
      Expr::Unary(unary) => eval::eval_unary_expression(self, unary),
      Expr::Bin(binary) => eval::eval_binary_expression(self, binary),
      Expr::Array(array) => eval::eval_array_expression(self, array),
      Expr::New(new) => eval::eval_new_expression(self, new),
      Expr::Call(call) => eval::eval_call_expression(self, call),
      Expr::Paren(paren) => self.evaluating(&paren.expr),
      Expr::Member(member) => {
        if let Some(MemberExpressionInfo::Expression(info)) =
          self.get_member_expression_info(member, AllowedMemberTypes::Expression)
        {
          self
            .plugin_drive
            .clone()
            .evaluate_identifier(self, &info.name, member.span.real_lo(), member.span.hi().0)
            .or_else(|| {
              // TODO: fallback with `evaluateDefinedIdentifier`
              let mut eval =
                BasicEvaluatedExpression::with_range(member.span.real_lo(), member.span.hi().0);
              eval.set_identifier(info.name, info.root_info);
              Some(eval)
            })
        } else {
          None
        }
      }
      Expr::Ident(ident) => {
        let drive = self.plugin_drive.clone();
        let Some(info) = self.get_variable_info(&ident.sym) else {
          // use `ident.sym` as fallback for global variable(or maybe just a undefined variable)
          return drive
            .evaluate_identifier(
              self,
              ident.sym.as_str(),
              ident.span.real_lo(),
              ident.span.hi.0,
            )
            .or_else(|| {
              let mut eval =
                BasicEvaluatedExpression::with_range(ident.span.real_lo(), ident.span.hi.0);

              if ident.sym.eq("undefined") {
                eval.set_undefined();
              } else {
                eval.set_identifier(
                  ident.sym.to_string(),
                  ExportedVariableInfo::Name(ident.sym.to_string()),
                );
              }

              Some(eval)
            });
        };
        if let Some(FreeName::String(name)) = info.free_name.as_ref() {
          // avoid ownership
          let name = name.to_string();
          return drive.evaluate_identifier(self, &name, ident.span.real_lo(), ident.span.hi.0);
        }
        None
      }
      _ => None,
    }
  }
}
