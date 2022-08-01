use anyhow::Result;
use async_trait::async_trait;

use rspack_core::{
  Asset, AssetContent, Plugin, PluginContext, PluginRenderManifestHookOutput,
  PluginRenderRuntimeHookOutput, RenderManifestArgs, RenderRuntimeArgs, RuntimeSourceNode,
};

mod common;
mod web;

#[derive(Debug)]
pub struct RuntimePlugin {}

#[async_trait]
impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "runtime"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn render_runtime(
    &self,
    _ctx: PluginContext,
    args: RenderRuntimeArgs,
  ) -> PluginRenderRuntimeHookOutput {
    let compilation = args.compilation;
    let public_path = &compilation.options.output.public_path;

    let has_dynamic_chunk = true;
    // if the complition has dynamic chunk
    //Todo we need a dynamic chunk tag to judge it

    // common runtime
    let mut sources: Vec<RuntimeSourceNode> = vec![];
    sources.push(generate_common_init_runtime!());
    sources.push(generate_common_module_and_chunk_data!());
    sources.push(generate_common_check_by_id!());
    sources.push(generate_common_public_path!(public_path));

    if let rspack_core::Target::String(target) = &compilation.options.target {
      if target.eq("web") {
        sources.push(generate_web_rspack_require!());
        sources.push(generate_web_rspack_register!());
        if has_dynamic_chunk {
          sources.push(generate_common_dynamic_data!());
          sources.push(generate_web_dynamic_get_chunk_url!());
          sources.push(generate_web_dynamic_require!());
          sources.push(generate_web_dynamic_load_script!());
          sources.push(generate_web_dynamic_load_style!());
        }
      }
    }

    Ok(sources)
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let runtime = &compilation.runtime;

    Ok(vec![Asset::new(
      AssetContent::String(runtime.generate()),
      String::from("runtime.js"),
    )])
  }
}
