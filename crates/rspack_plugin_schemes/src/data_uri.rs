use std::borrow::Cow;

use rspack_core::{
  Content, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme, NormalModuleReadResource,
  Plugin, ResourceData, Scheme,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use winnow::{
  ascii::{alphanumeric1, till_line_ending},
  combinator::{opt, repeat, separated},
  error::ParseError,
  prelude::*,
  token::{take_till, take_while},
};

#[derive(Debug, PartialEq)]
struct DataUri<'a> {
  mimetype: Option<&'a str>,
  parameters: Vec<&'a str>,
  encoding: Option<&'a str>,
  data: &'a str,
}

fn parse_data_uri(input: &mut &str) -> PResult<DataUri> {
  let _: &str = "data:".parse_next(input)?;
  
  // Parse mimetype (optional)
  let mimetype = opt(take_till(0.., [';', ','])).parse_next(input)?;
  
  // Parse parameters (optional, multiple)
  let mut parameters = Vec::new();
  let mut encoding = None;
  
  while input.starts_with(';') {
    let _: char = ';'.parse_next(input)?;
    let param = take_till(0.., [';', ',']).parse_next(input)?;
    
    // Check if this parameter is "base64"
    if param.eq_ignore_ascii_case("base64") {
      encoding = Some(param);
    } else {
      parameters.push(param);
    }
  }
  
  // Expect comma before data
  let _: char = ','.parse_next(input)?;
  
  // Parse the remaining data
  let data = take_while(0.., |_| true).parse_next(input)?;
  
  Ok(DataUri {
    mimetype,
    parameters,
    encoding,
    data,
  })
}

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
  if scheme.is_data() {
    if let Ok(parsed) = parse_data_uri.parse(resource_data.resource.as_str()) {
      let mimetype = parsed.mimetype.unwrap_or_default().to_owned();
      let parameters = parsed.parameters.join(";");
      let encoding = parsed.encoding.unwrap_or_default().to_owned();
      let encoded_content = parsed.data.to_owned();
      
      resource_data.set_mimetype(mimetype);
      resource_data.set_parameters(parameters);
      resource_data.set_encoding(encoding);
      resource_data.set_encoded_content(encoded_content);
      return Ok(None);
    }
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for DataUriPlugin,tracing=false)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_data() {
    if let Ok(parsed) = parse_data_uri.parse(resource_data.resource.as_str()) {
      let body = parsed.data;
      let is_base64 = parsed.encoding.map_or(false, |e| e.eq_ignore_ascii_case("base64"));
      
      if is_base64 && let Some(cleaned) = rspack_base64::clean_base64(body) {
        return match rspack_base64::decode_to_vec(cleaned.as_bytes()) {
          Ok(buffer) => Ok(Some(Content::Buffer(buffer))),
          Err(_) => Ok(Some(Content::String(resource_data.resource.to_string()))),
        };
      }

      return match urlencoding::decode(body) {
        Ok(decoded_content) => Ok(Some(Content::Buffer(decoded_content.as_bytes().to_vec()))),
        Err(_) => Ok(Some(Content::Buffer(body.as_bytes().to_vec()))),
      };
    }
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
