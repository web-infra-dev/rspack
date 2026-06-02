use std::sync::Arc;

use rspack_core::{
  Content, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme, NormalModuleReadResource,
  Plugin, ResourceData, Scheme,
};
use rspack_error::{Result, error};
use rspack_fs::ReadableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::RspackResource;
#[cfg(all(not(target_family = "wasm"), not(feature = "codspeed")))]
use tokio::task::spawn_blocking;

#[plugin]
#[derive(Debug, Default)]
pub struct FileUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for FileUriPlugin)]
async fn normal_module_factory_resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  scheme: &Scheme,
) -> Result<Option<bool>> {
  if scheme.is_file() {
    let typed_resource = RspackResource::from_request(resource_data.resource(), None)
      .map_err(|err| error!("{err}"))?;
    let url = typed_resource
      .as_url()
      .ok_or_else(|| error!("Expected file URL resource {}", resource_data.resource()))?;
    let path = typed_resource
      .as_file_path()
      .ok_or_else(|| error!("Failed to get file path of {url}"))?
      .to_owned();
    let query = typed_resource.query.as_ref().map(ToString::to_string);
    let fragment = typed_resource.fragment.as_ref().map(ToString::to_string);
    let resource = format!(
      "{}{}{}",
      path,
      query.as_deref().unwrap_or(""),
      fragment.as_deref().unwrap_or("")
    );
    *resource_data = ResourceData::new_with_path_and_typed_resource(
      resource,
      path,
      query,
      fragment,
      Some(typed_resource),
    );
    return Ok(Some(true));
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for FileUriPlugin,tracing=false)]
async fn read_resource(
  &self,
  resource_data: &ResourceData,
  fs: &Arc<dyn ReadableFileSystem>,
) -> Result<Option<Content>> {
  let scheme = resource_data.get_scheme();
  if scheme.is_none()
    && let Some(resource_path) = resource_data.path()
    && !resource_path.as_str().is_empty()
  {
    let resource_path_owned = resource_path.to_owned();
    let fs = fs.clone();
    #[cfg(all(not(target_family = "wasm"), not(feature = "codspeed")))]
    let result = {
      // Avoid blocking the Tokio worker thread on native targets.
      spawn_blocking(move || fs.read_sync(resource_path_owned.as_path()))
        .await
        .map_err(|e| error!("{e}, spawn task failed"))?
    };
    #[cfg(all(not(target_family = "wasm"), feature = "codspeed"))]
    // Keep CodSpeed benchmark file reads on the current runtime thread to avoid
    // Tokio blocking-pool scheduling noise in simulation measurements.
    let result = fs.read_sync(resource_path_owned.as_path());
    #[cfg(target_family = "wasm")]
    // Keep WASI filesystem access on the current thread. Under node:wasi,
    // blocking workers may observe a different host-side WASI environment.
    let result = fs.read(resource_path_owned.as_path()).await;
    let result = result.map_err(|e| error!("{e}, failed to read {resource_path}"))?;
    return Ok(Some(Content::from(result)));
  }

  Ok(None)
}

impl Plugin for FileUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.FileUriPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(normal_module_factory_resolve_for_scheme::new(self));
    ctx
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
