// use napi;

use napi::bindgen_prelude::*;
use rspack_binding_options::*;

use crate::{
  create_node_adapter_from_plugin_callbacks, get_named_property_value_string, PluginCallbacks,
};

#[napi]
pub struct Rspack(rspack::Compiler);

#[napi]
impl Rspack {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    mut options: RawOptions,
    plugin_callbacks: Option<PluginCallbacks>,
  ) -> Result<Self> {
    NAPI_ENV.with(|napi_env| *napi_env.borrow_mut() = Some(env.raw()));

    #[cfg(debug_assertions)]
    {
      if let Some(module) = options.module.as_mut() {
        for rule in &mut module.rules {
          if let Some(uses) = rule.uses.as_mut() {
            for item in uses {
              if let Some(loader) = item.loader.as_ref() {
                // let (env_ptr, loader_ptr) = unsafe { (env.raw(), loader.raw()) };
                if let Ok(display_name) =
                  get_named_property_value_string(env, loader, "displayName")
                {
                  item.__loader_name = Some(display_name);
                } else if let Ok(name) = get_named_property_value_string(env, loader, "name") {
                  item.__loader_name = Some(name);
                }
              }
            }
          }
        }
      }
    }
    let node_adapter = create_node_adapter_from_plugin_callbacks(env, plugin_callbacks)?;
    let mut compiler_options =
      normalize_bundle_options(options).map_err(|e| Error::from_reason(format!("{:?}", e)))?;
    if let Some(node_adapter) = node_adapter {
      compiler_options
        .plugins
        .push(Box::new(node_adapter) as Box<dyn rspack_core::Plugin>);
    }
    // TODO: this way or passing env as context to `normalize_bundle_option`?
    compiler_options
      .module
      .rules
      .iter_mut()
      .try_for_each(|rule| {
        rule.uses.iter_mut().try_for_each(|loader| {
          let casted = loader.as_any_mut();
          if let Some(adapter) = casted.downcast_mut::<NodeLoaderAdapter>() {
            adapter.unref(&env)
          } else {
            Ok(())
          }
        })
      })
      .map_err(|e| Error::from_reason(format!("failed to unref tsfn {:?}", e)))?;

    let rspack = rspack::rspack(compiler_options, vec![]);

    Ok(Self(rspack))
  }
}
