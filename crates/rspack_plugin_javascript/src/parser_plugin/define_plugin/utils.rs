use std::borrow::Cow;

use concat_string::concat_string;
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
    concat_string!(for_name, ": ", code)
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
    Some(true) => Cow::Owned(concat_string!("(", code, ")")),
    Some(false) if is_array => Cow::Owned(concat_string!(";", code)),
    Some(false) => Cow::Owned(concat_string!(";(", code, ")")),
    None => code,
  }
}

fn code_object_to_string_with_filter<'a>(
  object: &'a Map<String, Value>,
  asi_safe: Option<bool>,
  should_include: impl Fn(&str) -> bool,
) -> Cow<'a, str> {
  let elements = object
    .iter()
    .filter_map(|(key, value)| {
      if should_include(key) {
        // Emit `__proto__` as a computed key so it becomes an own property
        // instead of setting the prototype (matches webpack's `stringifyObj`).
        let key = if key == "__proto__" {
          concat_string!("[", json_stringify_str(key), "]")
        } else {
          json_stringify_str(key)
        };
        Some(concat_string!(key, ":", code_to_string(value, None, None)))
      } else {
        None
      }
    })
    .join(",");
  wrap_code(
    Cow::Owned(concat_string!("{ ", elements, " }")),
    false,
    asi_safe,
  )
}

pub(crate) fn code_object_to_string<'a>(
  object: &'a Map<String, Value>,
  asi_safe: Option<bool>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> Cow<'a, str> {
  code_object_to_string_with_filter(object, asi_safe, |key| {
    obj_keys.is_none_or(|keys| keys.iter().any(|prop| prop.id.as_str() == key))
  })
}

pub(crate) fn code_object_property_to_string<'a>(
  object: &'a Map<String, Value>,
  asi_safe: Option<bool>,
  property: &str,
) -> Cow<'a, str> {
  code_object_to_string_with_filter(object, asi_safe, |key| key == property)
}

pub fn code_to_string<'a>(
  code: &'a Value,
  asi_safe: Option<bool>,
  obj_keys: Option<&DestructuringAssignmentProperties>,
) -> Cow<'a, str> {
  match code {
    Value::Null => Cow::Borrowed("null"),
    Value::String(s) => wrap_code(Cow::Borrowed(s), false, asi_safe),
    Value::Bool(b) => Cow::Borrowed(if *b { "true" } else { "false" }),
    Value::Number(n) => Cow::Owned(n.to_string()),
    Value::Array(arr) => {
      let elements = arr
        .iter()
        .map(|code| code_to_string(code, None, None))
        .join(",");
      wrap_code(
        Cow::Owned(concat_string!("[", elements, "]")),
        true,
        asi_safe,
      )
    }
    Value::Object(obj) => code_object_to_string(obj, asi_safe, obj_keys),
  }
}
