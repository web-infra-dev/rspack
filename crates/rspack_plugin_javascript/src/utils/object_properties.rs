use std::ops::Deref;

use rspack_core::ImportAttributes;
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
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Str(str) => Some(str),
    _ => None,
  }
}

pub fn get_bool_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Bool> {
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Bool(bool) => Some(bool),
    _ => None,
  }
}

pub fn get_regex_by_obj_prop<'a>(obj: &'a ObjectLit, field: &'a str) -> Option<&'a Regex> {
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Regex(regexp) => Some(regexp),
    _ => None,
  }
}

pub fn get_attributes(obj: &ObjectLit) -> ImportAttributes {
  ImportAttributes::from_iter(obj.props.iter().filter_map(|p| {
    p.as_prop().and_then(|p| p.as_key_value()).and_then(|kv| {
      kv.key
        .as_ident()
        .map(|k| k.sym.as_str())
        .or_else(|| kv.key.as_str().map(|k| k.value.as_str()))
        .map(|s| s.to_string())
        .zip(kv.value.as_lit().and_then(|lit| match lit {
          Lit::Str(s) => Some(s.value.to_string()),
          _ => None,
        }))
    })
  }))
}
