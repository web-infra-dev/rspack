use napi::bindgen_prelude::{
  ClassInstance, Either, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue,
};
use napi_derive::napi;
use rspack_core::AfterResolveData;

use crate::RawRegex;

#[napi(object)]
pub struct JsContextModuleFactoryBeforeResolveData {
  pub context: String,
  pub request: String,
  pub reg_exp: Option<RawRegex>,
  pub recursive: bool,
}

pub type JsContextModuleFactoryBeforeResolveResult =
  Either<bool, JsContextModuleFactoryBeforeResolveData>;

#[napi]
pub struct JsContextModuleFactoryAfterResolveData(Box<AfterResolveData>);

#[napi]
impl JsContextModuleFactoryAfterResolveData {
  #[napi(getter)]
  pub fn resource(&self) -> &str {
    self.0.resource.as_str()
  }

  #[napi(setter)]
  pub fn set_resource(&mut self, resource: String) {
    self.0.resource = resource.into();
  }

  #[napi(getter)]
  pub fn context(&self) -> &str {
    &self.0.context
  }

  #[napi(setter)]
  pub fn set_context(&mut self, context: String) {
    self.0.context = context;
  }

  #[napi(getter)]
  pub fn request(&self) -> &str {
    &self.0.request
  }

  #[napi(setter)]
  pub fn set_request(&mut self, request: String) {
    self.0.request = request;
  }

  #[napi(getter)]
  pub fn reg_exp(&self) -> Either<RawRegex, ()> {
    match &self.0.reg_exp {
      Some(r) => Either::A(r.clone().into()),
      None => Either::B(()),
    }
  }

  #[napi(setter)]
  pub fn set_reg_exp(&mut self, raw_reg_exp: Either<RawRegex, ()>) {
    self.0.reg_exp = match raw_reg_exp {
      Either::A(raw_reg_exp) => Some(raw_reg_exp.try_into().unwrap()),
      Either::B(_) => None,
    };
  }

  #[napi(getter)]
  pub fn recursive(&self) -> bool {
    self.0.recursive
  }

  #[napi(setter)]
  pub fn set_recursive(&mut self, recursive: bool) {
    self.0.recursive = recursive;
  }
}

pub struct JsContextModuleFactoryAfterResolveDataWrapper(Box<AfterResolveData>);

impl JsContextModuleFactoryAfterResolveDataWrapper {
  pub fn new(data: Box<AfterResolveData>) -> Self {
    JsContextModuleFactoryAfterResolveDataWrapper(data)
  }

  pub fn take(self) -> Box<AfterResolveData> {
    self.0
  }
}

impl FromNapiValue for JsContextModuleFactoryAfterResolveDataWrapper {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let instance =
      <ClassInstance<JsContextModuleFactoryAfterResolveData> as FromNapiValue>::from_napi_value(
        env, napi_val,
      )?;
    Ok(JsContextModuleFactoryAfterResolveDataWrapper(
      instance.0.clone(),
    ))
  }
}

impl ToNapiValue for JsContextModuleFactoryAfterResolveDataWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let js_val = JsContextModuleFactoryAfterResolveData(val.0);
    ToNapiValue::to_napi_value(env, js_val)
  }
}

impl TypeName for JsContextModuleFactoryAfterResolveDataWrapper {
  fn type_name() -> &'static str {
    "JsContextModuleFactoryAfterResolveData"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsContextModuleFactoryAfterResolveDataWrapper {}

pub type JsContextModuleFactoryAfterResolveResult =
  Either<bool, JsContextModuleFactoryAfterResolveDataWrapper>;
