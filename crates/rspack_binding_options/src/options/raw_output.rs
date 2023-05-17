use napi_derive::napi;
use rspack_core::{
  to_identifier, BoxPlugin, CrossOriginLoading, LibraryAuxiliaryComment, LibraryName,
  LibraryOptions, OutputOptions, PluginExt, TrustedTypes, WasmLoading,
};
use rspack_error::internal_error;
use serde::Deserialize;

use crate::JsLoaderRunner;
use crate::RawOptionsApply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawTrustedTypes {
  pub policy_name: Option<String>,
}

impl From<RawTrustedTypes> for TrustedTypes {
  fn from(value: RawTrustedTypes) -> Self {
    Self {
      policy_name: value.policy_name,
    }
  }
}

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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCrossOriginLoading {
  #[napi(ts_type = r#""bool" | "string""#)]
  pub r#type: String,
  pub string_payload: Option<String>,
  pub bool_payload: Option<bool>,
}

impl From<RawCrossOriginLoading> for CrossOriginLoading {
  fn from(value: RawCrossOriginLoading) -> Self {
    match value.r#type.as_str() {
      "string" => Self::Enable(
        value
          .string_payload
          .expect("should have a string_payload when RawCrossOriginLoading.type is \"string\""),
      ),
      "bool" => Self::Disable,
      _ => unreachable!(),
    }
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub path: String,
  pub clean: bool,
  pub public_path: String,
  pub asset_module_filename: String,
  pub wasm_loading: String,
  pub enabled_wasm_loading_types: Vec<String>,
  pub webassembly_module_filename: String,
  pub filename: String,
  pub chunk_filename: String,
  pub cross_origin_loading: RawCrossOriginLoading,
  pub css_filename: String,
  pub css_chunk_filename: String,
  pub hot_update_main_filename: String,
  pub hot_update_chunk_filename: String,
  pub unique_name: String,
  pub chunk_loading_global: String,
  pub library: Option<RawLibraryOptions>,
  pub strict_module_error_handling: bool,
  pub enabled_library_types: Option<Vec<String>>,
  pub global_object: String,
  pub import_function_name: String,
  pub iife: bool,
  pub module: bool,
  pub chunk_format: Option<String>,
  pub chunk_loading: Option<String>,
  pub enabled_chunk_loading_types: Option<Vec<String>>,
  pub trusted_types: Option<RawTrustedTypes>,
  pub source_map_filename: String,
}

impl RawOptionsApply for RawOutputOptions {
  type Options = OutputOptions;
  fn apply(
    self,
    plugins: &mut Vec<BoxPlugin>,
    _: &JsLoaderRunner,
  ) -> Result<OutputOptions, rspack_error::Error> {
    self.apply_chunk_format_plugin(plugins)?;
    plugins.push(rspack_plugin_runtime::RuntimePlugin {}.boxed());
    self.apply_chunk_loading_plugin(plugins)?;
    self.apply_library_plugin(plugins);
    for wasm_loading in self.enabled_wasm_loading_types {
      plugins.push(rspack_plugin_wasm::enable_wasm_loading_plugin(
        wasm_loading.as_str().into(),
      ))
    }

    Ok(OutputOptions {
      path: self.path.into(),
      clean: self.clean,
      public_path: self.public_path.into(),
      asset_module_filename: self.asset_module_filename.into(),
      wasm_loading: match self.wasm_loading.as_str() {
        "false" => WasmLoading::Disable,
        i => WasmLoading::Enable(i.into()),
      },
      webassembly_module_filename: self.webassembly_module_filename.into(),
      unique_name: self.unique_name,
      chunk_loading_global: to_identifier(&self.chunk_loading_global).to_string(),
      filename: self.filename.into(),
      chunk_filename: self.chunk_filename.into(),
      cross_origin_loading: self.cross_origin_loading.into(),
      css_filename: self.css_filename.into(),
      css_chunk_filename: self.css_chunk_filename.into(),
      hot_update_main_filename: self.hot_update_main_filename.into(),
      hot_update_chunk_filename: self.hot_update_chunk_filename.into(),
      library: self.library.map(Into::into),
      strict_module_error_handling: self.strict_module_error_handling,
      enabled_library_types: self.enabled_library_types,
      global_object: self.global_object,
      import_function_name: self.import_function_name,
      iife: self.iife,
      module: self.module,
      trusted_types: self.trusted_types.map(Into::into),
      source_map_filename: self.source_map_filename.into(),
    })
  }
}

impl RawOutputOptions {
  fn apply_chunk_format_plugin(
    &self,
    plugins: &mut Vec<BoxPlugin>,
  ) -> Result<(), rspack_error::Error> {
    if let Some(chunk_format) = &self.chunk_format {
      match chunk_format.as_str() {
        "array-push" => {
          plugins.push(rspack_plugin_runtime::ArrayPushCallbackChunkFormatPlugin {}.boxed());
        }
        "commonjs" => plugins.push(rspack_plugin_runtime::CommonJsChunkFormatPlugin {}.boxed()),
        "module" => {
          plugins.push(rspack_plugin_runtime::ModuleChunkFormatPlugin {}.boxed());
        }
        _ => {
          return Err(internal_error!(
            "Unsupported chunk format '{chunk_format}'."
          ))
        }
      }
    }
    Ok(())
  }

  fn apply_chunk_loading_plugin(
    &self,
    plugins: &mut Vec<BoxPlugin>,
  ) -> Result<(), rspack_error::Error> {
    if let Some(enabled_chunk_loading_types) = &self.enabled_chunk_loading_types {
      for chunk_loading in enabled_chunk_loading_types {
        match chunk_loading.as_str() {
          "jsonp" => {
            plugins.push(rspack_plugin_runtime::JsonpChunkLoadingPlugin {}.boxed());
            plugins.push(rspack_plugin_runtime::CssModulesPlugin {}.boxed());
          }
          "require" => {
            plugins.push(rspack_plugin_runtime::StartupChunkDependenciesPlugin::new(false).boxed());
            plugins.push(rspack_plugin_runtime::CommonJsChunkLoadingPlugin {}.boxed());
          }
          // TODO async-node
          "import" => {
            plugins.push(rspack_plugin_runtime::ModuleChunkLoadingPlugin {}.boxed());
          }
          "import-scripts" => {
            plugins.push(rspack_plugin_runtime::StartupChunkDependenciesPlugin::new(true).boxed());
            plugins.push(rspack_plugin_runtime::ImportScriptsChunkLoadingPlugin {}.boxed());
          }
          "universal" => {
            return Err(internal_error!(
              "Universal Chunk Loading is not implemented yet.",
            ))
          }
          _ => {
            return Err(internal_error!(
              "Unsupported chunk loading type ${chunk_loading}.",
            ))
          }
        }
      }
    }
    Ok(())
  }

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
            plugins.push(rspack_plugin_library::ExportPropertyLibraryPlugin::default().boxed());
            plugins.push(rspack_plugin_library::UmdLibraryPlugin::new("umd2".eq(library)).boxed());
          }
          "amd" | "amd-require" => {
            plugins.push(rspack_plugin_library::ExportPropertyLibraryPlugin::default().boxed());
            plugins.push(
              rspack_plugin_library::AmdLibraryPlugin::new("amd-require".eq(library)).boxed(),
            );
          }
          "module" => {
            plugins.push(rspack_plugin_library::ExportPropertyLibraryPlugin::default().boxed());
            plugins.push(rspack_plugin_library::ModuleLibraryPlugin::default().boxed());
          }
          "system" => plugins.push(rspack_plugin_library::SystemLibraryPlugin::default().boxed()),
          _ => {}
        }
      }
    }
  }
}
