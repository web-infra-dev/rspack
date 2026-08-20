use std::borrow::Cow;

use itertools::Itertools as _;
use rspack_core::{
  BoxDependencyTemplate, ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency,
};
use serde_json::{Value, json};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{EsVersion, Expr, Prop, PropName, PropOrSpread};
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_expr};

use crate::visitors::{DestructuringAssignmentProperties, JavascriptParser};

fn is_compile_time_expression(expr: &Expr<'_>) -> bool {
  match expr {
    Expr::Lit(_) => true,
    Expr::Unary(expr) => is_compile_time_expression(&expr.arg),
    Expr::Bin(expr) => {
      is_compile_time_expression(&expr.left) && is_compile_time_expression(&expr.right)
    }
    Expr::Cond(expr) => {
      is_compile_time_expression(&expr.test)
        && is_compile_time_expression(&expr.cons)
        && is_compile_time_expression(&expr.alt)
    }
    Expr::Array(expr) => expr.elems.iter().all(|element| {
      element
        .as_ref()
        .is_none_or(|element| element.spread.is_none() && is_compile_time_expression(&element.expr))
    }),
    Expr::Object(expr) => expr.props.iter().all(|prop| match prop {
      PropOrSpread::Prop(prop) => match &**prop {
        Prop::KeyValue(prop) => {
          let key_is_compile_time = match &prop.key {
            PropName::Computed(computed) => is_compile_time_expression(&computed.expr),
            PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) | PropName::BigInt(_) => true,
          };
          key_is_compile_time && is_compile_time_expression(&prop.value)
        }
        Prop::Shorthand(_)
        | Prop::Assign(_)
        | Prop::Getter(_)
        | Prop::Setter(_)
        | Prop::Method(_) => false,
      },
      PropOrSpread::Spread(_) => false,
    }),
    Expr::Tpl(expr) => expr.exprs.iter().all(is_compile_time_expression),
    Expr::Seq(expr) => expr.exprs.iter().all(is_compile_time_expression),
    Expr::Paren(expr) => is_compile_time_expression(&expr.expr),
    _ => false,
  }
}

fn is_compile_time_source(source: &str) -> bool {
  let allocator = Allocator::new();
  parse_file_as_expr(
    &allocator,
    allocator.alloc_str(source),
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
  )
  .is_ok_and(|expr| is_compile_time_expression(&expr))
}

pub fn is_compile_time_define_value(code: &Value) -> bool {
  match code {
    Value::Array(items) => items.iter().all(is_compile_time_define_value),
    Value::Object(object) => object.values().all(is_compile_time_define_value),
    Value::String(source) => is_compile_time_source(source),
    Value::Null | Value::Bool(_) | Value::Number(_) => true,
  }
}

pub fn gen_const_dep(
  parser: &JavascriptParser,
  code: Cow<str>,
  for_name: &str,
  start: u32,
  end: u32,
) -> Vec<BoxDependencyTemplate> {
  let code = if parser.in_short_hand {
    format!("{for_name}: {code}")
  } else {
    code.into_owned()
  };

  let to_const_dep = |requirements: Option<RuntimeGlobals>| {
    let mut res: Vec<BoxDependencyTemplate> = vec![];
    res.push(Box::new(ConstDependency::new(
      (start, end).into(),
      code.clone().into_boxed_str(),
    )));
    if let Some(requirements) = requirements {
      res.push(Box::new(RuntimeRequirementsDependency::add_only(
        requirements,
      )));
    }
    res
  };

  if parser
    .parser_runtime_requirements
    .require_regex
    .is_match(&code)
  {
    to_const_dep(Some(RuntimeGlobals::REQUIRE))
  } else if code.contains(&parser.parser_runtime_requirements.require) {
    to_const_dep(Some(RuntimeGlobals::REQUIRE_SCOPE))
  } else {
    to_const_dep(None)
  }
}

pub fn code_to_string<'a>(
  code: &'a Value,
  asi_safe: Option<bool>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> Cow<'a, str> {
  fn wrap_ansi(code: Cow<str>, is_arr: bool, asi_safe: Option<bool>) -> Cow<str> {
    match asi_safe {
      Some(true) if is_arr => code,
      Some(true) => Cow::Owned(format!("({code})")),
      Some(false) if is_arr => Cow::Owned(format!(";{code}")),
      Some(false) => Cow::Owned(format!(";({code})")),
      None => code,
    }
  }

  match code {
    Value::Null => Cow::Borrowed("null"),
    Value::String(s) => Cow::Borrowed(s),
    Value::Bool(b) => Cow::Borrowed(if *b { "true" } else { "false" }),
    Value::Number(n) => Cow::Owned(n.to_string()),
    Value::Array(arr) => {
      let elements = arr
        .iter()
        .map(|code| code_to_string(code, None, None))
        .join(",");
      wrap_ansi(Cow::Owned(format!("[{elements}]")), true, asi_safe)
    }
    Value::Object(obj) => {
      let elements = obj
        .iter()
        .filter_map(|(key, value)| {
          if obj_keys.is_none_or(|keys| keys.iter().any(|prop| prop.id.as_str() == key)) {
            Some(format!(
              "{}:{}",
              json!(key),
              code_to_string(value, None, None)
            ))
          } else {
            None
          }
        })
        .join(",");
      wrap_ansi(Cow::Owned(format!("{{ {elements} }}")), false, asi_safe)
    }
  }
}
