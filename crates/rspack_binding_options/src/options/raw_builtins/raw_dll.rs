use napi_derive::napi;
use rspack_binding_values::JsFilename;
use rspack_plugin_dll::{DllEntryPluginOptions, LibManifestPluginOptions};

#[derive(Debug)]
#[napi(object)]
pub struct RawDllEntyPluginOptions {
  pub context: String,
  pub entries: Vec<String>,
  pub name: String,
}

impl From<RawDllEntyPluginOptions> for DllEntryPluginOptions {
  fn from(value: RawDllEntyPluginOptions) -> Self {
    let RawDllEntyPluginOptions {
      name,
      context,
      entries,
    } = value;

    Self {
      name,
      context: context.into(),
      entries,
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawLibManifestPluginOptions {
  pub context: Option<String>,
  pub entry_only: Option<bool>,
  pub name: Option<JsFilename>,
  pub path: JsFilename,
  pub format: Option<bool>,
  pub ty: Option<String>,
}

impl From<RawLibManifestPluginOptions> for LibManifestPluginOptions {
  fn from(value: RawLibManifestPluginOptions) -> Self {
    let RawLibManifestPluginOptions {
      context,
      entry_only,
      name,
      path,
      ty,
      format,
    } = value;

    Self {
      context: context.map(|c| c.into()),
      format,
      entry_only,
      name: name.map(|n| n.into()),
      path: path.into(),
      ty,
    }
  }
}
