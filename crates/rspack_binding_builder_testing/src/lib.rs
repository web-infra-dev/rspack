#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

use rspack_binding_builder_macros::register_plugin;
use rspack_core::{ApplyContext, BoxPlugin, CompilerOptions, Plugin, PluginContext};
use rspack_napi::{napi, napi::bindgen_prelude::*};

#[derive(Debug)]
struct BindingBuilderTestingPlugin;

impl Plugin for BindingBuilderTestingPlugin {
  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    Ok(())
  }
}

fn get_binding_plugin(_env: Env, options: Unknown<'_>) -> Result<BoxPlugin> {
  let options = options.coerce_to_object()?;
  let foo = options.get::<String>("foo")?.unwrap();
  assert_eq!(foo, "bar".to_string());
  Ok(Box::new(BindingBuilderTestingPlugin) as BoxPlugin)
}

register_plugin!("BindingBuilderTestingPlugin", get_binding_plugin);
