use napi_derive::napi;
use rspack_binding_values::JsFilename;
use rspack_plugin_dll::{
  DllEntryPluginOptions, DllReferenceAgencyPluginOptions, LibManifestPluginOptions,
};

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
  pub r#type: Option<String>,
}

impl From<RawLibManifestPluginOptions> for LibManifestPluginOptions {
  fn from(value: RawLibManifestPluginOptions) -> Self {
    let RawLibManifestPluginOptions {
      context,
      entry_only,
      name,
      path,
      r#type,
      format,
    } = value;

    Self {
      context: context.map(|c| c.into()),
      format,
      entry_only,
      name: name.map(|n| n.into()),
      path: path.into(),
      r#type,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawDllReferenceAgencyPluginOptions {
  pub context: Option<String>,
  pub name: Option<String>,
  pub extensions: Vec<String>,
  pub scope: Option<String>,
  pub source_type: Option<String>,
  pub r#type: String,
  pub content: Option<String>,
  pub manifest: Option<String>,
}

impl From<RawDllReferenceAgencyPluginOptions> for DllReferenceAgencyPluginOptions {
  fn from(value: RawDllReferenceAgencyPluginOptions) -> Self {
    let RawDllReferenceAgencyPluginOptions {
      context,
      name,
      extensions,
      scope,
      source_type,
      r#type,
      content,
      manifest,
    } = value;

    Self {
      context: context.map(|ctx| ctx.into()),
      name,
      extensions,
      scope,
      source_type,
      r#type,
      content,
      manifest,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawFlagAllModulesAsUsedPluginOptions {
  pub explanation: String,
}
