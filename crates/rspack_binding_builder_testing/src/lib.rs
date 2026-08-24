#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

use rspack_binding_builder_macros::register_plugin;
use rspack_core::{
  BoxDependency, BoxModule, BoxPlugin, CompilationId, CompilationSucceedModule, CompilerId,
  ModuleDependency, Plugin,
};
use rspack_hook::{plugin, plugin_hook};
use rspack_napi::{napi, napi::bindgen_prelude::*};
use rspack_plugin_javascript::dependency::{
  ESMExportImportedSpecifierDependency, ESMImportSpecifierDependency,
};

#[plugin]
#[derive(Debug)]
#[allow(unused)]
struct BindingBuilderTestingPlugin {
  reroute_specifiers: bool,
}

impl BindingBuilderTestingPlugin {
  fn new(reroute_specifiers: bool) -> Self {
    Self::new_inner(reroute_specifiers)
  }
}

fn rerouted_request(request: &str, imported: Option<&str>) -> Option<&'static str> {
  if request != "pkg" {
    return None;
  }

  match imported {
    Some("A") => Some("pkg/A"),
    Some("B") => Some("pkg/B"),
    _ => None,
  }
}

#[plugin_hook(CompilationSucceedModule for BindingBuilderTestingPlugin, tracing = false)]
async fn succeed_module(
  &self,
  _compiler_id: CompilerId,
  _compilation_id: CompilationId,
  _module: &mut BoxModule,
  dependencies: &mut [BoxDependency],
) -> rspack_error::Result<()> {
  for dependency in dependencies {
    let dependency = dependency.as_mut();

    if let Some(dependency) = dependency.downcast_mut::<ESMImportSpecifierDependency>() {
      if let Some(request) = rerouted_request(
        dependency.request(),
        dependency.ids().first().map(|id| id.as_str()),
      ) {
        dependency.set_request(request.into());
      }
      continue;
    }

    if let Some(dependency) = dependency.downcast_mut::<ESMExportImportedSpecifierDependency>()
      && let Some(request) = rerouted_request(
        dependency.request(),
        dependency.ids().first().map(|id| id.as_str()),
      )
    {
      dependency.set_request(request.into());
    }
  }

  Ok(())
}

impl Plugin for BindingBuilderTestingPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    if self.reroute_specifiers {
      ctx
        .compilation_hooks
        .succeed_module
        .tap(succeed_module::new(self));
    }
    Ok(())
  }
}

#[allow(unused)]
fn get_binding_plugin(_env: Env, options: Unknown<'_>) -> Result<BoxPlugin> {
  let options = options.coerce_to_object()?;
  #[allow(clippy::disallowed_names, clippy::unwrap_used)]
  let foo = options.get::<String>("foo")?.unwrap();
  assert_eq!(foo, "bar".to_string());
  let reroute_specifiers = options
    .get::<bool>("rerouteSpecifiers")?
    .unwrap_or_default();
  Ok(Box::new(BindingBuilderTestingPlugin::new(reroute_specifiers)) as BoxPlugin)
}

register_plugin!("BindingBuilderTestingPlugin", get_binding_plugin);
