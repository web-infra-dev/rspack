use rspack_core::{BoxDependency, DependencyRange};
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use rspack_util::fx_hash::FxDashMap;
use serde::Deserialize;

use crate::{css_dependency::CssDependency, plugin::PLUGIN_NAME};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssExtractJsonData {
  pub identifier: String,
  pub content: String,
  pub context: String,
  pub media: Option<String>,
  pub supports: Option<String>,
  pub source_map: Option<String>,
  pub identifier_index: u32,
  pub layer: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct PluginCssExtractParserPlugin {
  cache: FxDashMap<String, Vec<BoxDependency>>,
}

impl JavascriptParserPlugin for PluginCssExtractParserPlugin {
  fn finish(&self, parser: &mut JavascriptParser) -> Option<bool> {
    let deps = if let Some(data_str) = parser.parse_meta.remove(PLUGIN_NAME)
      && let Ok(data_str) = (data_str as Box<dyn std::any::Any>)
        .downcast::<String>()
        .map(|i| *i)
    {
      if let Some(deps) = self.cache.get(&data_str) {
        deps.clone()
      } else if let Ok(data) = serde_json::from_str::<Vec<CssExtractJsonData>>(&data_str) {
        let deps = data
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
                layer,
              },
            )| {
              Box::new(CssDependency::new(
                identifier.into(),
                parser.get_module_layer().cloned(),
                layer.clone(),
                content.clone(),
                context.clone(),
                media.clone(),
                supports.clone(),
                source_map.clone(),
                *identifier_index,
                DependencyRange::new(index as u32, (index + 1) as u32),
                parser.build_info.cacheable,
                parser.build_info.file_dependencies.clone(),
                parser.build_info.context_dependencies.clone(),
                parser.build_info.missing_dependencies.clone(),
                parser.build_info.build_dependencies.clone(),
              )) as BoxDependency
            },
          )
          .collect::<Vec<_>>();
        self.cache.insert(data_str, deps.clone());
        parser.build_info.strict = true;
        deps
      } else {
        vec![]
      }
    } else {
      vec![]
    };
    parser.add_dependencies(deps);
    None
  }
}
