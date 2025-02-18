use napi::{
  bindgen_prelude::{
    ClassInstance, Either, FromNapiMutRef, FromNapiValue, Object, ToNapiValue, TypeName,
    ValidateNapiValue,
  },
  Env,
};
use napi_derive::napi;
use rspack_core::{AfterResolveData, BeforeResolveData};
use rspack_regex::RspackRegex;
use swc_core::common::util::take::Take;

use crate::{JsDependency, JsDependencyWrapper};

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
pub struct JsContextModuleFactoryAfterResolveData {
  data: Option<Box<AfterResolveData>>,
  js_dependencies: Option<Vec<ClassInstance<'static, JsDependency>>>,
}

impl JsContextModuleFactoryAfterResolveData {
  fn as_ref(&self) -> napi::Result<&AfterResolveData> {
    match &self.data {
        Some(data) => {
          Ok(data.as_ref())
        },
        None => Err(napi::Error::from_reason(
          "Unable to access context module factory after resolve data now. The data has been taken on the Rust side."
        )),
    }
  }

  fn as_mut(&mut self) -> napi::Result<&mut AfterResolveData> {
    match &mut self.data {
        Some(data) => {
         Ok(data.as_mut())
        },
        None =>   Err(napi::Error::from_reason(
          "Unable to modify context module factory after resolve data now. The data has been taken on the Rust side."
        )),
    }
  }
}

#[napi]
impl JsContextModuleFactoryAfterResolveData {
  #[napi(getter)]
  pub fn resource(&self) -> napi::Result<&str> {
    let data = self.as_ref()?;
    Ok(data.resource.as_str())
  }

  #[napi(setter)]
  pub fn set_resource(&mut self, resource: String) -> napi::Result<()> {
    let data = self.as_mut()?;
    data.resource = resource.into();
    Ok(())
  }

  #[napi(getter)]
  pub fn context(&self) -> napi::Result<&str> {
    let data = self.as_ref()?;
    Ok(&data.context)
  }

  #[napi(setter)]
  pub fn set_context(&mut self, context: String) -> napi::Result<()> {
    let data = self.as_mut()?;
    data.context = context;
    Ok(())
  }

  #[napi(getter)]
  pub fn request(&self) -> napi::Result<&str> {
    let data = self.as_ref()?;
    Ok(&data.request)
  }

  #[napi(setter)]
  pub fn set_request(&mut self, request: String) -> napi::Result<()> {
    let data = self.as_mut()?;
    data.request = request;
    Ok(())
  }

  #[napi(getter, ts_return_type = "RegExp | undefined")]
  pub fn reg_exp(&self) -> napi::Result<Either<RspackRegex, ()>> {
    let data = self.as_ref()?;
    Ok(match &data.reg_exp {
      Some(r) => Either::A(r.clone()),
      None => Either::B(()),
    })
  }

  #[napi(setter, ts_args_type = "rawRegExp: RegExp | undefined")]
  pub fn set_reg_exp(&mut self, raw_reg_exp: Either<RspackRegex, ()>) -> napi::Result<()> {
    let data = self.as_mut()?;
    data.reg_exp = match raw_reg_exp {
      Either::A(regex) => Some(regex),
      Either::B(_) => None,
    };
    Ok(())
  }

  #[napi(getter)]
  pub fn recursive(&self) -> napi::Result<bool> {
    let data = self.as_ref()?;
    Ok(data.recursive)
  }

  #[napi(setter)]
  pub fn set_recursive(&mut self, recursive: bool) -> napi::Result<()> {
    let data = self.as_mut()?;
    data.recursive = recursive;
    Ok(())
  }

  #[napi(getter, ts_return_type = "JsDependency[]")]
  pub fn dependencies(&self, env: Env) -> napi::Result<Vec<Object>> {
    match &self.js_dependencies {
      Some(js_dependencies) => {
        Ok(js_dependencies.iter().map(|dep| dep.as_object(&env)).collect())
      },
      None => Err(napi::Error::from_reason(
        "Unable to access dependencies in context module factory after resolve data now. The data has been taken on the Rust side."
      )),
    }
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
      <JsContextModuleFactoryAfterResolveData as FromNapiMutRef>::from_napi_mut_ref(env, napi_val)?;
    let Some(mut data) = instance.data.take() else {
      return Err(napi::Error::from_reason(
        "Unable to take context module factory after resolve data now. The data has been taken on the Rust side."
      ));
    };
    let Some(js_dependencies) = instance.js_dependencies.take() else {
      return Err(napi::Error::from_reason(
        "Unable to take dependencies in context module factory after resolve data now. The data has been taken on the Rust side."
      ));
    };

    data.dependencies = js_dependencies
      .into_iter()
      .filter_map(|mut dep| dep.dependency.take())
      .collect::<Vec<_>>();

    Ok(Self(data))
  }
}

impl ToNapiValue for JsContextModuleFactoryAfterResolveDataWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    mut val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let dependencies = val.0.dependencies.take();
    let compiler_id = val.0.compiler_id;
    let js_data = JsContextModuleFactoryAfterResolveData {
      data: Some(val.0),
      js_dependencies: Some(
        dependencies
          .into_iter()
          .map(|dep| JsDependencyWrapper::from_owned(dep, compiler_id).into_instance(env))
          .collect::<napi::Result<Vec<_>>>()?,
      ),
    };
    ToNapiValue::to_napi_value(env, js_data)
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
