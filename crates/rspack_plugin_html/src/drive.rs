use rspack_hook::define_hook;
use serde::{Deserialize, Serialize};

use crate::visitors::tag::HtmlPluginTag;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlPluginAssets {
  pub public_path: String,
  pub js: Vec<String>,
  pub css: Vec<String>,
  pub favicon: Option<String>,
  // manifest: Option<String>,
}

#[derive(Clone)]
pub struct BeforeAssetTagGenerationData {
  pub assets: HtmlPluginAssets,
  pub output_name: String,
}

#[derive(Clone, Default)]
pub struct HtmlPluginAssetTags {
  pub scripts: Vec<HtmlPluginTag>,
  pub styles: Vec<HtmlPluginTag>,
  pub meta: Vec<HtmlPluginTag>,
}

#[derive(Clone)]
pub struct AlterAssetTagsData {
  pub asset_tags: HtmlPluginAssetTags,
  pub output_name: String,
  pub public_path: String,
}

#[derive(Clone)]
pub struct AlterAssetTagGroupsData {
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub public_path: String,
  pub output_name: String,
}

#[derive(Clone)]
pub struct AfterTemplateExecutionData {
  pub html: String,
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub output_name: String,
}

#[derive(Clone)]
pub struct BeforeEmitData {
  pub html: String,
  pub output_name: String,
}

#[derive(Clone)]
pub struct AfterEmitData {
  pub output_name: String,
}

define_hook!(HtmlPluginBeforeAssetTagGeneration: AsyncSeriesWaterfall(data: BeforeAssetTagGenerationData) -> BeforeAssetTagGenerationData);
define_hook!(HtmlPluginAlterAssetTags: AsyncSeriesWaterfall(data: AlterAssetTagsData) -> AlterAssetTagsData);
define_hook!(HtmlPluginAlterAssetTagGroups: AsyncSeriesWaterfall(data: AlterAssetTagGroupsData) -> AlterAssetTagGroupsData);
define_hook!(HtmlPluginAfterTemplateExecution: AsyncSeriesWaterfall(data: AfterTemplateExecutionData) -> AfterTemplateExecutionData);
define_hook!(HtmlPluginBeforeEmit: AsyncSeriesWaterfall(data: BeforeEmitData) -> BeforeEmitData);
define_hook!(HtmlPluginAfterEmit: AsyncSeriesWaterfall(data: AfterEmitData) -> AfterEmitData);

#[derive(Debug, Default)]
pub struct HtmlPluginHooks {
  pub before_asset_tag_generation: HtmlPluginBeforeAssetTagGenerationHook,
  pub alter_asset_tags: HtmlPluginAlterAssetTagsHook,
  pub alter_asset_tag_groups: HtmlPluginAlterAssetTagGroupsHook,
  pub after_template_execution: HtmlPluginAfterTemplateExecutionHook,
  pub before_emit: HtmlPluginBeforeEmitHook,
  pub after_emit: HtmlPluginAfterEmitHook,
}
