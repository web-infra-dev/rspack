use rspack_core::SpanExt;

use super::JavascriptParserPlugin;
use crate::utils::eval::BasicEvaluatedExpression;

const SLICE_METHOD_NAME: &str = "slice";
const REPLACE_METHOD_NAME: &str = "replace";

fn mock_javascript_slice(str: &str, number: f64) -> String {
  if number == f64::INFINITY {
    String::new()
  } else if number == f64::NEG_INFINITY || number.is_nan() {
    str.to_string()
  } else {
    let n = number.trunc() as isize;
    if n >= 0 {
      str[n as usize..].to_string()
    } else {
      str[str.len() - ((-n) as usize)..].to_string()
    }
  }
}

pub struct InitializeEvaluating;

impl JavascriptParserPlugin for InitializeEvaluating {
  fn evaluate_call_expression_member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    property: &str,
    expr: &swc_core::ecma::ast::CallExpr,
    param: &crate::utils::eval::BasicEvaluatedExpression,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if property == SLICE_METHOD_NAME
      && param.is_string()
      && expr.args.len() == 1
      && expr.args[0].spread.is_none()
    {
      // TODO: expr.args.len == 2
      let arg1 = parser.evaluate_expression(&expr.args[0].expr);
      if arg1.is_number() {
        let result = mock_javascript_slice(param.string().as_str(), arg1.number());
        let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
        res.set_string(result);
        res.set_side_effects(param.could_have_side_effects());
        return Some(res);
      }
    } else if property == REPLACE_METHOD_NAME
      && param.is_string()
      && expr.args.len() == 2
      && expr.args[0].spread.is_none()
      && expr.args[1].spread.is_none()
    {
      let arg1 = parser.evaluate_expression(&expr.args[0].expr);
      if !arg1.is_string() && !arg1.is_regexp() {
        return None;
      }
      let arg2 = parser.evaluate_expression(&expr.args[1].expr);
      if !arg2.is_string() {
        return None;
      }
      let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
      // mock js replace
      let s = if arg1.is_string() {
        param
          .string()
          .replacen(arg1.string(), arg2.string().as_str(), 1)
      } else if arg1.regexp().1.contains('g') {
        let raw = arg1.regexp();
        let regexp = eval_regexp_to_regexp(&raw.0, &raw.1);
        regexp
          .replace_all(param.string().as_ref(), arg2.string())
          .to_string()
      } else {
        let raw = arg1.regexp();
        let regexp = eval_regexp_to_regexp(&raw.0, &raw.1);
        regexp
          .replace(param.string().as_ref(), arg2.string())
          .to_string()
      };
      res.set_string(s);
      res.set_side_effects(param.could_have_side_effects());
      return Some(res);
    }

    None
  }
}

fn eval_regexp_to_regexp(expr: &str, flags: &str) -> regex::Regex {
  let mut re = String::new();
  for ch in flags.chars() {
    match ch {
      'i' => re.push('i'),
      'm' => re.push('m'),
      's' => re.push('s'),
      'x' => re.push('x'),
      _ => (),
    }
  }
  let re = if re.is_empty() {
    expr.to_string()
  } else {
    format!("(?{re}){expr}")
  };
  regex::Regex::new(&re).expect("should an valid regexp")
}
