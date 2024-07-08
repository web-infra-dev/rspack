use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};
use swc_html::ast::{Attribute, Element, Namespace};

use super::asset::{HTMLPluginTag, HtmlPluginAttribute};

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
  attrs
    .iter()
    .map(|attr| create_attribute(&attr.attr_name, &attr.attr_value))
    .collect()
}

pub fn create_element(tag: &HTMLPluginTag) -> Element {
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
