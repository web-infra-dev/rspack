use napi::Either;
use napi_derive::napi;
use rspack_core::ModuleId;
use rspack_plugin_dll::{
  DllEntryPluginOptions, DllManifest, DllManifestContent, DllManifestContentItem,
  DllManifestContentItemExports, DllReferenceAgencyPluginOptions, LibManifestPluginOptions,
};
use rustc_hash::FxHashMap as HashMap;
use swc_core::atoms::Atom;

use crate::{JsBuildMeta, JsFilename};

#[derive(Debug)]
#[napi(object)]
pub struct RawDllEntryPluginOptions {
  pub context: String,
  pub entries: Vec<String>,
  pub name: String,
}

impl From<RawDllEntryPluginOptions> for DllEntryPluginOptions {
  fn from(value: RawDllEntryPluginOptions) -> Self {
    let RawDllEntryPluginOptions {
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

#[napi(object, object_to_js = false)]
pub struct RawDllReferenceAgencyPluginOptions {
  pub context: Option<String>,
  pub name: Option<String>,
  pub extensions: Vec<String>,
  pub scope: Option<String>,
  pub source_type: Option<String>,
  pub r#type: String,
  pub content: Option<HashMap<String, RawDllManifestContentItem>>,
  pub manifest: Option<RawDllManifest>,
}

#[napi(object, object_to_js = false)]
pub struct RawDllManifestContentItem {
  pub build_meta: Option<JsBuildMeta>,
  #[napi(ts_type = "string[] | true")]
  pub exports: Option<Either<Vec<String>, bool>>,
  pub id: Option<Either<u32, String>>,
}

impl From<RawDllManifestContentItem> for DllManifestContentItem {
  fn from(value: RawDllManifestContentItem) -> Self {
    let raw_exports = value.exports;

    let exports = raw_exports.map(|exports| match exports {
      Either::A(seq) => {
        DllManifestContentItemExports::Vec(seq.into_iter().map(Atom::from).collect::<Vec<_>>())
      }
      Either::B(bool) => {
        if bool {
          DllManifestContentItemExports::True
        } else {
          unreachable!()
        }
      }
    });

    Self {
      build_meta: value.build_meta.map(|meta| meta.into()).unwrap_or_default(),
      exports,
      id: value.id.map(|id| match id {
        Either::A(n) => ModuleId::from(n),
        Either::B(s) => ModuleId::from(s),
      }),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct RawDllManifest {
  pub content: HashMap<String, RawDllManifestContentItem>,
  pub name: Option<String>,
  pub r#type: Option<String>,
}

impl From<RawDllManifest> for DllManifest {
  fn from(value: RawDllManifest) -> Self {
    Self {
      content: value
        .content
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect::<DllManifestContent>(),
      name: value.name,
      r#type: value.r#type,
    }
  }
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
      content: content.map(|c| {
        c.into_iter()
          .map(|(k, v)| (k, v.into()))
          .collect::<DllManifestContent>()
      }),
      manifest: manifest.map(|m| m.into()),
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawFlagAllModulesAsUsedPluginOptions {
  pub explanation: String,
}
