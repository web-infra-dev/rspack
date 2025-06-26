#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

use rspack_binding_builder_macros::register_plugin;
use rspack_core::{ApplyContext, BoxPlugin, CompilerOptions, Plugin, PluginContext};
use rspack_napi::{napi, napi::bindgen_prelude::*};

#[derive(Debug)]
struct TestingPlugin;

impl Plugin for TestingPlugin {
  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    Ok(())
  }
}

register_plugin!("TestingPlugin", |_env, _options| {
  Ok(Box::new(TestingPlugin) as BoxPlugin)
});
