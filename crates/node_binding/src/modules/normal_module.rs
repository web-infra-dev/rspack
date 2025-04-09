use napi::{
  bindgen_prelude::{Object, ToNapiValue},
  Either, NapiValue,
};
use rspack_core::{parse_resource, ResourceData, ResourceParsedData};

use crate::{impl_module_methods, plugins::JsLoaderItem, JsResourceData, Module};

#[napi]
pub struct NormalModule {
  pub(crate) module: Module,
}

impl NormalModule {
  pub(crate) fn custom_into_instance(
    mut self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<Self>> {
    let (_, module) = self.as_ref()?;

    let resource_resolved_data = module.resource_resolved_data();
    let resource = env.create_string(&resource_resolved_data.resource)?;
    let request = env.create_string(module.request())?;
    let user_request = env.create_string(module.user_request())?;
    let raw_request = env.create_string(module.raw_request())?;
    let resource_resolve_data = unsafe {
      Object::from_raw_unchecked(
        env.raw(),
        ToNapiValue::to_napi_value(
          env.raw(),
          JsResourceData::from(resource_resolved_data.clone()),
        )?,
      )
    };

    let properties = vec![
      napi::Property::new("resource")?.with_value(&resource),
      napi::Property::new("request")?.with_value(&request),
      napi::Property::new("userRequest")?.with_value(&user_request),
      napi::Property::new("rawRequest")?.with_value(&raw_request),
      napi::Property::new("resourceResolveData")?.with_value(&resource_resolve_data),
    ];
    Self::new_inherited(self, env, properties)
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
