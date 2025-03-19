use rspack_core::CompilationId;
use rspack_hook::define_hook;

use crate::{
  asset::{HtmlPluginAssetTags, HtmlPluginAssets},
  tag::HtmlPluginTag,
};

#[derive(Clone, Debug)]
pub struct BeforeAssetTagGenerationData {
  pub assets: HtmlPluginAssets,
  pub output_name: String,
  pub compilation_id: CompilationId,
}

#[derive(Clone, Debug)]
pub struct AlterAssetTagsData {
  pub asset_tags: HtmlPluginAssetTags,
  pub output_name: String,
  pub public_path: String,
  pub compilation_id: CompilationId,
}

#[derive(Clone, Debug)]
pub struct AlterAssetTagGroupsData {
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub public_path: String,
  pub output_name: String,
  pub compilation_id: CompilationId,
}

#[derive(Clone, Debug)]
pub struct AfterTemplateExecutionData {
  pub html: String,
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub output_name: String,
  pub compilation_id: CompilationId,
}

#[derive(Clone, Debug)]
pub struct BeforeEmitData {
  pub html: String,
  pub output_name: String,
  pub compilation_id: CompilationId,
}

#[derive(Clone, Debug)]
pub struct AfterEmitData {
  pub output_name: String,
  pub compilation_id: CompilationId,
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
