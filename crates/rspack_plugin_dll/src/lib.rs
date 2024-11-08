use rspack_core::{BuildMeta, LibraryType};
use rspack_util::atom::Atom;
use rustc_hash::FxHashMap as HashMap;
use serde::Serialize;

mod dll_entry;
mod dll_reference;
mod flag_all_modules_as_used_plugin;
mod lib_manifest_plugin;

pub type DllManifestContent = HashMap<String, DllManifestContentItem>;

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllManifestContentItem {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub build_meta: Option<BuildMeta>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub exports: Option<Vec<Atom>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
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
