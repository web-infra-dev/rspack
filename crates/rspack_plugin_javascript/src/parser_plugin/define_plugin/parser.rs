use std::{borrow::Cow, sync::Arc};

use rspack_core::SpanExt as _;
use swc_core::common::Spanned as _;

use super::{VALUE_DEP_PREFIX, utils::gen_const_dep, walk_data::WalkData};
use crate::{
  JavascriptParserPlugin,
  utils::eval::{BasicEvaluatedExpression, evaluate_to_string},
  visitors::JavascriptParser,
};

pub struct DefineParserPlugin {
  pub walk_data: Arc<WalkData>,
}

impl DefineParserPlugin {
  fn add_value_dependency(&self, parser: &mut JavascriptParser, key: &str) {
    let key = format!("{VALUE_DEP_PREFIX}{key}");
    if let Some(value) = self.walk_data.value_cache_versions.get(&key) {
      parser
        .build_info
        .value_dependencies
        .insert(key, value.clone());
    }
  }
}

impl JavascriptParserPlugin for DefineParserPlugin {
  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    if self.walk_data.can_rename.contains(str) {
      self.add_value_dependency(parser, str);
      return Some(true);
    }
    None
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_evaluate_typeof) = &record.on_evaluate_typeof
    {
      self.add_value_dependency(parser, for_name);
      return on_evaluate_typeof(record, parser, expr.span.real_lo(), expr.span.hi.0);
    } else if self.walk_data.object_define_record.contains_key(for_name) {
      self.add_value_dependency(parser, for_name);
      return Some(evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.hi.0,
      ));
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'static>> {
    if let Some(record) = self.walk_data.define_record.get(ident)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      self.add_value_dependency(parser, ident);
      return on_evaluate_identifier(record, parser, ident, start, end);
    } else if let Some(record) = self.walk_data.object_define_record.get(ident)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      self.add_value_dependency(parser, ident);
      return on_evaluate_identifier(record, parser, ident, start, end);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_typeof) = &record.on_typeof
    {
      self.add_value_dependency(parser, for_name);
      return on_typeof(record, parser, expr.span.real_lo(), expr.span.real_hi());
    } else if self.walk_data.object_define_record.contains_key(for_name) {
      self.add_value_dependency(parser, for_name);
      debug_assert!(!parser.in_short_hand);
      parser
        .presentational_dependencies
        .push(Box::new(gen_const_dep(
          parser,
          Cow::Borrowed(r#""object""#),
          for_name,
          expr.span.real_lo(),
          expr.span.real_hi(),
        )));
      return Some(true);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        expr.span,
        expr.callee.span().real_lo(),
        expr.callee.span().real_hi(),
        for_name,
      )
      .map(|_| {
        // FIXME: webpack use `walk_expression` here
        parser.walk_expr_or_spread(&expr.args);
        true
      });
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        expr.span,
        expr.callee.span().real_lo(),
        expr.callee.span().real_hi(),
        for_name,
      )
      .map(|_| {
        // FIXME: webpack use `walk_expression` here
        parser.walk_expr_or_spread(&expr.args);
        true
      });
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        expr.span,
        expr.span.real_lo(),
        expr.span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        expr.span,
        expr.span.real_lo(),
        expr.span.real_hi(),
        for_name,
      );
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.walk_data.define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        ident.span,
        ident.span.real_lo(),
        ident.span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      return on_expression(
        record,
        parser,
        ident.span,
        ident.span.real_lo(),
        ident.span.real_hi(),
        for_name,
      );
    }
    None
  }
}
