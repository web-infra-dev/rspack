use rspack_core::{
  Plugin, PluginContext, PluginNormalModuleFactoryResolveForSchemeOutput, ResourceData,
};
use rspack_error::internal_error;
use url::Url;

#[derive(Debug)]
pub struct FileUriPlugin;

#[async_trait::async_trait]
impl Plugin for FileUriPlugin {
  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: PluginContext,
    resource_data: ResourceData,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    if resource_data.get_scheme().is_file() {
      let url = Url::parse(&resource_data.resource).map_err(|e| internal_error!(e.to_string()))?;
      let path = url
        .to_file_path()
        .map_err(|_| internal_error!("Failed to get file path of {url}"))?;
      let query = url.query().map(|q| format!("?{q}"));
      let fragment = url.fragment().map(|f| format!("#{f}"));
      return Ok((
        ResourceData::new(
          format!(
            "{}{}{}",
            path.to_string_lossy(),
            query.as_deref().unwrap_or(""),
            fragment.as_deref().unwrap_or("")
          ),
          path,
        )
        .query_optional(query)
        .fragment_optional(fragment),
        true,
      ));
    }
    Ok((resource_data, false))
  }
}
