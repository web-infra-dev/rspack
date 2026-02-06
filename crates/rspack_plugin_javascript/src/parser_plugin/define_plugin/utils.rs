use std::borrow::Cow;

use itertools::Itertools as _;
use rspack_core::{
  BoxDependencyTemplate, ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency,
};
use serde_json::{Value, json};

use crate::visitors::{DestructuringAssignmentProperties, JavascriptParser};

pub(super) fn gen_const_dep(
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

pub(super) fn code_to_string<'a>(
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
