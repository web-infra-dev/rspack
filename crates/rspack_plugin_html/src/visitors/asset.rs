use itertools::Itertools;
use rspack_core::Compilation;
use swc_core::{common::DUMMY_SP, ecma::atoms::JsWord};
use swc_html::ast::{Attribute, Child, Element, Namespace, Text};
use swc_html::visit::{VisitMut, VisitMutWith};

use super::utils::create_element;
use crate::config::{HtmlPluginConfig, HtmlPluginConfigInject, HtmlPluginConfigScriptLoading};

// the tag
#[derive(Debug)]
pub struct HTMLPluginTag {
  pub tag_name: String,
  pub attributes: Vec<HtmlPluginAttribute>,
  pub void_tag: bool,
  // `head` or `body`
  pub append_to: HtmlPluginConfigInject,
}

impl HTMLPluginTag {
  pub fn create_style(href: &str, append_to: Option<HtmlPluginConfigInject>) -> HTMLPluginTag {
    HTMLPluginTag {
      tag_name: "link".to_string(),
      append_to: append_to.unwrap_or(HtmlPluginConfigInject::Head),
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
      void_tag: true,
    }
  }

  pub fn create_script(
    src: &str,
    append_to: Option<HtmlPluginConfigInject>,
    script_loading: &HtmlPluginConfigScriptLoading,
  ) -> HTMLPluginTag {
    let mut attributes = vec![HtmlPluginAttribute {
      attr_name: "src".to_string(),
      attr_value: Some(src.to_string()),
    }];
    match script_loading {
      HtmlPluginConfigScriptLoading::Defer => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "defer".to_string(),
          attr_value: None,
        });
      }
      HtmlPluginConfigScriptLoading::Module => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "type".to_string(),
          attr_value: Some("module".to_string()),
        });
      }
      _ => {}
    }

    HTMLPluginTag {
      tag_name: "script".to_string(),
      append_to: append_to.unwrap_or(HtmlPluginConfigInject::Body),
      attributes,
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
pub struct AssetWriter<'a, 'c> {
  config: &'a HtmlPluginConfig,
  head_tags: Vec<&'a HTMLPluginTag>,
  body_tags: Vec<&'a HTMLPluginTag>,
  compilation: &'c Compilation,
}

impl<'a, 'c> AssetWriter<'a, 'c> {
  pub fn new(
    config: &'a HtmlPluginConfig,
    tags: &'a [HTMLPluginTag],
    compilation: &'c Compilation,
  ) -> AssetWriter<'a, 'c> {
    let mut head_tags: Vec<&HTMLPluginTag> = vec![];
    let mut body_tags: Vec<&HTMLPluginTag> = vec![];
    for ele in tags.iter() {
      match ele.append_to {
        HtmlPluginConfigInject::Head => {
          head_tags.push(ele);
        }
        HtmlPluginConfigInject::Body => {
          body_tags.push(ele);
        }
      }
    }
    AssetWriter {
      config,
      head_tags,
      body_tags,
      compilation,
    }
  }
}

impl VisitMut for AssetWriter<'_, '_> {
  fn visit_mut_element(&mut self, n: &mut Element) {
    let head_tags = &self.head_tags;
    let body_tags = &self.body_tags;

    match &*n.tag_name {
      "head" => {
        // add title
        if let Some(title) = &self.config.title {
          let title_ele = n.children.iter_mut().find(|child| {
            if let Child::Element(ele) = child {
              return ele.tag_name.eq("title");
            }
            false
          });

          if let Some(Child::Element(title_ele)) = title_ele {
            title_ele.children = vec![Child::Text(Text {
              span: DUMMY_SP,
              data: JsWord::from(title.as_str()),
              raw: None,
            })];
          } else {
            n.children.push(Child::Element(Element {
              tag_name: JsWord::from("title"),
              children: vec![Child::Text(Text {
                span: DUMMY_SP,
                data: JsWord::from(title.as_str()),
                raw: None,
              })],
              is_self_closing: false,
              namespace: Namespace::HTML,
              span: DUMMY_SP,
              attributes: vec![],
              content: None,
            }));
          }
        }

        // add favicon
        if let Some(favicon) = &self.config.favicon {
          let favicon_path = format!(
            "{}{favicon}",
            self.config.get_public_path(self.compilation, favicon)
          );
          n.children.push(Child::Element(Element {
            tag_name: JsWord::from("link"),
            children: vec![],
            is_self_closing: true,
            namespace: Namespace::HTML,
            span: DUMMY_SP,
            attributes: vec![
              Attribute {
                span: Default::default(),
                namespace: None,
                prefix: None,
                name: "rel".into(),
                raw_name: None,
                value: Some("icon".into()),
                raw_value: None,
              },
              Attribute {
                span: Default::default(),
                namespace: None,
                prefix: None,
                name: "href".into(),
                raw_name: None,
                value: Some(favicon_path.into()),
                raw_value: None,
              },
            ],
            content: None,
          }));
        }

        // add meta tags
        if let Some(meta) = &self.config.meta {
          for key in meta.keys().sorted() {
            let value = meta.get(key).expect("should have value");
            let meta_ele = Element {
              tag_name: JsWord::from("meta"),
              attributes: value
                .iter()
                .sorted()
                .map(|(key, value)| Attribute {
                  span: Default::default(),
                  namespace: None,
                  prefix: None,
                  name: key.clone().into(),
                  raw_name: None,
                  value: Some(value.clone().into()),
                  raw_value: None,
                })
                .collect(),
              children: vec![],
              content: None,
              is_self_closing: true,
              namespace: Namespace::HTML,
              span: DUMMY_SP,
            };
            n.children.push(Child::Element(meta_ele));
          }
        }

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
