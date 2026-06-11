use std::{
  borrow::Cow,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{Expr, Ident, MemberExpr, UnaryExpr};

use super::{VALUE_DEP_PREFIX, utils::gen_const_dep, walk_data::WalkData};
use crate::{
  JavascriptParserPlugin,
  define_plugin::walk_data::DefineRecord,
  utils::eval::{BasicEvaluatedExpression, evaluate_to_string},
  visitors::{AllowedMemberTypes, JavascriptParser, MemberExpressionInfo},
};

pub struct DefineParserPlugin {
  recurse: AtomicBool,
  recurse_typeof: AtomicBool,
  walk_data: Arc<WalkData>,
}

impl DefineParserPlugin {
  pub fn new(walk_data: Arc<WalkData>) -> Self {
    Self {
      recurse: AtomicBool::new(false),
      recurse_typeof: AtomicBool::new(false),
      walk_data,
    }
  }

  fn add_value_dependency(&self, parser: &mut JavascriptParser, key: &str) {
    if let Some(value) = self.walk_data.tiling_definitions.get(key) {
      let cache_key = format!("{VALUE_DEP_PREFIX}{key}");
      parser
        .build_info
        .value_dependencies
        .insert(cache_key, value.clone());
    }
  }

  fn get_define_record(&self, for_name: &str) -> Option<&DefineRecord> {
    self
      .walk_data
      .define_record
      .get(for_name)
      .or_else(|| self.walk_data.typeof_define_record.get(for_name))
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for DefineParserPlugin {
  fn can_rename(&self, parser: &mut JavascriptParser<'p>, str: &str) -> Option<bool> {
    if let Some(first_key) = self.walk_data.can_rename.get(str) {
      self.add_value_dependency(parser, str);
      if let Some(first_key) = first_key
        && let Some(info) = parser.get_variable_info(&first_key.as_ref().into())
        && !info.is_free()
      {
        return Some(false);
      }
      return Some(true);
    }
    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr<'a>,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>>
  where
    'p: 'a,
  {
    if let Some(record) = self.get_define_record(for_name)
      && let Some(on_evaluate_typeof) = &record.on_evaluate_typeof
    {
      // Avoid endless recursion, for example: new DefinePlugin({ "typeof a": "typeof a" })
      if self.recurse_typeof.swap(true, Ordering::Acquire) {
        return None;
      }
      self.add_value_dependency(parser, for_name);
      let evaluated = on_evaluate_typeof(record, parser, expr.span.real_lo(), expr.span.real_hi());
      self.recurse_typeof.store(false, Ordering::Release);
      return evaluated;
    }
    if self.walk_data.object_define_record.contains_key(for_name) {
      self.add_value_dependency(parser, for_name);
      return Some(evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
    if let Some(record) = self.get_define_record(for_name)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      // Avoid endless recursion, for example: new DefinePlugin({ "a": "a" })
      if self.recurse.swap(true, Ordering::Acquire) {
        return None;
      }
      self.add_value_dependency(parser, for_name);
      let evaluated = on_evaluate_identifier(record, parser, for_name, start, end);
      self.recurse.store(false, Ordering::Release);
      return evaluated;
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_evaluate_identifier) = &record.on_evaluate_identifier
    {
      self.add_value_dependency(parser, for_name);
      return on_evaluate_identifier(record, parser, for_name, start, end);
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &UnaryExpr<'_>,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.get_define_record(for_name)
      && let Some(on_typeof) = &record.on_typeof
    {
      self.add_value_dependency(parser, for_name);
      return on_typeof(record, parser, expr.span.real_lo(), expr.span.real_hi());
    } else if self.walk_data.object_define_record.contains_key(for_name) {
      self.add_value_dependency(parser, for_name);
      debug_assert!(!parser.in_short_hand);
      for dep in gen_const_dep(
        parser,
        Cow::Borrowed(r#""object""#),
        for_name,
        expr.span.real_lo(),
        expr.span.real_hi(),
      ) {
        parser.add_presentational_dependency(dep);
      }

      return Some(true);
    }
    None
  }

  fn can_collect_destructuring_assignment_properties(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &Expr,
  ) -> Option<bool> {
    if let MemberExpressionInfo::Expression(info) =
      parser.get_member_expression_info_from_expr(expr, AllowedMemberTypes::Expression)?
      && (self
        .walk_data
        .define_record
        .contains_key(info.name.as_str())
        || self
          .walk_data
          .object_define_record
          .contains_key(info.name.as_str()))
    {
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &MemberExpr<'_>,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.get_define_record(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      let span = expr.span;
      return on_expression(
        record,
        parser,
        span,
        span.real_lo(),
        span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      let span = expr.span;
      return on_expression(
        record,
        parser,
        span,
        span.real_lo(),
        span.real_hi(),
        for_name,
      );
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Ident<'_>,
    for_name: &str,
  ) -> Option<bool> {
    if let Some(record) = self.get_define_record(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      let span = ident.span;
      return on_expression(
        record,
        parser,
        span,
        span.real_lo(),
        span.real_hi(),
        for_name,
      );
    } else if let Some(record) = self.walk_data.object_define_record.get(for_name)
      && let Some(on_expression) = &record.on_expression
    {
      self.add_value_dependency(parser, for_name);
      let span = ident.span;
      return on_expression(
        record,
        parser,
        span,
        span.real_lo(),
        span.real_hi(),
        for_name,
      );
    }
    None
  }
}
