use rspack_util::SpanExt;
use swc_next_ecma_ast::{GetSpan, TaggedTemplateExpression, TemplateLiteral};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[derive(Debug, Clone, Copy)]
pub enum TemplateStringKind {
  Cooked,
  Raw,
}

fn get_simplified_template_result<'parser>(
  parser: &mut JavascriptParser<'parser>,
  kind: TemplateStringKind,
  template: TemplateLiteral,
) -> (
  Vec<BasicEvaluatedExpression<'parser>>,
  Vec<BasicEvaluatedExpression<'parser>>,
) {
  let ast = parser.ast.ast;
  let quasis_nodes = template.quasis(ast);
  let expressions = template.expressions(ast);
  let mut quasis: Vec<BasicEvaluatedExpression<'parser>> = Vec::new();
  let mut parts: Vec<BasicEvaluatedExpression<'parser>> = Vec::new();
  for (index, quasi_node) in quasis_nodes
    .iter()
    .map(|id| ast.get_node_in_sub_range(id))
    .enumerate()
  {
    let quasi = match kind {
      TemplateStringKind::Cooked if !quasi_node.is_cooked_undefined(ast) => ast
        .get_wtf8(quasi_node.cooked(ast))
        .to_string_lossy()
        .into_owned(),
      TemplateStringKind::Cooked | TemplateStringKind::Raw => {
        ast.get_utf8(quasi_node.raw(ast)).to_string()
      }
    };
    let quasi_span = quasi_node.span(ast);
    if index > 0 {
      let previous = parts.last_mut().expect("template has a preceding quasi");
      let expression = parser.evaluate_expression(
        expressions
          .get_node(ast, index - 1)
          .expect("template has an expression before each non-leading quasi"),
      );
      if !expression.could_have_side_effects()
        && let Some(value) = expression.as_string()
      {
        previous.set_string(format!("{}{}{}", previous.string(), value, quasi));
        previous.set_range(previous.range().0, quasi_span.real_hi());
        previous.set_expression(None);

        let previous_quasi = quasis.last_mut().expect("template has a preceding quasi");
        previous_quasi.set_string(format!("{}{}{}", previous_quasi.string(), value, quasi));
        previous_quasi.set_range(previous_quasi.range().0, quasi_span.real_hi());
        previous_quasi.set_expression(None);
        continue;
      }
      parts.push(expression);
    }
    let part = || {
      let mut part = BasicEvaluatedExpression::new();
      part.set_string(quasi.clone());
      part.set_range(quasi_span.real_lo(), quasi_span.real_hi());
      part
    };
    quasis.push(part());
    parts.push(part());
  }
  (quasis, parts)
}

#[inline]
pub fn eval_tpl_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  template: TemplateLiteral,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let kind = TemplateStringKind::Cooked;
  let (quasis, mut parts) = get_simplified_template_result(parser, kind, template);
  let span = template.span(parser.ast.ast);
  if parts.len() == 1 {
    let mut part = parts.remove(0);
    part.set_range(span.real_lo(), span.real_hi());
    Some(part)
  } else {
    let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
    result.set_template_string(quasis, parts, kind);
    Some(result)
  }
}

#[inline]
pub fn eval_tagged_tpl_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  tagged: TaggedTemplateExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let tag = parser.evaluate_expression(tagged.tag(ast));
  if !tag.is_identifier() || tag.identifier() != "String.raw" {
    return None;
  }
  let kind = TemplateStringKind::Raw;
  let (quasis, parts) = get_simplified_template_result(parser, kind, tagged.quasi(ast));
  let span = tagged.span(ast);
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  result.set_template_string(quasis, parts, kind);
  Some(result)
}
