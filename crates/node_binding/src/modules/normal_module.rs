use napi::{Either, Env, JsString};
use rspack_core::{parse_resource, ResourceData, ResourceParsedData};

use crate::{
  AssetInfo, DependencyWrapper, JsCompatSource, JsDependenciesBlockWrapper, JsFactoryMeta,
  JsLibIdentOptions, JsResourceData, Module,
};

#[napi]
pub struct NormalModule {
  pub(crate) module: Module,
}

#[napi]
impl NormalModule {
  #[napi(getter)]
  pub fn context(&mut self) -> napi::Result<Either<String, ()>> {
    self.module.context()
  }

  #[napi(js_name = "_originalSource")]
  pub fn original_source(&mut self, env: &Env) -> napi::Result<Either<JsCompatSource, ()>> {
    self.module.original_source(env)
  }

  #[napi(getter)]
  pub fn resource(&mut self) -> napi::Result<Either<&String, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(&normal_module.resource_resolved_data().resource),
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn identifier(&mut self) -> napi::Result<&str> {
    self.module.identifier()
  }

  #[napi]
  pub fn name_for_condition(&mut self) -> napi::Result<Either<String, ()>> {
    self.module.name_for_condition()
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<Either<&str, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(normal_module.request()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn user_request(&mut self) -> napi::Result<Either<&str, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(normal_module.user_request()),
      None => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_user_request(&mut self, val: Either<String, ()>) -> napi::Result<()> {
    match val {
      Either::A(val) => {
        let module: &mut dyn rspack_core::Module = self.module.as_mut()?;
        if let Some(normal_module) = module.as_normal_module_mut() {
          *normal_module.user_request_mut() = val;
        }
      }
      Either::B(_) => {}
    }
    Ok(())
  }

  #[napi(getter)]
  pub fn raw_request(&mut self) -> napi::Result<Either<&str, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(normal_module.raw_request()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn factory_meta(&mut self) -> napi::Result<Either<JsFactoryMeta, ()>> {
    self.module.factory_meta()
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    self.module.get_type()
  }

  #[napi(getter)]
  pub fn layer(&mut self) -> napi::Result<Either<&String, ()>> {
    self.module.layer()
  }

  #[napi(getter, js_name = "_blocks", ts_return_type = "JsDependenciesBlock[]")]
  pub fn blocks(&mut self) -> napi::Result<Vec<JsDependenciesBlockWrapper>> {
    self.module.blocks()
  }

  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&mut self) -> napi::Result<Vec<DependencyWrapper>> {
    self.module.dependencies()
  }

  #[napi]
  pub fn size(&mut self, ty: Option<String>) -> napi::Result<f64> {
    self.module.size(ty)
  }

  #[napi(getter)]
  pub fn use_source_map(&mut self) -> napi::Result<bool> {
    self.module.use_source_map()
  }

  #[napi]
  pub fn lib_ident(
    &mut self,
    env: Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString>> {
    self.module.lib_ident(env, options)
  }

  #[napi(getter)]
  pub fn resource_resolve_data(&mut self) -> napi::Result<Either<JsResourceData, ()>> {
    let (_, module) = self.module.as_ref()?;
    Ok(match module.as_normal_module() {
      Some(module) => Either::A(module.resource_resolved_data().into()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn match_resource(&mut self) -> napi::Result<Either<&String, ()>> {
    let (_, module) = self.module.as_ref()?;
    Ok(match module.as_normal_module() {
      Some(module) => match &module.match_resource() {
        Some(match_resource) => Either::A(&match_resource.resource),
        None => Either::B(()),
      },
      None => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_match_resource(&mut self, val: Either<String, ()>) -> napi::Result<()> {
    match val {
      Either::A(val) => {
        let module: &mut dyn rspack_core::Module = self.module.as_mut()?;
        if let Some(normal_module) = module.as_normal_module_mut() {
          let ResourceParsedData {
            path,
            query,
            fragment,
          } = parse_resource(&val).expect("Should parse resource");
          *normal_module.match_resource_mut() = Some(
            ResourceData::new(val)
              .path(path)
              .query_optional(query)
              .fragment_optional(fragment),
          );
        }
      }
      Either::B(_) => {}
    }
    Ok(())
  }

  #[napi(js_name = "_emitFile")]
  pub fn emit_file(
    &mut self,
    filename: String,
    source: JsCompatSource,
    js_asset_info: Option<AssetInfo>,
  ) -> napi::Result<()> {
    self.module.emit_file(filename, source, js_asset_info)
  }
}
