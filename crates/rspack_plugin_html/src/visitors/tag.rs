use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::asset::HtmlPluginAttribute;
use crate::config::{HtmlInject, HtmlRspackPluginBaseOptions, HtmlScriptLoading};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HTMLPluginTag {
  pub tag_name: String,
  pub attributes: Vec<HtmlPluginAttribute>,
  pub void_tag: bool,
  pub content: Option<String>,
  // `head`, `body`, `false`
  #[serde(skip)]
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
      ..Default::default()
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
      ..Default::default()
    }
  }

  pub fn create_base(base: &HtmlRspackPluginBaseOptions) -> Option<HTMLPluginTag> {
    let mut attributes = vec![];

    if let Some(href) = &base.href {
      attributes.push(HtmlPluginAttribute {
        attr_name: "href".to_string(),
        attr_value: Some(href.to_string()),
      });
    }

    if let Some(target) = &base.target {
      attributes.push(HtmlPluginAttribute {
        attr_name: "target".to_string(),
        attr_value: Some(target.to_string()),
      });
    }

    if !attributes.is_empty() {
      Some(HTMLPluginTag {
        tag_name: "base".to_string(),
        attributes,
        void_tag: true,
        ..Default::default()
      })
    } else {
      None
    }
  }

  pub fn create_title(title: &str) -> HTMLPluginTag {
    HTMLPluginTag {
      tag_name: "title".to_string(),
      void_tag: true,
      content: Some(title.to_string()),
      ..Default::default()
    }
  }

  pub fn create_meta(meta: &HashMap<String, HashMap<String, String>>) -> Vec<HTMLPluginTag> {
    meta
      .iter()
      .map(|(_, value)| HTMLPluginTag {
        tag_name: "meta".to_string(),
        attributes: value
          .iter()
          .sorted()
          .map(|(key, value)| HtmlPluginAttribute {
            attr_name: key.to_string(),
            attr_value: Some(value.to_string()),
          })
          .collect_vec(),
        void_tag: true,
        ..Default::default()
      })
      .collect_vec()
  }

  pub fn create_favicon(favicon: &str) -> HTMLPluginTag {
    HTMLPluginTag {
      tag_name: "link".to_string(),
      attributes: vec![
        HtmlPluginAttribute {
          attr_name: "rel".to_string(),
          attr_value: Some("icon".into()),
        },
        HtmlPluginAttribute {
          attr_name: "href".to_string(),
          attr_value: Some(favicon.into()),
        },
      ],
      void_tag: true,
      ..Default::default()
    }
  }
}
