use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeriesBail, AsyncSeriesBail2};

static URI_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?is)^data:([^;,]+)?((?:;[^;,]+)*?)(?:;(base64))?,(.*)$").expect("Invalid Regex")
});

#[plugin]
#[derive(Debug, Default)]
pub struct DataUriPlugin;

#[plugin_hook(AsyncSeriesBail2<ModuleFactoryCreateData, ResourceData, bool> for DataUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_data()
    && let Some(captures) = URI_REGEX.captures(&resource_data.resource)
  {
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
    resource_data.set_mimetype(mimetype);
    resource_data.set_parameters(parameters);
    resource_data.set_encoding(encoding);
    resource_data.set_encoded_content(encoded_content);
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(AsyncSeriesBail<ResourceData, Content> for DataUriPlugin)]
async fn read_resource(&self, resource_data: &mut ResourceData) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_data()
    && let Some(captures) = URI_REGEX.captures(&resource_data.resource)
  {
    let body = captures.get(4).expect("should have data uri body").as_str();
    let is_base64 = captures.get(3).is_some();
    if is_base64 && let Some(cleaned) = rspack_base64::clean_base64(body) {
      return match rspack_base64::decode_to_vec(cleaned.as_bytes()) {
        Ok(buffer) => Ok(Some(Content::Buffer(buffer))),
        Err(_) => Ok(Some(Content::String(resource_data.resource.to_string()))),
      };
    }
    if !body.is_ascii() {
      return Ok(Some(Content::Buffer(
        urlencoding::decode_binary(body.as_bytes()).into_owned(),
      )));
    } else {
      return Ok(Some(Content::Buffer(body.bytes().collect())));
    }
  }
  Ok(None)
}

#[async_trait::async_trait]
impl Plugin for DataUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.DataUriPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(resolve_for_scheme::new(self));
    ctx
      .context
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
