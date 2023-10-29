use std::ops::Deref;

use swc_core::ecma::ast::{Bool, Expr, Lit, ObjectLit, Regex, Str};

pub fn get_value_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Expr> {
  obj
    .props
    .iter()
    .find_map(|p| {
      p.as_prop()
        .and_then(|p| p.as_key_value())
        .filter(|kv| {
          kv.key.as_ident().filter(|key| key.sym == field).is_some()
            || kv.key.as_str().filter(|key| key.value == field).is_some()
        })
        .map(|name| &name.value)
    })
    .map(|boxed| boxed.deref())
}

pub fn get_literal_str_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Str> {
  let Some(lit) = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit()) else {
    return None;
  };
  match lit {
    Lit::Str(str) => Some(str),
    _ => None,
  }
}

pub fn get_bool_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Bool> {
  let Some(lit) = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit()) else {
    return None
  };
  match lit {
    Lit::Bool(bool) => Some(bool),
    _ => None,
  }
}

pub fn get_regex_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Regex> {
  let Some(lit) = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit()) else {
    return None
  };
  match lit {
    Lit::Regex(regexp) => Some(regexp),
    _ => None,
  }
}
