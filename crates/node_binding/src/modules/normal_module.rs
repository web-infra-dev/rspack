use napi::{
  bindgen_prelude::{FromNapiMutRef, Object, ToNapiValue},
  CallContext, Either, NapiRaw, NapiValue,
};
use rspack_core::{parse_resource, ResourceData, ResourceParsedData};

use crate::{impl_module_methods, plugins::JsLoaderItem, JsResourceData, Module};

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
    let loaders = unsafe {
      Object::from_raw_unchecked(
        env.raw(),
        ToNapiValue::to_napi_value(
          env.raw(),
          module
            .loaders()
            .iter()
            .map(JsLoaderItem::from)
            .collect::<Vec<_>>(),
        )?,
      )
    };

    #[js_function]
    pub fn match_resource_getter(ctx: CallContext) -> napi::Result<Either<&String, ()>> {
      let this = ctx.this_unchecked::<Object>();
      let env = ctx.env.raw();
      let wrapped_value = unsafe { NormalModule::from_napi_mut_ref(env, this.raw())? };

      let (_, module) = wrapped_value.as_ref()?;
      Ok(match module.match_resource() {
        Some(match_resource) => Either::A(&match_resource.resource),
        None => Either::B(()),
      })
    }

    #[js_function(1)]
    pub fn match_resource_setter(ctx: CallContext) -> napi::Result<()> {
      let this = ctx.this_unchecked::<Object>();
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

    let properties = vec![
      napi::Property::new("resource")?.with_value(&resource),
      napi::Property::new("request")?.with_value(&request),
      napi::Property::new("userRequest")?.with_value(&user_request),
      napi::Property::new("rawRequest")?.with_value(&raw_request),
      napi::Property::new("resourceResolveData")?.with_value(&resource_resolve_data),
      napi::Property::new("loaders")?.with_value(&loaders),
      napi::Property::new("matchResource")?
        .with_getter(match_resource_getter)
        .with_setter(match_resource_setter),
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

impl_module_methods!(NormalModule);
