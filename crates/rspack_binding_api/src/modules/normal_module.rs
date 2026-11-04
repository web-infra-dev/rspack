use napi::{Result, bindgen_prelude::*};
use rspack_core::{ResourceData, ResourceParsedData, parse_resource};
use rspack_error::Diagnosable;

use crate::{error::RspackError, module::ModuleHandle, module_bridge::ModuleRef};

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
pub fn normal_module_get_match_resource(module: ModuleRef<'_>) -> Result<Either<String, ()>> {
  module.with_normal(|_, module| {
    Ok(match module.match_resource() {
      Some(resource) => Either::A(resource.resource().to_owned()),
      None => Either::B(()),
    })
  })
}
#[napi(ts_args_type = "module: Module, value: string | undefined")]
pub fn normal_module_set_match_resource(
  module: ModuleRef<'_>,
  value: Either<String, ()>,
) -> Result<()> {
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
pub fn normal_module_get_error(module: ModuleRef<'_>) -> Result<Either<RspackError, ()>> {
  module.with_normal(|compilation, module| {
    Ok(match module.first_error() {
      Some(diagnostic) => Either::A(RspackError::try_from_diagnostic(
        compilation,
        diagnostic.as_ref(),
      )?),
      None => Either::B(()),
    })
  })
}
