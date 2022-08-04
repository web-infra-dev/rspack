use swc_atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_html::ast::{Attribute, Element, Namespace};

use super::asset::{HTMLPluginTag, HtmlPluginAttribute};

pub fn create_attribute(name: &str, value: &Option<String>) -> Attribute {
  Attribute {
    span: Default::default(),
    namespace: None,
    prefix: None,
    name: name.into(),
    raw_name: None,
    value: value.as_ref().map(|str| JsWord::from(str.to_string())),
    raw_value: None,
  }
}

pub fn create_attributes(attrs: &[HtmlPluginAttribute]) -> Vec<Attribute> {
  attrs
    .iter()
    .map(|attr| create_attribute(&attr.attr_name, &attr.attr_value))
    .collect()
}

pub fn create_element(tag: &HTMLPluginTag) -> Element {
  Element {
    tag_name: JsWord::from(tag.tag_name.clone()),
    attributes: create_attributes(&tag.attributes),
    children: vec![],
    content: None,
    is_self_closing: tag.void_tag,
    namespace: Namespace::HTML,
    span: DUMMY_SP,
  }
}
