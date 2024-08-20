use std::path::PathBuf;

use itertools::Itertools;
use rspack_core::Compilation;
use sugar_path::SugarPath;
use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};
use swc_html::ast::{Attribute, Child, Element, Namespace, Text};
use swc_html::visit::{VisitMut, VisitMutWith};

use super::utils::{append_hash, create_element, generate_posix_path};
use crate::config::{HtmlInject, HtmlRspackPluginOptions, HtmlScriptLoading};

// the tag
#[derive(Debug)]
pub struct HTMLPluginTag {
  pub tag_name: String,
  pub attributes: Vec<HtmlPluginAttribute>,
  pub void_tag: bool,
  // `head`, `body`, `false`
  pub append_to: HtmlInject,
}

impl HTMLPluginTag {
  pub fn create_style(href: &str, append_to: HtmlInject) -> HTMLPluginTag {
    HTMLPluginTag {
      tag_name: "link".to_string(),
      append_to,
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
    append_to: HtmlInject,
    script_loading: &HtmlScriptLoading,
  ) -> HTMLPluginTag {
    let mut attributes = vec![];
    match script_loading {
      HtmlScriptLoading::Defer => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "defer".to_string(),
          attr_value: None,
        });
      }
      HtmlScriptLoading::Module => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "type".to_string(),
          attr_value: Some("module".to_string()),
        });
      }
      HtmlScriptLoading::SystemjsModule => {
        attributes.push(HtmlPluginAttribute {
          attr_name: "type".to_string(),
          attr_value: Some("systemjs-module".to_string()),
        });
      }
      _ => {}
    }

    attributes.push(HtmlPluginAttribute {
      attr_name: "src".to_string(),
      attr_value: Some(src.to_string()),
    });

    HTMLPluginTag {
      tag_name: "script".to_string(),
      append_to,
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
  config: &'a HtmlRspackPluginOptions,
  head_tags: Vec<&'a HTMLPluginTag>,
  body_tags: Vec<&'a HTMLPluginTag>,
  compilation: &'c Compilation,
  html_path: &'a str,
}

impl<'a, 'c> AssetWriter<'a, 'c> {
  pub fn new(
    config: &'a HtmlRspackPluginOptions,
    tags: &'a [HTMLPluginTag],
    compilation: &'c Compilation,
    html_path: &'a str,
  ) -> AssetWriter<'a, 'c> {
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
      config,
      head_tags,
      body_tags,
      compilation,
      html_path,
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
              data: Atom::from(title.as_str()),
              raw: None,
            })];
          } else {
            n.children.push(Child::Element(Element {
              tag_name: Atom::from("title"),
              children: vec![Child::Text(Text {
                span: DUMMY_SP,
                data: Atom::from(title.as_str()),
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

        // add basic tag
        if let Some(base) = &self.config.base {
          let mut attributes = vec![];

          if let Some(href) = &base.href {
            attributes.push(Attribute {
              span: Default::default(),
              namespace: None,
              prefix: None,
              name: "href".into(),
              raw_name: None,
              value: Some(href.to_string().into()),
              raw_value: None,
            });
          }

          if let Some(target) = &base.target {
            attributes.push(Attribute {
              span: Default::default(),
              namespace: None,
              prefix: None,
              name: "target".into(),
              raw_name: None,
              value: Some(target.to_string().into()),
              raw_value: None,
            });
          }

          if !attributes.is_empty() {
            n.children.push(Child::Element(Element {
              tag_name: Atom::from("base"),
              children: vec![],
              is_self_closing: true,
              namespace: Namespace::HTML,
              span: DUMMY_SP,
              attributes,
              content: None,
            }));
          }
        }

        // add favicon link
        if let Some(favicon) = &self.config.favicon {
          let favicon = PathBuf::from(favicon)
            .file_name()
            .expect("favicon should have file name")
            .to_string_lossy()
            .to_string();

          let favicon_relative_path =
            PathBuf::from(self.config.get_relative_path(self.compilation, &favicon));

          let mut favicon_path: PathBuf = PathBuf::from(self.config.get_public_path(
            self.compilation,
            favicon_relative_path.to_string_lossy().to_string().as_str(),
          ));

          if favicon_path.to_str().unwrap_or_default().is_empty() {
            favicon_path = self
              .compilation
              .options
              .output
              .path
              .as_std_path()
              .join(favicon_relative_path)
              .relative(PathBuf::from(self.html_path).join(".."));
          } else {
            favicon_path.push(favicon_relative_path);
          }

          let mut favicon_link_path = favicon_path.to_string_lossy().to_string();

          if self.config.hash.unwrap_or_default() {
            if let Some(hash) = self.compilation.get_hash() {
              favicon_link_path = append_hash(&favicon_link_path, hash);
            }
          }

          n.children.push(Child::Element(Element {
            tag_name: Atom::from("link"),
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
                value: Some(generate_posix_path(&favicon_link_path).into()),
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
              tag_name: Atom::from("meta"),
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
