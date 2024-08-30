use std::{borrow::Cow, env};

use regex::Regex;
use serde_json::Value;
use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};
use swc_html::ast::{Attribute, Element, Namespace};

use super::{asset::HtmlPluginAttribute, tag::HtmlPluginTag};

pub fn create_attribute(name: &str, value: &Option<String>) -> Attribute {
  Attribute {
    span: Default::default(),
    namespace: None,
    prefix: None,
    name: name.into(),
    raw_name: None,
    value: value.as_ref().map(|str| Atom::from(str.as_str())),
    raw_value: None,
  }
}

pub fn create_attributes(attrs: &[HtmlPluginAttribute]) -> Vec<Attribute> {
  let mut res = attrs
    .iter()
    .map(|attr| create_attribute(&attr.attr_name, &attr.attr_value))
    .collect::<Vec<_>>();
  res.sort_unstable_by(|a, b| a.name.cmp(&b.name));
  res
}

pub fn create_element(tag: &HtmlPluginTag) -> Element {
  Element {
    tag_name: Atom::from(&*tag.tag_name),
    attributes: create_attributes(&tag.attributes),
    children: vec![],
    content: None,
    is_self_closing: tag.void_tag,
    namespace: Namespace::HTML,
    span: DUMMY_SP,
  }
}

pub fn append_hash(url: &str, hash: &str) -> String {
  format!(
    "{}{}{}",
    url,
    if url.contains("?") {
      "$$RSPACK_URL_AMP$$"
    } else {
      "?"
    },
    hash
  )
}

pub fn generate_posix_path(path: &str) -> Cow<'_, str> {
  if env::consts::OS == "windows" {
    let reg = Regex::new(r"[/\\]").expect("Invalid RegExp");
    reg.replace_all(path, "/")
  } else {
    path.into()
  }
}

pub fn merge_json(a: &mut Value, b: Value) {
  match (a, b) {
    (a @ &mut Value::Object(_), Value::Object(b)) => {
      let a = a
        .as_object_mut()
        .unwrap_or_else(|| panic!("merged json is not an object"));
      for (k, v) in b {
        merge_json(a.entry(k).or_insert(Value::Null), v);
      }
    }
    (a, b) => *a = b,
  }
}

pub fn html_tag_object_to_string(tag: &HtmlPluginTag) -> String {
  let mut attributes = tag
    .attributes
    .iter()
    .map(|attr| {
      if let Some(attr_value) = &attr.attr_value {
        format!(r#"{}="{}""#, attr.attr_name, attr_value)
      } else {
        attr.attr_name.to_string()
      }
    })
    .collect::<Vec<String>>();

  attributes.sort();

  let res = format!(
    "<{} {}{}>{}{}",
    tag.tag_name,
    attributes.join(" "),
    if tag.void_tag && tag.inner_html.is_none() {
      "/"
    } else {
      ""
    },
    if let Some(inner_html) = &tag.inner_html {
      inner_html
    } else {
      ""
    },
    if !tag.void_tag || tag.inner_html.is_some() {
      format!("</{}>", tag.tag_name)
    } else {
      String::new()
    }
  );
  res
}
