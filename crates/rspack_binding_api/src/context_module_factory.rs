use napi::bindgen_prelude::{
  ClassInstance, Either, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue,
};
use napi_derive::napi;
use rspack_core::{AfterResolveData, BeforeResolveData};
use rspack_regex::RspackRegex;

use crate::DependencyWrapper;

#[napi]
pub struct JsContextModuleFactoryBeforeResolveData(Box<BeforeResolveData>);

#[napi]
impl JsContextModuleFactoryBeforeResolveData {
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

  #[napi(getter, ts_return_type = "RegExp | undefined")]
  pub fn reg_exp(&self) -> Either<RspackRegex, ()> {
    match &self.0.reg_exp {
      Some(r) => Either::A(r.clone()),
      None => Either::B(()),
    }
  }

  #[napi(setter, ts_args_type = "rawRegExp: RegExp | undefined")]
  pub fn set_reg_exp(&mut self, raw_reg_exp: Either<RspackRegex, ()>) {
    self.0.reg_exp = match raw_reg_exp {
      Either::A(regex) => Some(regex),
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

pub struct JsContextModuleFactoryBeforeResolveDataWrapper(Box<BeforeResolveData>);

impl JsContextModuleFactoryBeforeResolveDataWrapper {
  pub fn new(data: Box<BeforeResolveData>) -> Self {
    Self(data)
  }

  pub fn take(self) -> Box<BeforeResolveData> {
    self.0
  }
}

impl FromNapiValue for JsContextModuleFactoryBeforeResolveDataWrapper {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let instance =
      <ClassInstance<JsContextModuleFactoryBeforeResolveData> as FromNapiValue>::from_napi_value(
        env, napi_val,
      )?;
    Ok(Self(instance.0.clone()))
  }
}

impl ToNapiValue for JsContextModuleFactoryBeforeResolveDataWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let js_val = JsContextModuleFactoryBeforeResolveData(val.0);
    ToNapiValue::to_napi_value(env, js_val)
  }
}

impl TypeName for JsContextModuleFactoryBeforeResolveDataWrapper {
  fn type_name() -> &'static str {
    "JsContextModuleFactoryBeforeResolveData"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsContextModuleFactoryBeforeResolveDataWrapper {}

pub type JsContextModuleFactoryBeforeResolveResult =
  Either<bool, JsContextModuleFactoryBeforeResolveDataWrapper>;

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

  #[napi(getter, ts_return_type = "RegExp | undefined")]
  pub fn reg_exp(&self) -> Either<RspackRegex, ()> {
    match &self.0.reg_exp {
      Some(r) => Either::A(r.clone()),
      None => Either::B(()),
    }
  }

  #[napi(setter, ts_args_type = "rawRegExp: RegExp | undefined")]
  pub fn set_reg_exp(&mut self, raw_reg_exp: Either<RspackRegex, ()>) {
    self.0.reg_exp = match raw_reg_exp {
      Either::A(regex) => Some(regex),
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

  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&self) -> Vec<DependencyWrapper> {
    self
      .0
      .dependencies
      .iter()
      .map(|dep| DependencyWrapper::new(dep.as_ref(), self.0.compilation_id, None))
      .collect::<Vec<_>>()
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
    Ok(Self(instance.0.clone()))
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
