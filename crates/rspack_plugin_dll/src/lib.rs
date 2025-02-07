use rspack_cacheable::{
  cacheable,
  with::{AsPreset, AsVec},
};
use rspack_core::{BuildMeta, LibraryType, ModuleId};
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;
use serde::{ser::SerializeSeq, Serialize};

mod dll_entry;
mod dll_reference;
mod flag_all_modules_as_used_plugin;
mod lib_manifest_plugin;

pub type DllManifestContent = HashMap<String, DllManifestContentItem>;

#[cacheable]
#[derive(Debug, Default, Clone)]
pub enum DllManifestContentItemExports {
  #[default]
  True,
  Vec(#[cacheable(with=AsVec<AsPreset>)] Vec<Atom>),
}

impl Serialize for DllManifestContentItemExports {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      DllManifestContentItemExports::True => serializer.serialize_bool(true),
      DllManifestContentItemExports::Vec(vec) => {
        let mut seq = serializer.serialize_seq(Some(vec.len()))?;
        for item in vec {
          seq.serialize_element(item)?;
        }
        seq.end()
      }
    }
  }
}

#[cacheable]
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllManifestContentItem {
  pub build_meta: BuildMeta,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub exports: Option<DllManifestContentItemExports>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<ModuleId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DllManifest {
  pub content: DllManifestContent,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#type: Option<LibraryType>,
}

pub use dll_entry::dll_entry_plugin::{DllEntryPlugin, DllEntryPluginOptions};
pub use dll_reference::dll_reference_agency_plugin::{
  DllReferenceAgencyPlugin, DllReferenceAgencyPluginOptions,
};
pub use flag_all_modules_as_used_plugin::FlagAllModulesAsUsedPlugin;
pub use lib_manifest_plugin::{LibManifestPlugin, LibManifestPluginOptions};
