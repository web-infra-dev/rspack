use std::{collections::HashMap, fmt};

use itertools::Itertools;
use serde::{
  de::{MapAccess, Visitor},
  ser::SerializeMap,
  Deserialize, Deserializer, Serialize, Serializer,
};

use super::asset::HtmlPluginAttribute;
use crate::config::{HtmlRspackPluginBaseOptions, HtmlScriptLoading};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlPluginTag {
  pub tag_name: String,
  #[serde(
    serialize_with = "serialize_attributes",
    deserialize_with = "deserialize_attributes"
  )]
  pub attributes: Vec<HtmlPluginAttribute>,
  pub void_tag: bool,
  #[serde(rename = "innerHTML")]
  pub inner_html: Option<String>,
  // `head`, `body`, `false`
  pub asset: Option<String>,
}

fn serialize_attributes<S>(x: &Vec<HtmlPluginAttribute>, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let mut map = s.serialize_map(Some(x.len()))?;
  for attr in x {
    let attr_value = attr.attr_value.to_owned().unwrap_or("true".to_string());
    map.serialize_entry(&attr.attr_name, &attr_value)?;
  }
  map.end()
}

fn deserialize_attributes<'de, D>(d: D) -> Result<Vec<HtmlPluginAttribute>, D::Error>
where
  D: Deserializer<'de>,
{
  struct DataVisitor;

  impl<'de> Visitor<'de> for DataVisitor {
    type Value = Vec<HtmlPluginAttribute>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("html attributes")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
      A: MapAccess<'de>,
    {
      let mut res = vec![];

      while let Some((k, v)) = access.next_entry::<String, Option<String>>()? {
        res.push(HtmlPluginAttribute {
          attr_name: k,
          attr_value: v.filter(|value| value != "true"),
        });
      }

      Ok(res)
    }
  }

  d.deserialize_map(DataVisitor)
}

impl HtmlPluginTag {
  pub fn create_style(href: &str) -> HtmlPluginTag {
    HtmlPluginTag {
      tag_name: "link".to_string(),
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
      asset: Some(href.to_string()),
      ..Default::default()
    }
  }

  pub fn create_script(src: &str, script_loading: &HtmlScriptLoading) -> HtmlPluginTag {
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

    HtmlPluginTag {
      tag_name: "script".to_string(),
      attributes,
      asset: Some(src.to_string()),
      ..Default::default()
    }
  }

  pub fn create_base(base: &HtmlRspackPluginBaseOptions) -> Option<HtmlPluginTag> {
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
      Some(HtmlPluginTag {
        tag_name: "base".to_string(),
        attributes,
        void_tag: true,
        ..Default::default()
      })
    } else {
      None
    }
  }

  pub fn create_title(title: &str) -> HtmlPluginTag {
    HtmlPluginTag {
      tag_name: "title".to_string(),
      void_tag: true,
      inner_html: Some(title.to_string()),
      ..Default::default()
    }
  }

  pub fn create_meta(meta: &HashMap<String, HashMap<String, String>>) -> Vec<HtmlPluginTag> {
    meta
      .iter()
      .map(|(_, value)| HtmlPluginTag {
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

  pub fn create_favicon(favicon: &str) -> HtmlPluginTag {
    HtmlPluginTag {
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
