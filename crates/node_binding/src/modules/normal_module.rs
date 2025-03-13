use napi::Either;
use rspack_core::{parse_resource, ResourceData, ResourceParsedData};

use crate::{impl_module_methods, plugins::JsLoaderItem, JsResourceData, Module};

#[napi]
pub struct NormalModule {
  pub(crate) module: Module,
}

#[napi]
impl NormalModule {
  #[napi(getter)]
  pub fn resource(&mut self) -> napi::Result<Either<&String, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(&normal_module.resource_resolved_data().resource),
      None => Either::B(()),
    })
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
  pub fn loaders(&mut self) -> napi::Result<Either<Vec<JsLoaderItem>, ()>> {
    let (_, module) = self.module.as_ref()?;

    Ok(match module.as_normal_module() {
      Some(normal_module) => Either::A(
        normal_module
          .loaders()
          .iter()
          .map(|i| rspack_loader_runner::LoaderItem::<rspack_core::RunnerContext>::from(i.clone()))
          .map(|i| JsLoaderItem::from(&i))
          .collect::<Vec<_>>(),
      ),
      None => Either::B(()),
    })
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
}

impl_module_methods!(NormalModule);
