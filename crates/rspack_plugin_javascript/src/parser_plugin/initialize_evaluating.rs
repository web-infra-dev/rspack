use rspack_core::SpanExt;

use super::JavascriptParserPlugin;
use crate::utils::eval::BasicEvaluatedExpression;

const SLICE_METHOD_NAME: &str = "slice";
const REPLACE_METHOD_NAME: &str = "replace";
const CONCAT_METHOD_NAME: &str = "concat";

pub struct InitializeEvaluating;

impl JavascriptParserPlugin for InitializeEvaluating {
  fn evaluate_call_expression_member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    property: &str,
    expr: &swc_core::ecma::ast::CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<BasicEvaluatedExpression> {
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
    } else if property == CONCAT_METHOD_NAME && (param.is_string() || param.is_wrapped()) {
      let mut string_suffix: Option<BasicEvaluatedExpression> = None;
      let mut has_unknown_params = false;
      let mut inner_exprs = Vec::new();
      for arg in expr.args.iter().rev() {
        if arg.spread.is_some() {
          return None;
        }
        let arg_expr = parser.evaluate_expression(&arg.expr);
        if has_unknown_params || (!arg_expr.is_string() && !arg_expr.is_number()) {
          has_unknown_params = true;
          inner_exprs.push(arg_expr);
          continue;
        }
        let mut new_string = if arg_expr.is_string() {
          arg_expr.string().to_owned()
        } else {
          format!("{}", arg_expr.number())
        };
        if let Some(string_suffix) = &string_suffix {
          new_string += string_suffix.string();
        }
        let mut eval = BasicEvaluatedExpression::with_range(
          arg_expr.range().0,
          string_suffix.as_ref().unwrap_or(&arg_expr).range().1,
        );
        eval.set_string(new_string);
        eval.set_side_effects(
          string_suffix
            .as_ref()
            .map(|s| s.could_have_side_effects())
            .unwrap_or_else(|| arg_expr.could_have_side_effects()),
        );
        string_suffix = Some(eval);
      }
      if has_unknown_params {
        let prefix = if param.is_string() {
          Some(param)
        } else {
          param.prefix()
        }
        .cloned();
        inner_exprs.reverse();
        let inner = if param.is_wrapped()
          && let Some(wrapped_inner_expressions) = param.wrapped_inner_expressions()
        {
          let mut wrapped_inner_exprs = wrapped_inner_expressions.to_vec();
          wrapped_inner_exprs.extend(inner_exprs);
          wrapped_inner_exprs
        } else {
          inner_exprs
        };
        let mut eval = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
        eval.set_wrapped(prefix, string_suffix, inner);
        return Some(eval);
      } else if param.is_wrapped() {
        let postfix = string_suffix.or_else(|| param.postfix().cloned());
        let inner = if param.is_wrapped()
          && let Some(wrapped_inner_expressions) = param.wrapped_inner_expressions()
        {
          let mut wrapped_inner_exprs = wrapped_inner_expressions.to_vec();
          wrapped_inner_exprs.extend(inner_exprs);
          wrapped_inner_exprs
        } else {
          inner_exprs
        };
        let mut eval = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
        eval.set_wrapped(param.prefix().cloned(), postfix, inner);
        return Some(eval);
      } else {
        let mut new_string = param.string().to_owned();
        if let Some(string_suffix) = &string_suffix {
          new_string += string_suffix.string();
        }
        let mut eval = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
        eval.set_string(new_string);
        eval.set_side_effects(
          string_suffix
            .as_ref()
            .map(|s| s.could_have_side_effects())
            .unwrap_or_else(|| param.could_have_side_effects()),
        );
        return Some(eval);
      }
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

fn mock_javascript_slice(str: &str, number: f64) -> String {
  if number == f64::INFINITY {
    String::new()
  } else if number == f64::NEG_INFINITY || number.is_nan() {
    str.to_string()
  } else {
    let n = number.trunc() as isize;
    if n >= str.len() as isize {
      String::new()
    } else if n >= 0 {
      str[n as usize..].to_string()
    } else if n.unsigned_abs() >= str.len() {
      str.to_string()
    } else {
      str[str.len() - (n.unsigned_abs())..].to_string()
    }
  }
}

#[test]
fn test_mock_javascript_slice() {
  assert_eq!(mock_javascript_slice("123", 0.), "123".to_string());
  assert_eq!(mock_javascript_slice("123", 0.1), "123".to_string());
  assert_eq!(mock_javascript_slice("123", 1.1), "23".to_string());
  assert_eq!(mock_javascript_slice("123", f64::INFINITY), String::new());
  assert_eq!(mock_javascript_slice("123", f64::NAN), "123".to_string());
  assert_eq!(mock_javascript_slice("123", 3.), String::new());
  assert_eq!(mock_javascript_slice("123", -0.), "123".to_string());
  assert_eq!(
    mock_javascript_slice("123", f64::NEG_INFINITY),
    "123".to_string()
  );
  assert_eq!(mock_javascript_slice("123", -1.), "3".to_string());
  assert_eq!(mock_javascript_slice("123", -2.2), "23".to_string());
  assert_eq!(mock_javascript_slice("123", -3.), "123".to_string());
}
