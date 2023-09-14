mod amd_library_plugin;
mod assign_library_plugin;
mod export_property_plugin;
mod module_library_plugin;
mod system_library_plugin;
mod umd_library_plugin;
mod utils;

pub use amd_library_plugin::AmdLibraryPlugin;
pub use assign_library_plugin::*;
pub use export_property_plugin::ExportPropertyLibraryPlugin;
pub use module_library_plugin::ModuleLibraryPlugin;
use rspack_core::{BoxPlugin, PluginExt};
pub use system_library_plugin::SystemLibraryPlugin;
pub use umd_library_plugin::UmdLibraryPlugin;

pub fn enable_library_plugin(library_type: String, plugins: &mut Vec<BoxPlugin>) {
  match library_type.as_str() {
    "var" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec![]),
        declare: true,
        unnamed: Unnamed::Error,
        named: None,
      })
      .boxed(),
    ),
    "assign-properties" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec![]),
        declare: false,
        unnamed: Unnamed::Error,
        named: Some(Named::Copy),
      })
      .boxed(),
    ),
    "assign" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec![]),
        declare: false,
        unnamed: Unnamed::Error,
        named: None,
      })
      .boxed(),
    ),
    "this" | "window" | "self" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type: library_type.clone(),
        prefix: Prefix::Array(vec![library_type]),
        declare: false,
        unnamed: Unnamed::Copy,
        named: None,
      })
      .boxed(),
    ),
    "global" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Global,
        declare: false,
        unnamed: Unnamed::Copy,
        named: None,
      })
      .boxed(),
    ),
    "commonjs" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec!["exports".to_string()]),
        declare: false,
        unnamed: Unnamed::Copy,
        named: None,
      })
      .boxed(),
    ),
    "commonjs-static" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec!["exports".to_string()]),
        declare: false,
        unnamed: Unnamed::Static,
        named: None,
      })
      .boxed(),
    ),
    "commonjs2" | "commonjs-module" => plugins.push(
      AssignLibraryPlugin::new(AssignLibraryPluginOptions {
        library_type,
        prefix: Prefix::Array(vec!["module".to_string(), "exports".to_string()]),
        declare: false,
        unnamed: Unnamed::Assign,
        named: None,
      })
      .boxed(),
    ),
    "umd" | "umd2" => {
      plugins.push(ExportPropertyLibraryPlugin::default().boxed());
      plugins.push(UmdLibraryPlugin::new("umd2".eq(&library_type)).boxed());
    }
    "amd" | "amd-require" => {
      plugins.push(ExportPropertyLibraryPlugin::default().boxed());
      plugins.push(AmdLibraryPlugin::new("amd-require".eq(&library_type)).boxed());
    }
    "module" => {
      plugins.push(ExportPropertyLibraryPlugin::default().boxed());
      plugins.push(ModuleLibraryPlugin::default().boxed());
    }
    "system" => plugins.push(SystemLibraryPlugin::default().boxed()),
    _ => {}
  }
}
