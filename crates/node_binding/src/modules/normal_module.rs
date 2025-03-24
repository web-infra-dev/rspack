use napi::Either;
use rspack_core::{parse_resource, ResourceData, ResourceParsedData};

use crate::{impl_module_methods, plugins::JsLoaderItem, JsResourceData, Module};

#[napi]
pub struct NormalModule {
  pub(crate) module: Module,
}

impl NormalModule {
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
  #[napi(getter)]
  pub fn resource(&mut self) -> napi::Result<&String> {
    let (_, module) = self.as_ref()?;

    Ok(&module.resource_resolved_data().resource)
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.request())
  }

  #[napi(getter)]
  pub fn user_request(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.user_request())
  }

  #[napi(setter)]
  pub fn set_user_request(&mut self, val: String) -> napi::Result<()> {
    let module = self.as_mut()?;
    *module.user_request_mut() = val;
    Ok(())
  }

  #[napi(getter)]
  pub fn raw_request(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.raw_request())
  }

  #[napi(getter)]
  pub fn loaders(&mut self) -> napi::Result<Vec<JsLoaderItem>> {
    let (_, module) = self.as_ref()?;

    Ok(
      module
        .loaders()
        .iter()
        .map(|i| rspack_loader_runner::LoaderItem::<rspack_core::RunnerContext>::from(i.clone()))
        .map(|i| JsLoaderItem::from(&i))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(getter)]
  pub fn resource_resolve_data(&mut self) -> napi::Result<JsResourceData> {
    let (_, module) = self.as_ref()?;
    Ok(module.resource_resolved_data().into())
  }

  #[napi(getter)]
  pub fn match_resource(&mut self) -> napi::Result<Either<&String, ()>> {
    let (_, module) = self.as_ref()?;
    Ok(match module.match_resource() {
      Some(match_resource) => Either::A(&match_resource.resource),
      None => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_match_resource(&mut self, val: Either<String, ()>) -> napi::Result<()> {
    match val {
      Either::A(val) => {
        let module = self.as_mut()?;
        let ResourceParsedData {
          path,
          query,
          fragment,
        } = parse_resource(&val).expect("Should parse resource");
        *module.match_resource_mut() = Some(
          ResourceData::new(val)
            .path(path)
            .query_optional(query)
            .fragment_optional(fragment),
        );
      }
      Either::B(_) => {}
    }
    Ok(())
  }
}

impl_module_methods!(NormalModule);
