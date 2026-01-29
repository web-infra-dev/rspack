use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};
use swc_html::{
  ast::{Child, Element, Text},
  visit::{VisitMut, VisitMutWith},
};

use crate::tag::HtmlPluginTag;

#[derive(Debug)]
pub struct AssetInjector<'a> {
  head_tags: &'a Vec<HtmlPluginTag>,
  body_tags: &'a Vec<HtmlPluginTag>,
}

impl<'a> AssetInjector<'a> {
  pub fn new(
    head_tags: &'a Vec<HtmlPluginTag>,
    body_tags: &'a Vec<HtmlPluginTag>,
  ) -> AssetInjector<'a> {
    AssetInjector {
      head_tags,
      body_tags,
    }
  }
}

impl VisitMut for AssetInjector<'_> {
  fn visit_mut_element(&mut self, n: &mut Element) {
    let head_tags = &self.head_tags;
    let body_tags = &self.body_tags;

    match &*n.tag_name {
      "head" => {
        for tag in head_tags.iter() {
          if tag.tag_name == "title"
            && let Some(Child::Element(title_ele)) = n.children.iter_mut().find(|child| {
              if let Child::Element(ele) = child {
                return ele.tag_name.eq("title");
              }
              false
            })
          {
            title_ele.children = vec![Child::Text(Text {
              span: DUMMY_SP,
              data: Atom::from(
                tag
                  .inner_html
                  .as_ref()
                  .unwrap_or_else(|| panic!("should have title content")).clone(),
              ),
              raw: None,
            })];
            continue;
          }
          n.children
            .push(Child::Element(Element::from(tag.to_owned())));
        }
      }
      "body" => {
        for tag in body_tags.iter() {
          n.children
            .push(Child::Element(Element::from(tag.to_owned())));
        }
      }
      _ => {}
    }

    n.visit_mut_children_with(self);
  }
}
