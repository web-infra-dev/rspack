use std::fmt::Debug;

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};

#[napi(object)]
#[derive(Debug)]
pub struct SourcePosition {
  pub line: u32,
  pub column: Option<u32>,
}

impl From<&rspack_core::SourcePosition> for SourcePosition {
  fn from(value: &rspack_core::SourcePosition) -> Self {
    Self {
      line: value.line as u32,
      column: Some(value.column as u32),
    }
  }
}

impl From<rspack_core::SourcePosition> for SourcePosition {
  fn from(value: rspack_core::SourcePosition) -> Self {
    Self {
      line: value.line as u32,
      column: Some(value.column as u32),
    }
  }
}

#[napi(object)]
#[derive(Debug)]
pub struct RealDependencyLocation {
  pub start: SourcePosition,
  pub end: Option<SourcePosition>,
}

impl From<&rspack_core::RealDependencyLocation> for RealDependencyLocation {
  fn from(value: &rspack_core::RealDependencyLocation) -> Self {
    Self {
      start: value.start.into(),
      end: value.end.map(Into::into),
    }
  }
}

impl From<rspack_core::RealDependencyLocation> for RealDependencyLocation {
  fn from(value: rspack_core::RealDependencyLocation) -> Self {
    Self {
      start: value.start.into(),
      end: value.end.map(Into::into),
    }
  }
}

#[napi(object)]
#[derive(Debug)]
pub struct SyntheticDependencyLocation {
  pub name: String,
}

impl From<&rspack_core::SyntheticDependencyLocation> for SyntheticDependencyLocation {
  fn from(value: &rspack_core::SyntheticDependencyLocation) -> Self {
    Self {
      name: value.name.to_string(),
    }
  }
}

impl From<rspack_core::SyntheticDependencyLocation> for SyntheticDependencyLocation {
  fn from(value: rspack_core::SyntheticDependencyLocation) -> Self {
    Self { name: value.name }
  }
}

#[derive(Debug)]
pub enum DependencyLocation {
  Real(RealDependencyLocation),
  Synthetic(SyntheticDependencyLocation),
}

impl ToNapiValue for DependencyLocation {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    match val {
      DependencyLocation::Real(real_dependency_location) => {
        ToNapiValue::to_napi_value(env, real_dependency_location)
      }
      DependencyLocation::Synthetic(synthetic_dependency_location) => {
        ToNapiValue::to_napi_value(env, synthetic_dependency_location)
      }
    }
  }
}

impl FromNapiValue for DependencyLocation {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let obj = napi::bindgen_prelude::Object::from_napi_value(env, napi_val)?;
    if let Ok(Some(name)) = obj.get::<String>("name") {
      return Ok(DependencyLocation::Synthetic(SyntheticDependencyLocation {
        name,
      }));
    };
    let real_dependency_location: RealDependencyLocation =
      FromNapiValue::from_napi_value(env, napi_val)?;
    Ok(DependencyLocation::Real(real_dependency_location))
  }
}

impl From<&rspack_core::DependencyLocation> for DependencyLocation {
  fn from(value: &rspack_core::DependencyLocation) -> Self {
    match value {
      rspack_core::DependencyLocation::Real(real_dependency_location) => {
        DependencyLocation::Real(real_dependency_location.into())
      }
      rspack_core::DependencyLocation::Synthetic(synthetic_dependency_location) => {
        DependencyLocation::Synthetic(synthetic_dependency_location.into())
      }
    }
  }
}

impl From<rspack_core::DependencyLocation> for DependencyLocation {
  fn from(value: rspack_core::DependencyLocation) -> Self {
    match value {
      rspack_core::DependencyLocation::Real(real_dependency_location) => {
        DependencyLocation::Real(real_dependency_location.into())
      }
      rspack_core::DependencyLocation::Synthetic(synthetic_dependency_location) => {
        DependencyLocation::Synthetic(synthetic_dependency_location.into())
      }
    }
  }
}
