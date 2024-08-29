use std::collections::HashMap;

use napi_derive::napi;
use rspack_plugin_html::{
  visitors::{asset::HtmlPluginAttribute, tag::HtmlPluginTag},
  AfterEmitData, AfterTemplateExecutionData, AlterAssetTagGroupsData, AlterAssetTagsData,
  BeforeAssetTagGenerationData, BeforeEmitData, HtmlPluginAssetTags, HtmlPluginAssets,
};

#[napi(object)]
pub struct JsHtmlPluginTag {
  pub tag_name: String,
  pub attributes: HashMap<String, String>,
  pub void_tag: bool,
  pub inner_html: Option<String>,
  pub asset: Option<String>,
}

impl From<HtmlPluginTag> for JsHtmlPluginTag {
  fn from(value: HtmlPluginTag) -> Self {
    Self {
      tag_name: value.tag_name,
      attributes: value
        .attributes
        .iter()
        .map(|x| {
          (
            x.attr_name.to_owned(),
            x.attr_value.to_owned().unwrap_or("true".into()),
          )
        })
        .collect(),
      void_tag: value.void_tag,
      inner_html: value.inner_html,
      asset: value.asset,
    }
  }
}

impl From<JsHtmlPluginTag> for HtmlPluginTag {
  fn from(value: JsHtmlPluginTag) -> Self {
    Self {
      tag_name: value.tag_name,
      attributes: value
        .attributes
        .iter()
        .map(|(key, value)| HtmlPluginAttribute {
          attr_name: key.to_owned(),
          attr_value: if value == "true" {
            None
          } else {
            Some(value.to_owned())
          },
        })
        .collect::<Vec<_>>(),
      void_tag: value.void_tag,
      inner_html: value.inner_html,
      asset: value.asset,
    }
  }
}

#[napi(object)]
pub struct JsHtmlPluginAssets {
  pub public_path: String,
  pub js: Vec<String>,
  pub css: Vec<String>,
  pub favicon: Option<String>,
  // manifest: Option<String>,
}

impl From<HtmlPluginAssets> for JsHtmlPluginAssets {
  fn from(value: HtmlPluginAssets) -> Self {
    Self {
      public_path: value.public_path,
      js: value.js,
      css: value.css,
      favicon: value.favicon,
    }
  }
}

impl From<JsHtmlPluginAssets> for HtmlPluginAssets {
  fn from(value: JsHtmlPluginAssets) -> Self {
    Self {
      public_path: value.public_path,
      js: value.js,
      css: value.css,
      favicon: value.favicon,
    }
  }
}

#[napi(object)]
pub struct JsBeforeAssetTagGenerationData {
  pub assets: JsHtmlPluginAssets,
  pub output_name: String,
}

impl From<JsBeforeAssetTagGenerationData> for BeforeAssetTagGenerationData {
  fn from(value: JsBeforeAssetTagGenerationData) -> Self {
    Self {
      assets: value.assets.into(),
      output_name: value.output_name,
    }
  }
}

impl From<BeforeAssetTagGenerationData> for JsBeforeAssetTagGenerationData {
  fn from(value: BeforeAssetTagGenerationData) -> Self {
    Self {
      assets: value.assets.into(),
      output_name: value.output_name,
    }
  }
}

#[napi(object)]
pub struct JsHtmlPluginAssetTags {
  pub scripts: Vec<JsHtmlPluginTag>,
  pub styles: Vec<JsHtmlPluginTag>,
  pub meta: Vec<JsHtmlPluginTag>,
}

impl From<HtmlPluginAssetTags> for JsHtmlPluginAssetTags {
  fn from(value: HtmlPluginAssetTags) -> Self {
    Self {
      scripts: value
        .scripts
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      styles: value
        .styles
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      meta: value
        .meta
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
    }
  }
}

impl From<JsHtmlPluginAssetTags> for HtmlPluginAssetTags {
  fn from(value: JsHtmlPluginAssetTags) -> Self {
    Self {
      scripts: value
        .scripts
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      styles: value
        .styles
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      meta: value
        .meta
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
    }
  }
}

#[napi(object)]
pub struct JsAlterAssetTagsData {
  pub asset_tags: JsHtmlPluginAssetTags,
  pub output_name: String,
  pub public_path: String,
}

impl From<AlterAssetTagsData> for JsAlterAssetTagsData {
  fn from(value: AlterAssetTagsData) -> Self {
    Self {
      asset_tags: value.asset_tags.into(),
      output_name: value.output_name,
      public_path: value.public_path,
    }
  }
}

impl From<JsAlterAssetTagsData> for AlterAssetTagsData {
  fn from(value: JsAlterAssetTagsData) -> Self {
    Self {
      asset_tags: value.asset_tags.into(),
      output_name: value.output_name,
      public_path: value.public_path,
    }
  }
}

#[napi(object)]
pub struct JsAlterAssetTagGroupsData {
  pub head_tags: Vec<JsHtmlPluginTag>,
  pub body_tags: Vec<JsHtmlPluginTag>,
  pub public_path: String,
  pub output_name: String,
}

impl From<AlterAssetTagGroupsData> for JsAlterAssetTagGroupsData {
  fn from(value: AlterAssetTagGroupsData) -> Self {
    Self {
      head_tags: value
        .head_tags
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      body_tags: value
        .body_tags
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      public_path: value.public_path,
      output_name: value.output_name,
    }
  }
}

impl From<JsAlterAssetTagGroupsData> for AlterAssetTagGroupsData {
  fn from(value: JsAlterAssetTagGroupsData) -> Self {
    Self {
      head_tags: value
        .head_tags
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      body_tags: value
        .body_tags
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      public_path: value.public_path,
      output_name: value.output_name,
    }
  }
}

#[napi(object)]
pub struct JsAfterTemplateExecutionData {
  pub html: String,
  pub head_tags: Vec<JsHtmlPluginTag>,
  pub body_tags: Vec<JsHtmlPluginTag>,
  pub output_name: String,
}

impl From<AfterTemplateExecutionData> for JsAfterTemplateExecutionData {
  fn from(value: AfterTemplateExecutionData) -> Self {
    Self {
      html: value.html,
      head_tags: value
        .head_tags
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      body_tags: value
        .body_tags
        .into_iter()
        .map(JsHtmlPluginTag::from)
        .collect::<Vec<_>>(),
      output_name: value.output_name,
    }
  }
}

impl From<JsAfterTemplateExecutionData> for AfterTemplateExecutionData {
  fn from(value: JsAfterTemplateExecutionData) -> Self {
    Self {
      html: value.html,
      head_tags: value
        .head_tags
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      body_tags: value
        .body_tags
        .into_iter()
        .map(HtmlPluginTag::from)
        .collect::<Vec<_>>(),
      output_name: value.output_name,
    }
  }
}

#[napi(object)]
pub struct JsBeforeEmitData {
  pub html: String,
  pub output_name: String,
}

impl From<BeforeEmitData> for JsBeforeEmitData {
  fn from(value: BeforeEmitData) -> Self {
    Self {
      html: value.html,
      output_name: value.output_name,
    }
  }
}

impl From<JsBeforeEmitData> for BeforeEmitData {
  fn from(value: JsBeforeEmitData) -> Self {
    Self {
      html: value.html,
      output_name: value.output_name,
    }
  }
}

#[napi(object)]
pub struct JsAfterEmitData {
  pub output_name: String,
}

impl From<AfterEmitData> for JsAfterEmitData {
  fn from(value: AfterEmitData) -> Self {
    Self {
      output_name: value.output_name,
    }
  }
}

impl From<JsAfterEmitData> for AfterEmitData {
  fn from(value: JsAfterEmitData) -> Self {
    Self {
      output_name: value.output_name,
    }
  }
}
