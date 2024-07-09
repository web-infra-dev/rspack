use std::path::PathBuf;

use rspack_core::BoxDependency;
use rspack_plugin_javascript::{visitors::JavascriptParser, JavascriptParserPlugin};
use rspack_util::fx_hash::FxDashMap;

use crate::css_dependency::CssDependency;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExtractJsonData {
  pub identifier: String,
  pub content: String,
  pub context: String,
  pub media: String,
  pub supports: String,
  pub source_map: String,
  pub identifier_index: u32,
  pub filepath: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssExtractJsonDataList(pub Vec<CssExtractJsonData>);

#[derive(Debug, Default)]
pub struct PluginCssExtractParserPlugin {
  cache: FxDashMap<CssExtractJsonDataList, Vec<BoxDependency>>,
}

impl JavascriptParserPlugin for PluginCssExtractParserPlugin {
  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    let deps = if let Some(additional_data) = parser.additional_data.get::<CssExtractJsonDataList>()
    {
      if let Some(deps) = self.cache.get(additional_data) {
        deps.clone()
      } else {
        let deps = additional_data
          .0
          .iter()
          .enumerate()
          .map(
            |(
              index,
              CssExtractJsonData {
                identifier,
                content,
                context,
                media,
                supports,
                source_map,
                identifier_index,
                ..
              },
            )| {
              Box::new(CssDependency::new(
                identifier.into(),
                content.clone(),
                context.clone(),
                media.clone(),
                supports.clone(),
                source_map.clone(),
                *identifier_index,
                index as u32,
                parser.build_info.cacheable,
                parser.build_info.file_dependencies.clone(),
                parser.build_info.context_dependencies.clone(),
                parser.build_info.missing_dependencies.clone(),
                parser.build_info.build_dependencies.clone(),
              )) as BoxDependency
            },
          )
          .collect::<Vec<_>>();
        self.cache.insert(additional_data.clone(), deps.clone());
        deps
      }
    } else {
      vec![]
    };
    parser.dependencies.extend(deps);
    None
  }
}
