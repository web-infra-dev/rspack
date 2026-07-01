use napi::{
  Either,
  bindgen_prelude::{Object, ToNapiValue},
};
use rspack_core::{ResourceData, ResourceParsedData, parse_resource};
use rspack_error::Diagnosable;

use crate::{
  diagnostic, error::RspackError, impl_module_methods, module::Module, plugins::JsLoaderItem,
  resource_data::ReadonlyResourceDataWrapper,
};

const NORMAL_MODULE_OWN_PROPERTIES: &[&str] = &[
  "resource",
  "request",
  "userRequest",
  "rawRequest",
  "resourceResolveData",
  "loaders",
  "matchResource",
  "error",
];

#[napi]
#[repr(C)]
pub struct NormalModule {
  pub(crate) module: Module,
}

impl NormalModule {
  pub fn new(module: Module) -> Self {
    Self { module }
  }

  pub(crate) fn into_module_instance(
    self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'_, Self>> {
    Self::new_inherited(self, env, NORMAL_MODULE_OWN_PROPERTIES)
  }

  fn as_ref(&mut self) -> napi::Result<(&rspack_core::Compilation, &rspack_core::NormalModule)> {
    let (compilation, module) = self.module.as_ref()?;
    match module.as_normal_module() {
      Some(normal_module) => Ok((compilation, normal_module)),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a NormalModule",
      )),
    }
  }

  fn as_mut(&mut self) -> napi::Result<&mut rspack_core::NormalModule> {
    let module = self.module.as_mut()?;
    match module.as_normal_module_mut() {
      Some(normal_module) => Ok(normal_module),
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Module is not a NormalModule",
      )),
    }
  }
}

#[napi]
impl NormalModule {
  #[napi(skip_typescript, getter)]
  pub fn resource(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(module.resource_resolved_data().resource().to_string())
  }

  #[napi(skip_typescript, getter)]
  pub fn request(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(module.request().to_string())
  }

  #[napi(skip_typescript, getter, js_name = "userRequest")]
  pub fn user_request(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(module.user_request().to_string())
  }

  #[napi(skip_typescript, getter, js_name = "rawRequest")]
  pub fn raw_request(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(module.raw_request().to_string())
  }

  #[napi(skip_typescript, getter, js_name = "resourceResolveData")]
  pub fn resource_resolve_data<'a>(&mut self, env: &'a napi::Env) -> napi::Result<Object<'a>> {
    let (_, module) = self.as_ref()?;
    let resource_resolved_data = module.resource_resolved_data().clone();
    let napi_value = unsafe {
      ToNapiValue::to_napi_value(
        env.raw(),
        ReadonlyResourceDataWrapper::from(resource_resolved_data),
      )?
    };
    Ok(Object::from_raw(env.raw(), napi_value))
  }

  #[napi(skip_typescript, getter)]
  pub fn loaders(&mut self) -> napi::Result<Vec<JsLoaderItem>> {
    let (_, module) = self.as_ref()?;
    Ok(module.loaders().iter().map(JsLoaderItem::from).collect())
  }

  #[napi(skip_typescript, getter, js_name = "matchResource")]
  pub fn match_resource(&mut self) -> napi::Result<Either<String, ()>> {
    let (_, module) = self.as_ref()?;
    Ok(match module.match_resource() {
      Some(match_resource) => Either::A(match_resource.resource().to_string()),
      None => Either::B(()),
    })
  }

  #[napi(skip_typescript, setter)]
  pub fn set_match_resource(&mut self, val: Either<String, ()>) -> napi::Result<()> {
    match val {
      Either::A(val) => {
        let module = self.as_mut()?;
        let ResourceParsedData {
          path,
          query,
          fragment,
        } = parse_resource(&val).expect("Should parse resource");
        *module.match_resource_mut() =
          Some(ResourceData::new_with_path(val, path, query, fragment));
      }
      Either::B(_) => {}
    }
    Ok(())
  }

  #[napi(skip_typescript, getter)]
  pub fn error(&mut self) -> napi::Result<Either<RspackError, ()>> {
    let (compilation, module) = self.as_ref()?;
    Ok(match module.first_error() {
      Some(diagnostic) => Either::A(RspackError::try_from_diagnostic(
        compilation,
        diagnostic.as_ref(),
      )?),
      None => Either::B(()),
    })
  }
}

impl_module_methods!(NormalModule);
