use std::fmt::Debug;

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};

#[napi(object)]
#[derive(Debug, Clone)]
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

impl From<&SourcePosition> for rspack_core::SourcePosition {
  fn from(value: &SourcePosition) -> Self {
    Self {
      line: value.line as usize,
      column: value.column.map_or(value.line as usize, |c| c as usize),
    }
  }
}

#[napi(object)]
#[derive(Debug, Clone)]
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

impl From<&RealDependencyLocation> for rspack_core::RealDependencyLocation {
  fn from(value: &RealDependencyLocation) -> Self {
    Self {
      start: (&value.start).into(),
      end: value.end.as_ref().map(Into::into),
    }
  }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct SyntheticDependencyLocation {
  pub name: String,
}

impl From<&rspack_core::SyntheticDependencyLocation> for SyntheticDependencyLocation {
  fn from(value: &rspack_core::SyntheticDependencyLocation) -> Self {
    Self {
      name: value.name.clone(),
    }
  }
}

impl From<rspack_core::SyntheticDependencyLocation> for SyntheticDependencyLocation {
  fn from(value: rspack_core::SyntheticDependencyLocation) -> Self {
    Self { name: value.name }
  }
}

impl From<&SyntheticDependencyLocation> for rspack_core::SyntheticDependencyLocation {
  fn from(value: &SyntheticDependencyLocation) -> Self {
    Self {
      name: value.name.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum DependencyLocation {
  Real(RealDependencyLocation),
  Synthetic(SyntheticDependencyLocation),
}

impl ToNapiValue for DependencyLocation {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
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
}

impl FromNapiValue for DependencyLocation {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
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

impl From<&DependencyLocation> for rspack_core::DependencyLocation {
  fn from(value: &DependencyLocation) -> Self {
    match value {
      DependencyLocation::Real(real_dependency_location) => {
        rspack_core::DependencyLocation::Real(real_dependency_location.into())
      }
      DependencyLocation::Synthetic(synthetic_dependency_location) => {
        rspack_core::DependencyLocation::Synthetic(synthetic_dependency_location.into())
      }
    }
  }
}
