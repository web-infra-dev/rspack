use std::borrow::Cow;

use concat_string::concat_string;
use itertools::Itertools as _;
use rspack_core::{
  BoxDependencyTemplate, ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_util::json_stringify_str;
use serde_json::{Map, Value};

use crate::visitors::{DestructuringAssignmentProperties, JavascriptParser};

/// Collect the nested patterns for every destructuring property matching
/// `key`. The outer `Option` distinguishes an unreferenced property; the inner
/// `Option` is `None` when any match reads the complete property value.
fn merged_nested_properties(
  keys: &DestructuringAssignmentProperties,
  key: &str,
) -> Option<Option<DestructuringAssignmentProperties>> {
  let mut matched = false;
  let mut nested = DestructuringAssignmentProperties::default();

  for property in keys.iter().filter(|property| property.id.as_str() == key) {
    matched = true;
    let Some(pattern) = &property.pattern else {
      return Some(None);
    };
    nested.extend(pattern.clone());
  }

  matched.then_some(Some(nested))
}

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
  let elements = object
    .iter()
    .filter_map(|(key, value)| {
      let nested_keys = match obj_keys {
        Some(keys) => merged_nested_properties(keys, key)?,
        None => None,
      };
      let key = if key == "__proto__" {
        concat_string!("[", json_stringify_str(key), "]")
      } else {
        json_stringify_str(key)
      };
      Some(concat_string!(
        key,
        ":",
        code_to_string(value, None, nested_keys.as_ref())
      ))
    })
    .join(",");
  wrap_code(
    Cow::Owned(concat_string!("{ ", elements, " }")),
    false,
    asi_safe,
  )
}

pub(crate) fn code_object_property_to_string<'a>(
  object: &'a Map<String, Value>,
  asi_safe: Option<bool>,
  property: &str,
) -> Cow<'a, str> {
  code_object_to_string_with_filter(object, asi_safe, |key| key == property)
}

/// Serialize a define value to its code representation. String values are
/// code fragments and are embedded verbatim; object keys are JSON-escaped.
///
/// When `obj_keys` is given (the destructured properties collected from an
/// object pattern), only those properties are emitted, recursing into nested
/// object patterns. Note that values are never parsed: code fragments are
/// spliced verbatim, which tolerates fragments that are not valid standalone
/// expressions (they may only be valid—or even invalid—where they are used).
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
      wrap_code(
        Cow::Owned(concat_string!("[", elements, "]")),
        true,
        asi_safe,
      )
    }
    Value::Object(obj) => code_object_to_string(obj, asi_safe, obj_keys),
  }
}

#[cfg(test)]
mod tests {
  use rspack_core::DependencyRange;
  use rspack_util::fx_hash::FxIndexSet;
  use serde_json::json;
  use swc_atoms::Atom;

  use super::*;
  use crate::visitors::DestructuringAssignmentProperty;

  fn keys(
    props: impl IntoIterator<Item = DestructuringAssignmentProperty>,
  ) -> DestructuringAssignmentProperties {
    DestructuringAssignmentProperties::new(FxIndexSet::from_iter(props))
  }

  fn prop(id: &str) -> DestructuringAssignmentProperty {
    DestructuringAssignmentProperty {
      range: DependencyRange::default(),
      id: Atom::from(id),
      pattern: None,
      shorthand: true,
    }
  }

  fn prop_nested(
    id: &str,
    pattern: DestructuringAssignmentProperties,
  ) -> DestructuringAssignmentProperty {
    DestructuringAssignmentProperty {
      pattern: Some(pattern),
      ..prop(id)
    }
  }

  fn prop_nested_at(
    id: &str,
    pattern: DestructuringAssignmentProperties,
    start: u32,
  ) -> DestructuringAssignmentProperty {
    DestructuringAssignmentProperty {
      range: DependencyRange::new(start, start + 1),
      ..prop_nested(id, pattern)
    }
  }

  #[test]
  fn filters_top_level_keys() {
    let value = json!({ "a": 1, "b": 2, "c": 3 });
    assert_eq!(
      code_to_string(&value, None, Some(&keys([prop("a"), prop("c")]))),
      r#"{ "a":1,"c":3 }"#
    );
  }

  #[test]
  fn filters_nested_object_patterns() {
    let value = json!({ "env": { "NODE_ENV": "\"production\"", "DEBUG": true }, "other": 1 });
    assert_eq!(
      code_to_string(
        &value,
        None,
        Some(&keys([prop_nested("env", keys([prop("NODE_ENV")]))]))
      ),
      r#"{ "env":{ "NODE_ENV":"production" } }"#
    );
  }

  #[test]
  fn keeps_arrays_whole_even_with_nested_patterns() {
    let value = json!({ "arr": [1, 2, 3], "other": 1 });
    assert_eq!(
      code_to_string(
        &value,
        None,
        Some(&keys([prop_nested("arr", keys([prop("0")]))]))
      ),
      r#"{ "arr":[1,2,3] }"#
    );
  }

  #[test]
  fn merges_repeated_nested_object_patterns() {
    let value = json!({ "x": { "a": 1, "b": 2, "c": 3 } });
    let selected = keys([
      prop_nested_at("x", keys([prop("a")]), 1),
      prop_nested_at("x", keys([prop("b")]), 2),
    ]);

    assert_eq!(
      code_to_string(&value, None, Some(&selected)),
      r#"{ "x":{ "a":1,"b":2 } }"#
    );
  }

  #[test]
  fn repeated_leaf_keeps_the_complete_object_regardless_of_order() {
    let value = json!({ "x": { "a": 1, "b": 2 } });
    let nested_then_leaf = keys([
      prop_nested_at("x", keys([prop("a")]), 1),
      DestructuringAssignmentProperty {
        range: DependencyRange::new(2, 3),
        ..prop("x")
      },
    ]);
    let leaf_then_nested = keys([
      DestructuringAssignmentProperty {
        range: DependencyRange::new(1, 2),
        ..prop("x")
      },
      prop_nested_at("x", keys([prop("a")]), 2),
    ]);

    for selected in [nested_then_leaf, leaf_then_nested] {
      assert_eq!(
        code_to_string(&value, None, Some(&selected)),
        r#"{ "x":{ "a":1,"b":2 } }"#
      );
    }
  }

  #[test]
  fn prunes_unparseable_fragments_verbatim() {
    // Unused properties may contain fragments that are not valid standalone
    // expressions; they must be pruned without ever being parsed.
    let value = json!({
      "used": 1,
      "unused": "(() => throw new Error('unused property was rendered'))()",
    });
    assert_eq!(
      code_to_string(&value, None, Some(&keys([prop("used")]))),
      r#"{ "used":1 }"#
    );
  }
}
