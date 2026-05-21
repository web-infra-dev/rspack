use rspack_cacheable::cacheable;
use rspack_core::EvaluatedInlinableValue;
use rspack_util::ryu_js;
use swc_core::ecma::{
  ast::{ObjectPatProp, Pat, VarDeclarator},
  atoms::Atom,
};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{
    BasicEvaluatedExpression, evaluate_to_boolean, evaluate_to_null, evaluate_to_number,
    evaluate_to_string, evaluate_to_undefined,
  },
  visitors::{
    JavascriptParser, TagInfoData, VariableDeclaration, VariableDeclarationKind,
    scope_info::VariableInfoFlags,
  },
};

pub const INLINABLE_CONST_TAG: &str = "inlinable const";

#[derive(Debug, Clone)]
pub struct ConstValueData {
  pub value: ConstValue,
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum ConstValue {
  NoInlinable,
  Inlinable(EvaluatedInlinableValue),
}

impl ConstValue {
  pub fn as_inlinable(&self) -> Option<&EvaluatedInlinableValue> {
    match self {
      ConstValue::Inlinable(v) => Some(v),
      _ => None,
    }
  }
}

#[derive(Default)]
pub struct ConstValuePlugin {
  inline: bool,
}

impl ConstValuePlugin {
  pub fn new(inline: bool) -> Self {
    Self { inline }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for ConstValuePlugin {
  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name != INLINABLE_CONST_TAG {
      return None;
    }
    // Propagate inlinable constants. Help the rest const variable declarations that referencing the
    // inlinable constants to evaluate to an inlinable constants.
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = ConstValueData::downcast(tag_info.data.clone()?);
    let value = match data.value {
      ConstValue::NoInlinable => return None,
      ConstValue::Inlinable(v) => v,
    };
    Some(match value {
      EvaluatedInlinableValue::Null => evaluate_to_null(start, end),
      EvaluatedInlinableValue::Undefined => evaluate_to_undefined(start, end),
      EvaluatedInlinableValue::Boolean(v) => evaluate_to_boolean(v, start, end),
      EvaluatedInlinableValue::Number(v) => evaluate_to_number(v, start, end),
      EvaluatedInlinableValue::String(v) => evaluate_to_string(v.to_string(), start, end),
    })
  }

  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    declarator: &VarDeclarator,
    declaration: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if !parser.is_top_level_scope() {
      return None;
    }
    if !matches!(declaration.kind(), VariableDeclarationKind::Const) || declarator.init.is_none() {
      return None;
    }

    if let Some(name) = declarator.name.as_ident() {
      let const_value = if self.inline {
        let evaluated = parser.evaluate_expression(
          declarator
            .init
            .as_ref()
            .expect("init should exist for const value"),
        );
        match to_evaluated_inlinable_value(&evaluated) {
          Some(v) => ConstValue::Inlinable(v),
          None => ConstValue::NoInlinable,
        }
      } else {
        ConstValue::NoInlinable
      };
      tag_const_variable(parser, name.id.sym.clone(), const_value);
    } else {
      tag_const_pattern(parser, &declarator.name);
    }

    None
  }
}

fn tag_const_variable(parser: &mut JavascriptParser, name: Atom, value: ConstValue) {
  parser.tag_variable_with_flags(
    name,
    INLINABLE_CONST_TAG,
    Some(ConstValueData { value }),
    VariableInfoFlags::NORMAL,
  );
}

fn tag_const_pattern(parser: &mut JavascriptParser, pattern: &Pat) {
  match pattern {
    Pat::Ident(ident) => {
      tag_const_variable(parser, ident.id.sym.clone(), ConstValue::NoInlinable);
    }
    Pat::Array(array) => {
      for elem in array.elems.iter().flatten() {
        tag_const_pattern(parser, elem);
      }
    }
    Pat::Assign(assign) => {
      tag_const_pattern(parser, &assign.left);
    }
    Pat::Object(object) => {
      for prop in &object.props {
        match prop {
          ObjectPatProp::KeyValue(prop) => tag_const_pattern(parser, &prop.value),
          ObjectPatProp::Assign(prop) => {
            tag_const_variable(parser, prop.key.sym.clone(), ConstValue::NoInlinable);
          }
          ObjectPatProp::Rest(rest) => tag_const_pattern(parser, &rest.arg),
        }
      }
    }
    Pat::Rest(rest) => tag_const_pattern(parser, &rest.arg),
    Pat::Invalid(_) | Pat::Expr(_) => {}
  }
}

fn to_evaluated_inlinable_value(
  evaluated: &BasicEvaluatedExpression,
) -> Option<EvaluatedInlinableValue> {
  if evaluated.is_bool() {
    Some(EvaluatedInlinableValue::new_boolean(evaluated.bool()))
  } else if evaluated.is_number()
    && let num = evaluated.number()
    && ryu_js::Buffer::new().format(num).len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_number(num))
  } else if evaluated.is_string()
    && let str = evaluated.string()
    && str.len() <= EvaluatedInlinableValue::SHORT_SIZE
  {
    Some(EvaluatedInlinableValue::new_string(str.as_str().into()))
  } else if evaluated.is_null() {
    Some(EvaluatedInlinableValue::new_null())
  } else if evaluated.is_undefined() {
    Some(EvaluatedInlinableValue::new_undefined())
  } else {
    None
  }
}
