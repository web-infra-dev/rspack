use std::sync::Arc;

use rspack_core::{
  Content, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme, NormalModuleReadResource,
  Plugin, ResourceData, Scheme,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_fs::ReadableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
use tokio::task::spawn_blocking;
use url::Url;

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
    let url = Url::parse(resource_data.resource()).to_rspack_result()?;
    let path = url
      .to_file_path()
      .map_err(|_| error!("Failed to get file path of {url}"))?
      .assert_utf8();
    let query = url.query().map(|q| format!("?{q}"));
    let fragment = url.fragment().map(|f| format!("#{f}"));
    let resource = format!(
      "{}{}{}",
      path,
      query.as_deref().unwrap_or(""),
      fragment.as_deref().unwrap_or("")
    );
    *resource_data = ResourceData::new_with_path(resource, path, query, fragment);
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
    // use spawn_blocking to avoid block, see https://docs.rs/tokio/latest/src/tokio/fs/read.rs.html#48
    let result = spawn_blocking(move || fs.read_sync(resource_path_owned.as_path()))
      .await
      .map_err(|e| error!("{e}, spawn task failed"))?;
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
