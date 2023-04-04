use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CrossOriginLoading, LibraryAuxiliaryComment, LibraryName, LibraryOptions,
  OutputOptions, PluginExt, WasmLoading,
};
use serde::Deserialize;

use crate::RawOptionsApply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryName {
  pub amd: Option<String>,
  pub commonjs: Option<String>,
  pub root: Option<Vec<String>>,
}

impl From<RawLibraryName> for LibraryName {
  fn from(value: RawLibraryName) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryAuxiliaryComment {
  pub root: Option<String>,
  pub commonjs: Option<String>,
  pub commonjs2: Option<String>,
  pub amd: Option<String>,
}

impl From<RawLibraryAuxiliaryComment> for LibraryAuxiliaryComment {
  fn from(value: RawLibraryAuxiliaryComment) -> Self {
    Self {
      amd: value.amd,
      commonjs: value.commonjs,
      root: value.root,
      commonjs2: value.commonjs2,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLibraryOptions {
  pub name: Option<RawLibraryName>,
  pub export: Option<Vec<String>>,
  // webpack type
  pub library_type: String,
  pub umd_named_define: Option<bool>,
  pub auxiliary_comment: Option<RawLibraryAuxiliaryComment>,
}

impl From<RawLibraryOptions> for LibraryOptions {
  fn from(value: RawLibraryOptions) -> Self {
    Self {
      name: value.name.map(Into::into),
      export: value.export,
      library_type: value.library_type,
      umd_named_define: value.umd_named_define,
      auxiliary_comment: value.auxiliary_comment.map(Into::into),
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: String,
  pub public_path: String,
  pub asset_module_filename: String,
  pub wasm_loading: String,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: String,
  pub chunk_filename: String,
  // TODO false type
  #[napi(ts_type = "\"anonymous\" | \"use-credentials\"")]
  pub cross_origin_loading: String,
  pub css_filename: String,
  pub css_chunk_filename: String,
  pub unique_name: String,
  pub library: Option<RawLibraryOptions>,
  pub strict_module_error_handling: bool,
  pub enabled_library_types: Option<Vec<String>>,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
}

impl RawOptionsApply for RawOutputOptions {
  type Options = OutputOptions;
  fn apply(self, plugins: &mut Vec<BoxPlugin>) -> Result<OutputOptions, rspack_error::Error> {
    self.apply_library_plugin(plugins);
    for wasm_loading in self.enabled_wasm_loading_types {
      plugins.push(rspack_plugin_wasm::enable_wasm_loading_plugin(
        wasm_loading.as_str().into(),
      ))
    }

    Ok(OutputOptions {
      path: self.path.into(),
      public_path: self.public_path.into(),
      asset_module_filename: self.asset_module_filename.into(),
      wasm_loading: match self.wasm_loading.as_str() {
        "false" => WasmLoading::Disable,
        i => WasmLoading::Enable(i.into()),
      },
      webassembly_module_filename: self.webassembly_module_filename.into(),
      unique_name: self.unique_name,
      filename: self.filename.into(),
      chunk_filename: self.chunk_filename.into(),
      cross_origin_loading: match self.cross_origin_loading.as_str() {
        "false" => CrossOriginLoading::Disable,
        i => CrossOriginLoading::Enable(i.into()),
      },
      css_filename: self.css_filename.into(),
      css_chunk_filename: self.css_chunk_filename.into(),
      library: self.library.map(Into::into),
      strict_module_error_handling: self.strict_module_error_handling,
      enabled_library_types: self.enabled_library_types,
      global_object: self.global_object,
      import_function_name: self.import_function_name,
      iife: self.iife,
      module: self.module,
    })
  }
}

impl RawOutputOptions {
  fn apply_library_plugin(&self, plugins: &mut Vec<BoxPlugin>) {
    if let Some(enabled_library_types) = &self.enabled_library_types {
      for library in enabled_library_types {
        match library.as_str() {
          "var" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec![],
                  declare: true,
                  unnamed: rspack_plugin_library::Unnamed::Error,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "assign-properties" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec![],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Error,
                  named: Some(rspack_plugin_library::Named::Copy),
                },
              )
              .boxed(),
            );
          }
          "assign" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec![],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Error,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "this" | "window" | "self" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec![library.to_string()],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Copy,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "global" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec![self.global_object.clone()],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Copy,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "commonjs" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec!["exports".to_string()],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Copy,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "commonjs-static" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec!["exports".to_string()],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Static,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "commonjs2" | "commonjs-module" => {
            plugins.push(
              rspack_plugin_library::AssignLibraryPlugin::new(
                rspack_plugin_library::AssignLibraryPluginOptions {
                  library_type: library.clone(),
                  prefix: vec!["module".to_string(), "exports".to_string()],
                  declare: false,
                  unnamed: rspack_plugin_library::Unnamed::Assign,
                  named: None,
                },
              )
              .boxed(),
            );
          }
          "umd" | "umd2" => {
            plugins.push(rspack_plugin_library::UmdLibraryPlugin::new("umd2".eq(library)).boxed());
          }
          "amd" | "amd-require" => {
            plugins.push(
              rspack_plugin_library::AmdLibraryPlugin::new("amd-require".eq(library)).boxed(),
            );
          }
          _ => {}
        }
      }
    }
  }
}
