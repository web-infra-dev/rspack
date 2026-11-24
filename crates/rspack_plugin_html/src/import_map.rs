use std::collections::BTreeMap;

use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use serde_json::{Map, Value, json};
use swc_core::common::DUMMY_SP;
use swc_html::ast::{Child, Element, Namespace, Text};

use crate::{
  parser::CompiledDocument,
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};

pub(crate) fn add_integrity_to_import_map(
  document: &mut CompiledDocument,
  integrity: BTreeMap<String, String>,
) -> Result<()> {
  let children = match document {
    CompiledDocument::Document(document) | CompiledDocument::DocumentWithoutDoctype(document) => {
      &mut document.children
    }
    CompiledDocument::DocumentFragment(fragment) => &mut fragment.children,
  };
  let mut seen_module_fetch = false;
  if merge_existing_map(children, &integrity, &mut seen_module_fetch)? || !seen_module_fetch {
    return Ok(());
  }

  let contents = serialize_map(&json!({ "integrity": integrity }))?;
  insert_before_module_fetch(children, &contents);
  Ok(())
}

fn attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
  element
    .attributes
    .iter()
    .find(|attribute| attribute.name == name)
    .and_then(|attribute| attribute.value.as_deref())
}

fn is_script(element: &Element, script_type: &str) -> bool {
  element.tag_name == "script"
    && attribute(element, "type").is_some_and(|value| value.eq_ignore_ascii_case(script_type))
}

fn starts_module_fetch(element: &Element) -> bool {
  is_script(element, "module")
    || (element.tag_name == "link"
      && attribute(element, "rel").is_some_and(|value| {
        value
          .split_ascii_whitespace()
          .any(|rel| rel.eq_ignore_ascii_case("modulepreload"))
      }))
}

fn is_active_html_element(element: &Element) -> bool {
  element.namespace == Namespace::HTML && !matches!(&*element.tag_name, "template" | "noscript")
}

fn merge_existing_map(
  children: &mut [Child],
  integrity: &BTreeMap<String, String>,
  seen_module_fetch: &mut bool,
) -> Result<bool> {
  for child in children {
    let Child::Element(element) = child else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if is_script(element, "importmap") {
      if *seen_module_fetch {
        return Err(error!(
          "Place the import map before module scripts and modulepreload links to enable Subresource Integrity."
        ));
      }
      let mut source = String::new();
      for child in &element.children {
        if let Child::Text(text) = child {
          source.push_str(&text.data);
        }
      }
      let mut map: Map<String, Value> = serde_json::from_str(&source)
        .map_err(|error| error!("Cannot add Subresource Integrity to the import map: {error}"))?;
      let entries = map
        .entry("integrity")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| error!("The import map integrity field must be an object."))?;
      for (url, hash) in integrity {
        entries
          .entry(url)
          .or_insert_with(|| Value::String(hash.clone()));
      }
      element.children = vec![Child::Text(Text {
        span: DUMMY_SP,
        data: serialize_map(&Value::Object(map))?.into(),
        raw: None,
      })];
      return Ok(true);
    }
    *seen_module_fetch |= starts_module_fetch(element);
    if merge_existing_map(&mut element.children, integrity, seen_module_fetch)? {
      return Ok(true);
    }
  }
  Ok(false)
}

fn insert_before_module_fetch(children: &mut Vec<Child>, contents: &str) -> bool {
  for index in 0..children.len() {
    let Child::Element(element) = &mut children[index] else {
      continue;
    };
    if !is_active_html_element(element) {
      continue;
    }
    if starts_module_fetch(element) {
      let mut attributes = vec![HtmlPluginAttribute {
        attr_name: "type".into(),
        attr_value: Some("importmap".into()),
      }];
      if let Some(nonce) = attribute(element, "nonce") {
        attributes.push(HtmlPluginAttribute {
          attr_name: "nonce".into(),
          attr_value: Some(nonce.into()),
        });
      }
      children.insert(
        index,
        Child::Element(
          HtmlPluginTag {
            tag_name: "script".into(),
            attributes,
            inner_html: Some(contents.into()),
            ..Default::default()
          }
          .into(),
        ),
      );
      return true;
    }
    if insert_before_module_fetch(&mut element.children, contents) {
      return true;
    }
  }
  false
}

fn serialize_map(map: &Value) -> Result<String> {
  // A script element is raw text, so JSON string escaping alone does not protect its end tag.
  Ok(
    serde_json::to_string(map)
      .to_rspack_result()?
      .replace('<', "\\u003c"),
  )
}
