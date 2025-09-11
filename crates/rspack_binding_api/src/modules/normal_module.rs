use napi::{
  CallContext, Either, JsObject, NapiRaw,
  bindgen_prelude::{FromNapiMutRef, Object, ToNapiValue},
};
use rspack_core::{ResourceData, ResourceParsedData, parse_resource};

use crate::{
  impl_module_methods,
  module::{MODULE_PROPERTIES_BUFFER, Module},
  plugins::JsLoaderItem,
  resource_data::ReadonlyResourceDataWrapper,
};

#[napi]
#[repr(C)]
pub struct NormalModule {
  pub(crate) module: Module,
}

impl NormalModule {
  pub fn new(module: Module) -> Self {
    Self { module }
  }

  pub(crate) fn custom_into_instance(
    mut self,
    env: &napi::Env,
  ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'_, Self>> {
    let (_, module) = self.as_ref()?;

    let resource_resolved_data = module.resource_resolved_data();
    let resource = env.create_string(resource_resolved_data.resource())?;
    let request = env.create_string(module.request())?;
    let user_request = env.create_string(module.user_request())?;
    let raw_request = env.create_string(module.raw_request())?;
    let resource_resolve_data = Object::from_raw(env.raw(), unsafe {
      ToNapiValue::to_napi_value(
        env.raw(),
        ReadonlyResourceDataWrapper::from(resource_resolved_data.clone()),
      )?
    });
    let loaders = Object::from_raw(env.raw(), unsafe {
      ToNapiValue::to_napi_value(
        env.raw(),
        module
          .loaders()
          .iter()
          .map(JsLoaderItem::from)
          .collect::<Vec<_>>(),
      )?
    });

    #[js_function]
    pub fn match_resource_getter(ctx: CallContext<'_>) -> napi::Result<Either<&str, ()>> {
      let this = ctx.this_unchecked::<JsObject>();
      let env = ctx.env.raw();
      let wrapped_value = unsafe { NormalModule::from_napi_mut_ref(env, this.raw())? };

      let (_, module) = wrapped_value.as_ref()?;
      Ok(match module.match_resource() {
        Some(match_resource) => Either::A(match_resource.resource()),
        None => Either::B(()),
      })
    }

    #[js_function(1)]
    pub fn match_resource_setter(ctx: CallContext) -> napi::Result<()> {
      let this = ctx.this_unchecked::<JsObject>();
      let env = ctx.env.raw();
      let wrapped_value = unsafe { NormalModule::from_napi_mut_ref(env, this.raw())? };

      let val = ctx.get::<Either<String, ()>>(0)?;
      match val {
        Either::A(val) => {
          let module = wrapped_value.as_mut()?;
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

    MODULE_PROPERTIES_BUFFER.with(|ref_cell| {
      let mut properties = ref_cell.borrow_mut();
      properties.clear();

      properties.push(
        napi::Property::new()
          .with_utf8_name("resource")?
          .with_value(&resource),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("request")?
          .with_value(&request),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("userRequest")?
          .with_value(&user_request),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("rawRequest")?
          .with_value(&raw_request),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("resourceResolveData")?
          .with_value(&resource_resolve_data),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("loaders")?
          .with_value(&loaders),
      );
      properties.push(
        napi::Property::new()
          .with_utf8_name("matchResource")?
          .with_getter(match_resource_getter)
          .with_setter(match_resource_setter),
      );
      Self::new_inherited(self, env, &mut properties)
    })
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

impl_module_methods!(NormalModule);
