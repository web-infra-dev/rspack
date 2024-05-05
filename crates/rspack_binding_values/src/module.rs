use napi_derive::napi;
use rspack_core::{
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, ExportsArgument, Module, ModuleArgument,
};
use rspack_napi::napi::bindgen_prelude::*;

use super::{JsCompatSource, ToJsCompatSource};
use crate::{JsChunk, JsCodegenerationResults};

#[napi(object)]
pub struct JsBuildMeta {
  pub module_argument: String,
  pub exports_argument: String,
  #[napi(ts_type = "'namespace' | 'dynamic' | 'default' | 'flagged'")]
  pub exports_type: String,
  #[napi(ts_type = "false | 'redirect' | 'redirect-warn'")]
  pub default_object: String,
  pub strict_harmony_module: bool,
  pub side_effect_free: Option<bool>,
  pub has_top_level_await: bool,
  pub esm: bool,
}
impl From<BuildMeta> for JsBuildMeta {
  fn from(build_meta: BuildMeta) -> Self {
    Self {
      module_argument: match build_meta.module_argument {
        ModuleArgument::Module => String::from("module"),
        ModuleArgument::WebpackModule => String::from("webpack-module"),
      },
      exports_argument: match build_meta.exports_argument {
        ExportsArgument::Exports => String::from("exports"),
        ExportsArgument::WebpackExports => String::from("webpack-exports"),
      },
      exports_type: match build_meta.exports_type {
        BuildMetaExportsType::Namespace => String::from("namespace"),
        BuildMetaExportsType::Dynamic => String::from("dynamic"),
        BuildMetaExportsType::Default => String::from("default"),
        BuildMetaExportsType::Unset => String::from("unset"),
        BuildMetaExportsType::Flagged => String::from("flagged"),
      },
      default_object: match build_meta.default_object {
        BuildMetaDefaultObject::False => String::from("false"),
        BuildMetaDefaultObject::Redirect => String::from("redirect"),
        BuildMetaDefaultObject::RedirectWarn => String::from("redirect-warn"),
      },
      strict_harmony_module: build_meta.strict_harmony_module,
      side_effect_free: build_meta.side_effect_free,
      has_top_level_await: build_meta.has_top_level_await,
      esm: build_meta.esm,
    }
  }
}

#[derive(Default)]
#[napi(object)]
pub struct JsModule {
  pub context: Option<String>,
  pub original_source: Option<JsCompatSource>,
  pub resource: Option<String>,
  pub module_identifier: String,
  pub name_for_condition: Option<String>,
  pub raw_request: Option<String>,
  pub build_meta: Option<JsBuildMeta>,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = || {
      self
        .original_source()
        .and_then(|source| source.to_js_compat_source().ok())
    };
    let name_for_condition = || self.name_for_condition().map(|s| s.to_string());
    let module_identifier = || self.identifier().to_string();
    let context = || self.get_context().map(|c| c.to_string());

    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        context: context(),
        original_source: original_source(),
        resource: Some(
          normal_module
            .resource_resolved_data()
            .resource_path
            .to_string_lossy()
            .to_string(),
        ),
        module_identifier: module_identifier(),
        name_for_condition: name_for_condition(),
        raw_request: Some(normal_module.raw_request().to_string()),
        build_meta: match normal_module.build_meta() {
          Some(build_meta) => Some(JsBuildMeta::from(build_meta.clone())),
          _ => None,
        },
      })
      .or_else(|_| {
        self.try_as_raw_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          build_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_context_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          build_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_external_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          build_meta: None,
        })
      })
      .or_else(|_| {
        Ok(JsModule {
          context: context(),
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          ..Default::default()
        })
      })
  }
}

#[napi(object)]
pub struct JsExecuteModuleArg {
  pub entry: String,
  pub runtime_modules: Vec<String>,
  pub codegen_results: JsCodegenerationResults,
  pub id: u32,
}

#[derive(Default)]
#[napi(object)]
pub struct JsRuntimeModule {
  pub source: Option<JsCompatSource>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
}

#[napi(object)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  pub chunk: JsChunk,
}
