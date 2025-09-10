use rspack_core::{
  ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme, Plugin, ResourceData, Scheme,
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::AssertUtf8;
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

impl Plugin for FileUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.FileUriPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(normal_module_factory_resolve_for_scheme::new(self));
    Ok(())
  }
}
