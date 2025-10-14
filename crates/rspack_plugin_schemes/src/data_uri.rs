use std::sync::{Arc, LazyLock};

use regex::Regex;
use rspack_core::{
  Content, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme, NormalModuleReadResource,
  Plugin, ResourceData, Scheme,
};
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::base64;

static URI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(?i)^data:([^;,]+)?((?:;[^;,]+)*?)(?:;(base64)?)?,(.*)$").expect("Invalid Regex")
});

#[plugin]
#[derive(Debug, Default)]
pub struct DataUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for DataUriPlugin,tracing=false)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  scheme: &Scheme,
) -> Result<Option<bool>> {
  if scheme.is_data()
    && let Some(captures) = URI_REGEX.captures(resource_data.resource())
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

#[plugin_hook(NormalModuleReadResource for DataUriPlugin,tracing=false)]
async fn read_resource(
  &self,
  resource_data: &ResourceData,
  _fs: &Arc<dyn ReadableFileSystem>,
) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_data()
    && let Some(captures) = URI_REGEX.captures(resource_data.resource())
  {
    let body = captures.get(4).expect("should have data uri body").as_str();
    let is_base64 = captures.get(3).is_some();
    if is_base64 && let Some(cleaned) = base64::clean_base64(body) {
      return match base64::decode_to_vec(cleaned.as_bytes()) {
        Ok(buffer) => Ok(Some(Content::Buffer(buffer))),
        Err(_) => Ok(Some(Content::String(resource_data.resource().to_string()))),
      };
    }

    return match urlencoding::decode(body) {
      Ok(decoded_content) => Ok(Some(Content::Buffer(decoded_content.as_bytes().to_vec()))),
      Err(_) => Ok(Some(Content::Buffer(body.as_bytes().to_vec()))),
    };
  }
  Ok(None)
}

impl Plugin for DataUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.DataUriPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(resolve_for_scheme::new(self));
    ctx
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
