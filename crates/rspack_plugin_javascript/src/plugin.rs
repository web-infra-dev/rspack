use crate::module::JsModule;
use crate::utils::parse_file;
use rayon::prelude::*;
use rspack_core::{
  Asset, AssetFilename, BoxModule, JobContext, ParseModuleArgs, Plugin, PluginContext,
  PluginParseModuleHookOutput, PluginRenderManifestHookOutput, SourceType,
};

use tracing::instrument;

#[derive(Debug)]
pub struct JsPlugin {}

impl Plugin for JsPlugin {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<rspack_core::SourceType>> {
    Some(vec![
      SourceType::Js,
      SourceType::Jsx,
      SourceType::Ts,
      SourceType::Tsx,
    ])
  }

  #[instrument(skip_all)]
  fn parse_module(
    &self,
    ctx: PluginContext<&mut JobContext>,
    args: ParseModuleArgs,
  ) -> PluginParseModuleHookOutput {
    let source_type = *ctx
      .context
      .source_type
      .as_ref()
      .ok_or_else(|| anyhow::format_err!("TODO"))?;
    let ast = parse_file(args.source, args.uri, &source_type);
    Ok(Box::new(JsModule {
      ast,
      uri: args.uri.to_string(),
      source_type,
    }))
  }

  #[instrument(skip_all)]
  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;
    let ordered_modules = chunk.ordered_modules(module_graph);
    let code = ordered_modules
      .par_iter()
      .filter(|module| {
        matches!(
          module.source_type,
          SourceType::Js | SourceType::Ts | SourceType::Tsx | SourceType::Jsx
        )
      })
      .map(|module| module.module.render(module, compilation))
      .chain([{
        if chunk.kind.is_entry() {
          format!(
            "rs.require(\"{}\")",
            ordered_modules
              .last()
              .ok_or_else(|| anyhow::format_err!("TODO:"))?
              .id
              .as_str()
          )
        } else {
          String::new()
        }
      }])
      .fold(String::new, |mut output, cur| {
        output += &cur;
        output
      })
      .collect();
    Ok(vec![Asset::new(
      code,
      AssetFilename::Static(format!("{}.js", args.chunk_id)),
    )])
  }
}
