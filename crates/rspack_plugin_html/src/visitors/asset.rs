use swc_html::ast::{Child, Element};
use swc_html::visit::{VisitMut, VisitMutWith};

use crate::config::HtmlPluginConfig;

use super::utils::create_element;

// the tag
#[derive(Debug)]
pub struct HTMLPluginTag {
  pub tag_name: String,
  pub attributes: Vec<HtmlPluginAttribute>,
  pub void_tag: bool,
  // `head` or `body`
  pub append_to: String,
  // future
  pub meta: (),
}

impl HTMLPluginTag {
  pub fn create_style(href: &str, append_to: Option<String>) -> HTMLPluginTag {
    HTMLPluginTag {
      tag_name: "link".to_string(),
      append_to: append_to.unwrap_or_else(|| "head".to_string()),
      attributes: vec![
        HtmlPluginAttribute {
          attr_name: "href".to_string(),
          attr_value: Some(href.to_string()),
        },
        HtmlPluginAttribute {
          attr_name: "rel".to_string(),
          attr_value: Some("stylesheet".to_string()),
        },
      ],
      meta: (),
      void_tag: true,
    }
  }

  pub fn create_script(
    src: &str,
    append_to: Option<String>,
    script_loading: &str,
  ) -> HTMLPluginTag {
    let mut attributes = vec![HtmlPluginAttribute {
      attr_name: "src".to_string(),
      attr_value: Some(src.to_string()),
    }];
    match script_loading {
      "defer" => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "defer".to_string(),
          attr_value: None,
        });
      }
      "module" => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "type".to_string(),
          attr_value: Some("module".to_string()),
        });
      }
      _ => {}
    }

    HTMLPluginTag {
      tag_name: "script".to_string(),
      append_to: append_to.unwrap_or_else(|| "body".to_string()),
      attributes,
      meta: (),
      void_tag: false,
    }
  }
}

// attributes are presented as plain string.
// namespace is not supported currently.
#[derive(Debug)]
pub struct HtmlPluginAttribute {
  pub attr_name: String,
  // None is ``
  pub attr_value: Option<String>,
}

#[derive(Debug)]
pub struct AssetWriter<'a> {
  config: &'a HtmlPluginConfig,
  head_tags: Vec<&'a HTMLPluginTag>,
  body_tags: Vec<&'a HTMLPluginTag>,
}

impl<'a> AssetWriter<'a> {
  pub fn new(config: &'a HtmlPluginConfig, tags: &'a [HTMLPluginTag]) -> AssetWriter<'a> {
    let mut head_tags: Vec<&HTMLPluginTag> = vec![];
    let mut body_tags: Vec<&HTMLPluginTag> = vec![];
    for ele in tags.iter() {
      if ele.append_to.eq("head") {
        head_tags.push(ele);
      } else {
        body_tags.push(ele);
      }
    }
    AssetWriter {
      config,
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
