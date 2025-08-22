use std::{borrow::Cow, sync::LazyLock};

use itertools::Itertools as _;
use regex::Regex;
use rspack_core::{ConstDependency, RuntimeGlobals};
use rustc_hash::FxHashSet;
use serde_json::{Value, json};

use crate::visitors::{DestructuringAssignmentProperty, JavascriptParser};

static WEBPACK_REQUIRE_FUNCTION_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)")
    .expect("should init `WEBPACK_REQUIRE_FUNCTION_REGEXP`")
});
static WEBPACK_REQUIRE_IDENTIFIER: &str = "__webpack_require__";

pub fn gen_const_dep(
  parser: &JavascriptParser,
  code: Cow<str>,
  for_name: &str,
  start: u32,
  end: u32,
) -> ConstDependency {
  let code = if parser.in_short_hand {
    format!("{for_name}: {code}")
  } else {
    code.into_owned()
  };

  let to_const_dep = |requirements: Option<RuntimeGlobals>| {
    ConstDependency::new(
      (start, end).into(),
      code.clone().into_boxed_str(),
      requirements,
    )
  };

  if WEBPACK_REQUIRE_FUNCTION_REGEXP.is_match(&code) {
    to_const_dep(Some(RuntimeGlobals::REQUIRE))
  } else if code.contains(WEBPACK_REQUIRE_IDENTIFIER) {
    to_const_dep(Some(RuntimeGlobals::REQUIRE_SCOPE))
  } else {
    to_const_dep(None)
  }
}

pub fn code_to_string(
  code: &Value,
  asi_safe: Option<bool>,
  obj_keys: Option<FxHashSet<DestructuringAssignmentProperty>>,
) -> Cow<'_, str> {
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
          if obj_keys
            .as_ref()
            .is_none_or(|keys| keys.iter().any(|prop| prop.id.as_str() == key))
          {
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
