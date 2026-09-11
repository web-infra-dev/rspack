use rspack_util::{SpanExt, swc::AstSubRangeExt};
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
) -> Vec<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let quasis_nodes = template.quasis(ast);
  let mut expressions = ast.nodes(template.expressions(ast));
  let mut parts: Vec<BasicEvaluatedExpression<'parser>> = Vec::new();
  for (index, quasi_node) in ast.nodes(quasis_nodes).enumerate() {
    let quasi = match kind {
      TemplateStringKind::Cooked if !quasi_node.is_cooked_undefined(ast) => {
        ast.get_wtf8(quasi_node.cooked(ast)).to_string_lossy()
      }
      TemplateStringKind::Cooked | TemplateStringKind::Raw => {
        ast.get_utf8(quasi_node.raw(ast)).into()
      }
    };
    let quasi_span = quasi_node.span(ast);
    if index > 0 {
      let previous = parts.last_mut().expect("template has a preceding quasi");
      let expression = parser.evaluate_expression(
        expressions
          .next()
          .expect("template has an expression before each non-leading quasi"),
      );
      if !expression.could_have_side_effects()
        && let Some(value) = expression.as_string()
      {
        previous.set_string(format!("{}{}{}", previous.string(), value, quasi));
        previous.set_range(previous.range().0, quasi_span.real_hi());
        previous.set_expression(None);
        continue;
      }
      parts.push(expression);
    }
    let mut part = BasicEvaluatedExpression::new();
    part.set_string(quasi);
    part.set_range(quasi_span.real_lo(), quasi_span.real_hi());
    parts.push(part);
  }
  parts
}

#[inline]
pub fn eval_tpl_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  template: TemplateLiteral,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let kind = TemplateStringKind::Cooked;
  let mut parts = get_simplified_template_result(parser, kind, template);
  let span = template.span(parser.ast.ast);
  if parts.len() == 1 {
    let mut part = parts.remove(0);
    part.set_range(span.real_lo(), span.real_hi());
    Some(part)
  } else {
    let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
    result.set_template_string(parts, kind);
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
  let parts = get_simplified_template_result(parser, kind, tagged.quasi(ast));
  let span = tagged.span(ast);
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  result.set_template_string(parts, kind);
  Some(result)
}
