use rspack_core::CompilationId;
use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

use crate::{
  asset::{HtmlPluginAssetTags, HtmlPluginAssets},
  tag::HtmlPluginTag,
};

#[derive(Clone, Debug)]
pub struct BeforeAssetTagGenerationData {
  pub assets: HtmlPluginAssets,
  pub output_name: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct AlterAssetTagsData {
  pub asset_tags: HtmlPluginAssetTags,
  pub output_name: String,
  pub public_path: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct AlterAssetTagGroupsData {
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub public_path: String,
  pub output_name: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct AfterTemplateExecutionData {
  pub html: String,
  pub head_tags: Vec<HtmlPluginTag>,
  pub body_tags: Vec<HtmlPluginTag>,
  pub output_name: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct BeforeEmitData {
  pub html: String,
  pub output_name: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct AfterEmitData {
  pub output_name: String,
  pub compilation_id: CompilationId,
  pub uid: Option<u32>,
}

define_hook!(HtmlPluginBeforeAssetTagGeneration: SeriesWaterfall(data: BeforeAssetTagGenerationData) -> BeforeAssetTagGenerationData);
define_hook!(HtmlPluginAlterAssetTags: SeriesWaterfall(data: AlterAssetTagsData) -> AlterAssetTagsData);
define_hook!(HtmlPluginAlterAssetTagGroups: SeriesWaterfall(data: AlterAssetTagGroupsData) -> AlterAssetTagGroupsData);
define_hook!(HtmlPluginAfterTemplateExecution: SeriesWaterfall(data: AfterTemplateExecutionData) -> AfterTemplateExecutionData);
define_hook!(HtmlPluginBeforeEmit: SeriesWaterfall(data: BeforeEmitData) -> BeforeEmitData);
define_hook!(HtmlPluginAfterEmit: SeriesWaterfall(data: AfterEmitData) -> AfterEmitData);

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct HtmlPluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub before_asset_tag_generation: HtmlPluginBeforeAssetTagGenerationHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub alter_asset_tags: HtmlPluginAlterAssetTagsHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub alter_asset_tag_groups: HtmlPluginAlterAssetTagGroupsHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub after_template_execution: HtmlPluginAfterTemplateExecutionHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub before_emit: HtmlPluginBeforeEmitHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub after_emit: HtmlPluginAfterEmitHook,
}
