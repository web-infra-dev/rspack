use cow_utils::CowUtils;
use napi::Either;
use napi_derive::napi;
use rspack_core::CompilationId;
use rspack_plugin_html::{
  AfterEmitData, AfterTemplateExecutionData, AlterAssetTagGroupsData, AlterAssetTagsData,
  BeforeAssetTagGenerationData, BeforeEmitData,
  asset::{HtmlPluginAssetTags, HtmlPluginAssets},
  tag::{HtmlPluginAttribute, HtmlPluginTag},
};
use rustc_hash::FxHashMap as HashMap;

#[napi(object)]
pub struct JsHtmlPluginTag {
  pub tag_name: String,
  pub attributes: HashMap<String, Option<Either<String, bool>>>,
  pub void_tag: bool,
  #[napi(js_name = "innerHTML")]
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
            x.attr_name.clone(),
            if let Some(attr_value) = &x.attr_value {
              Some(Either::A(attr_value.to_owned()))
            } else {
              Some(Either::B(true))
            },
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
        .filter_map(|(key, value)| {
          value.as_ref().and_then(|v| match v {
            Either::A(x) => Some(HtmlPluginAttribute {
              attr_name: key.cow_to_ascii_lowercase().into_owned(),
              attr_value: Some(x.to_owned()),
            }),
            Either::B(x) => {
              if *x {
                Some(HtmlPluginAttribute {
                  attr_name: key.cow_to_ascii_lowercase().into_owned(),
                  attr_value: None,
                })
              } else {
                None
              }
            }
          })
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
  pub js_integrity: Option<Vec<Option<String>>>,
  pub css_integrity: Option<Vec<Option<String>>>,
}

impl From<HtmlPluginAssets> for JsHtmlPluginAssets {
  fn from(value: HtmlPluginAssets) -> Self {
    Self {
      public_path: value.public_path,
      js: value.js,
      css: value.css,
      favicon: value.favicon,
      js_integrity: value.js_integrity,
      css_integrity: value.css_integrity,
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
      js_integrity: value.js_integrity,
      css_integrity: value.css_integrity,
    }
  }
}

#[napi(object)]
pub struct JsBeforeAssetTagGenerationData {
  pub assets: JsHtmlPluginAssets,
  pub output_name: String,
  pub compilation_id: u32,
  pub uid: Option<u32>,
}

impl From<JsBeforeAssetTagGenerationData> for BeforeAssetTagGenerationData {
  fn from(value: JsBeforeAssetTagGenerationData) -> Self {
    Self {
      assets: value.assets.into(),
      output_name: value.output_name,
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}

impl From<BeforeAssetTagGenerationData> for JsBeforeAssetTagGenerationData {
  fn from(value: BeforeAssetTagGenerationData) -> Self {
    Self {
      assets: value.assets.into(),
      output_name: value.output_name,
      compilation_id: value.compilation_id.0,
      uid: value.uid,
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
  pub compilation_id: u32,
  pub uid: Option<u32>,
}

impl From<AlterAssetTagsData> for JsAlterAssetTagsData {
  fn from(value: AlterAssetTagsData) -> Self {
    Self {
      asset_tags: value.asset_tags.into(),
      output_name: value.output_name,
      public_path: value.public_path,
      compilation_id: value.compilation_id.0,
      uid: value.uid,
    }
  }
}

impl From<JsAlterAssetTagsData> for AlterAssetTagsData {
  fn from(value: JsAlterAssetTagsData) -> Self {
    Self {
      asset_tags: value.asset_tags.into(),
      output_name: value.output_name,
      public_path: value.public_path,
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}

#[napi(object)]
pub struct JsAlterAssetTagGroupsData {
  pub head_tags: Vec<JsHtmlPluginTag>,
  pub body_tags: Vec<JsHtmlPluginTag>,
  pub public_path: String,
  pub output_name: String,
  pub compilation_id: u32,
  pub uid: Option<u32>,
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
      compilation_id: value.compilation_id.0,
      uid: value.uid,
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
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}

#[napi(object)]
pub struct JsAfterTemplateExecutionData {
  pub html: String,
  pub head_tags: Vec<JsHtmlPluginTag>,
  pub body_tags: Vec<JsHtmlPluginTag>,
  pub output_name: String,
  pub compilation_id: u32,
  pub uid: Option<u32>,
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
      compilation_id: value.compilation_id.0,
      uid: value.uid,
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
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}

#[napi(object)]
pub struct JsBeforeEmitData {
  pub html: String,
  pub output_name: String,
  pub compilation_id: u32,
  pub uid: Option<u32>,
}

impl From<BeforeEmitData> for JsBeforeEmitData {
  fn from(value: BeforeEmitData) -> Self {
    Self {
      html: value.html,
      output_name: value.output_name,
      compilation_id: value.compilation_id.0,
      uid: value.uid,
    }
  }
}

impl From<JsBeforeEmitData> for BeforeEmitData {
  fn from(value: JsBeforeEmitData) -> Self {
    Self {
      html: value.html,
      output_name: value.output_name,
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}

#[napi(object)]
pub struct JsAfterEmitData {
  pub output_name: String,
  pub compilation_id: u32,
  pub uid: Option<u32>,
}

impl From<AfterEmitData> for JsAfterEmitData {
  fn from(value: AfterEmitData) -> Self {
    Self {
      output_name: value.output_name,
      compilation_id: value.compilation_id.0,
      uid: value.uid,
    }
  }
}

impl From<JsAfterEmitData> for AfterEmitData {
  fn from(value: JsAfterEmitData) -> Self {
    Self {
      output_name: value.output_name,
      compilation_id: CompilationId(value.compilation_id),
      uid: value.uid,
    }
  }
}
