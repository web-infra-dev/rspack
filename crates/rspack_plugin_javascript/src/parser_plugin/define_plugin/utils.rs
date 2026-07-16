use std::borrow::Cow;

use itertools::Itertools as _;
use rspack_core::{
  BoxDependencyTemplate, ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_util::json_stringify_str;
use serde_json::{Map, Value};

use crate::visitors::{DestructuringAssignmentProperties, JavascriptParser};

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

pub(crate) fn wrap_code<'a>(
  code: Cow<'a, str>,
  is_array: bool,
  asi_safe: Option<bool>,
) -> Cow<'a, str> {
  match asi_safe {
    Some(true) if is_array => code,
    Some(true) => Cow::Owned(format!("({code})")),
    Some(false) if is_array => Cow::Owned(format!(";{code}")),
    Some(false) => Cow::Owned(format!(";({code})")),
    None => code,
  }
}

pub fn code_object_properties_to_string<'a>(
  entries: impl IntoIterator<Item = (&'a str, &'a Value)>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> String {
  entries
    .into_iter()
    .filter_map(|(key, value)| {
      if obj_keys.is_none_or(|keys| keys.iter().any(|prop| prop.id.as_str() == key)) {
        // Emit `__proto__` as a computed key so it becomes an own property
        // instead of setting the prototype (matches webpack's `stringifyObj`).
        let key = if key == "__proto__" {
          format!("[{}]", json_stringify_str(key))
        } else {
          json_stringify_str(key)
        };
        Some(format!("{key}:{}", code_to_string(value, None, None)))
      } else {
        None
      }
    })
    .join(",")
}

pub(crate) fn code_object_to_string<'a>(
  object: &'a Map<String, Value>,
  asi_safe: Option<bool>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> Cow<'a, str> {
  let elements = code_object_properties_to_string(
    object.iter().map(|(key, value)| (key.as_str(), value)),
    obj_keys,
  );
  wrap_code(Cow::Owned(format!("{{ {elements} }}")), false, asi_safe)
}

pub fn code_to_string<'a>(
  code: &'a Value,
  asi_safe: Option<bool>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> Cow<'a, str> {
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
      wrap_code(Cow::Owned(format!("[{elements}]")), true, asi_safe)
    }
    Value::Object(obj) => code_object_to_string(obj, asi_safe, obj_keys),
  }
}
