use serde::{Deserialize, Serialize};
use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};
use swc_html::ast::{Child, Element, Text};
use swc_html::visit::{VisitMut, VisitMutWith};

use super::tag::HTMLPluginTag;
use super::utils::create_element;
use crate::config::HtmlInject;

// attributes are presented as plain string.
// namespace is not supported currently.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlPluginAttribute {
  pub attr_name: String,
  // None is ``
  pub attr_value: Option<String>,
}

#[derive(Debug)]
pub struct AssetWriter<'a> {
  head_tags: Vec<&'a HTMLPluginTag>,
  body_tags: Vec<&'a HTMLPluginTag>,
}

impl<'a> AssetWriter<'a> {
  pub fn new(tags: &'a [HTMLPluginTag]) -> AssetWriter<'a> {
    let mut head_tags: Vec<&HTMLPluginTag> = vec![];
    let mut body_tags: Vec<&HTMLPluginTag> = vec![];
    for ele in tags.iter() {
      match ele.append_to {
        HtmlInject::Head => {
          head_tags.push(ele);
        }
        HtmlInject::Body => {
          body_tags.push(ele);
        }
        _ => (),
      }
    }
    AssetWriter {
      head_tags,
      body_tags,
    }
  }
}

impl VisitMut for AssetWriter<'_> {
  fn visit_mut_element(&mut self, n: &mut Element) {
    let head_tags = &self.head_tags;
    let body_tags = &self.body_tags;

    match &*n.tag_name {
      "head" => {
        for tag in head_tags.iter() {
          if tag.tag_name == "title" {
            if let Some(Child::Element(title_ele)) = n.children.iter_mut().find(|child| {
              if let Child::Element(ele) = child {
                return ele.tag_name.eq("title");
              }
              false
            }) {
              title_ele.children = vec![Child::Text(Text {
                span: DUMMY_SP,
                data: Atom::from(
                  tag
                    .inner_html
                    .as_ref()
                    .unwrap_or_else(|| panic!("should have title content"))
                    .to_string(),
                ),
                raw: None,
              })];
              continue;
            }
          }

          let new_element = create_element(tag);

          n.children.push(Child::Element(new_element));
        }
      }
      "body" => {
        for tag in body_tags.iter() {
          let new_element = create_element(tag);
          n.children.push(Child::Element(new_element));
        }
      }
      _ => {}
    }

    n.visit_mut_children_with(self);
  }
}
