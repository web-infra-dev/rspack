use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  Content, Plugin, PluginContext, PluginNormalModuleFactoryResolveForSchemeOutput,
  PluginReadResourceOutput, ResourceData, Scheme,
};
use rspack_error::internal_error;

static URI_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^data:([^;,]+)?((?:;[^;,]+)*?)(?:;(base64))?,(.*)$").expect("Invalid Regex")
});

#[derive(Debug)]
pub struct DataUriPlugin;

#[async_trait::async_trait]
impl Plugin for DataUriPlugin {
  async fn normal_module_factory_resolve_for_scheme(
    &self,
    _ctx: PluginContext,
    resource_data: ResourceData,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    if let Some(captures) = URI_REGEX.captures(&resource_data.resource) {
      let mimetype = captures
        .get(1)
        .map(|i| i.as_str())
        .unwrap_or_default()
        .to_owned();
      let parameters = captures
        .get(2)
        .map(|i| i.as_str())
        .unwrap_or_default()
        .to_owned();
      let encoding = captures
        .get(3)
        .map(|i| i.as_str())
        .unwrap_or_default()
        .to_owned();
      let encoded_content = captures
        .get(4)
        .map(|i| i.as_str())
        .unwrap_or_default()
        .to_owned();
      return Ok((
        resource_data
          .mimetype(mimetype)
          .parameters(parameters)
          .encoding(encoding)
          .encoded_content(encoded_content),
        false,
      ));
    }
    Ok((resource_data, false))
  }

  async fn read_resource(&self, resource_data: &ResourceData) -> PluginReadResourceOutput {
    if resource_data.get_scheme() == &Scheme::Data && let Some(captures) = URI_REGEX.captures(&resource_data.resource) {
      let body = captures.get(4).expect("should have data uri body").as_str();
      let is_base64 = captures.get(3).is_some();
      if is_base64 {
        let base64 = rspack_base64::base64::Base64::new();
        return Ok(Some(Content::Buffer(base64.decode_to_vec(body.trim()).map_err(|e| internal_error!(e.to_string()))?)))
      }
      if !body.is_ascii() {
        return Ok(Some(Content::Buffer(urlencoding::decode_binary(body.as_bytes()).into_owned())))
      } else {
        return Ok(Some(Content::Buffer(body.bytes().collect())))
      }
    }
    Ok(None)
  }
}
