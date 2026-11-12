use napi::{Env, JsString, Result, bindgen_prelude::*};
use rspack_core::{ResourceData, ResourceParsedData, parse_resource};
use rspack_error::Diagnosable;

use crate::{
  error::RspackError, module::ModuleHandle, plugins::JsLoaderItem,
  resource_data::ReadonlyResourceDataWrapper,
};

impl ModuleHandle {
  fn with_normal<R>(
    &self,
    f: impl FnOnce(&rspack_core::Compilation, &rspack_core::NormalModule) -> Result<R>,
  ) -> Result<R> {
    self.with_ref(|compilation, module| match module.as_normal_module() {
      Some(module) => f(compilation, module),
      None => Err(Error::from_reason("Module is not a NormalModule")),
    })
  }
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_resource<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| env.create_string(module.resource_resolved_data().resource()))
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_request<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| env.create_string(module.request()))
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_user_request<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| env.create_string(module.user_request()))
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_raw_request<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| env.create_string(module.raw_request()))
}

#[napi(
  ts_args_type = "module: Module",
  ts_return_type = "Readonly<JsResourceData>"
)]
pub fn normal_module_get_resource_resolve_data(
  module: Object<'_>,
) -> Result<ReadonlyResourceDataWrapper> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| Ok(module.resource_resolved_data().clone().into()))
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_loaders(module: Object<'_>) -> Result<Vec<JsLoaderItem>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| Ok(module.loaders().iter().map(JsLoaderItem::from).collect()))
}

#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_match_resource(module: Object<'_>) -> Result<Either<String, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|_, module| {
    Ok(match module.match_resource() {
      Some(resource) => Either::A(resource.resource().to_owned()),
      None => Either::B(()),
    })
  })
}
#[napi(ts_args_type = "module: Module, value: string | undefined")]
pub fn normal_module_set_match_resource(
  module: Object<'_>,
  value: Either<String, ()>,
) -> Result<()> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  if module.type_id != std::any::TypeId::of::<rspack_core::NormalModule>() {
    return Err(Error::from_reason("Module is not a NormalModule"));
  }
  let handle = module;
  if let Either::A(value) = value {
    let ResourceParsedData {
      path,
      query,
      fragment,
    } = parse_resource(&value).expect("Should parse resource");
    let module = handle
      .as_mut()?
      .as_normal_module_mut()
      .ok_or_else(|| Error::from_reason("Module is not a NormalModule"))?;
    *module.match_resource_mut() = Some(Box::new(ResourceData::new_with_path(
      value, path, query, fragment,
    )));
  }
  Ok(())
}
#[napi(ts_args_type = "module: Module")]
pub fn normal_module_get_error(module: Object<'_>) -> Result<Either<RspackError, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_normal(|compilation, module| {
    Ok(match module.error() {
      Some(diagnostic) => Either::A(RspackError::try_from_diagnostic(compilation, diagnostic)?),
      None => Either::B(()),
    })
  })
}
