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
    args: &ResourceData,
  ) -> PluginNormalModuleFactoryResolveForSchemeOutput {
    if let Some(captures) = URI_REGEX.captures(&args.resource)
    && let Some(mimetype) = captures.get(1)
    && let Some(parameters) = captures.get(2)
    && let Some(encoding) = captures.get(3)
    && let Some(encoded_content) = captures.get(4) {
      let resource_data = args.clone();
      return Ok(Some(resource_data
        .mimetype(mimetype.as_str().to_owned())
        .parameters(parameters.as_str().to_owned())
        .encoding(encoding.as_str().to_owned())
        .encoded_content(encoded_content.as_str().to_owned())
      ))
    }
    Ok(None)
  }

  async fn read_resource(&self, resource_data: &ResourceData) -> PluginReadResourceOutput {
    if resource_data.get_scheme() == &Scheme::Data && let Some(captures) = URI_REGEX.captures(&resource_data.resource) {
      let body = captures.get(4).expect("should have data uri body").as_str();
      let is_base64 = captures.get(3).is_some();
      if is_base64 {
        let base64 = rspack_base64::base64::Base64::new();
        return Ok(Some(Content::Buffer(base64.decode_to_vec(body).map_err(|e| internal_error!(e.to_string()))?)))
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
