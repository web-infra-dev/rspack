use rspack_error::Result;

use common::*;
use rspack_core::{
  AssetContent, Plugin, PluginContext, PluginRenderManifestHookOutput,
  PluginRenderRuntimeHookOutput, RenderManifestArgs, RenderManifestEntry, RenderRuntimeArgs,
  TargetOptions,
};
use web::*;

mod common;
mod web;

pub const RUNTIME_FILE_NAME: &str = "runtime";
#[derive(Debug)]
pub struct RuntimePlugin {}

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
    let public_path = compilation.options.output.public_path.public_path();

    //Todo we are not implement hash now，it will be replaced by real value later
    let has_hash = false;

    let has_dynamic_chunk = true;
    // if the complition has dynamic chunk
    //Todo we need a dynamic chunk tag to judge it

    // common runtime
    let mut sources = args.sources.to_vec();
    sources.push(generate_common_init_runtime());
    sources.push(generate_common_module_and_chunk_data());
    sources.push(generate_common_check_by_id());
    sources.push(generate_common_public_path(public_path));

    if let rspack_core::Target::Target(target) = &compilation.options.target {
      match target {
        TargetOptions::Web => {
          sources.push(generate_web_rspack_require());
          sources.push(generate_web_rspack_register());
          if has_dynamic_chunk {
            sources.push(generate_common_dynamic_data());
            sources.push(generate_web_dynamic_get_chunk_url(has_hash));
            sources.push(generate_web_dynamic_require());
            sources.push(generate_web_dynamic_load_script());
            sources.push(generate_web_dynamic_load_style());
          }
        }
        TargetOptions::Node(_) => {}
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

    Ok(vec![RenderManifestEntry::new(
      AssetContent::String(runtime.generate()),
      RUNTIME_FILE_NAME.to_string() + ".js",
    )])
  }
}
