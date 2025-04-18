use napi::{
  bindgen_prelude::{FromNapiValue, Object, Reference, ToNapiValue, WeakReference},
  Env, NapiValue,
};
use rspack_core::Compilation;
use rspack_error::RspackSeverity;

use crate::{JsCompilation, JsRspackError};

#[napi]
pub struct Diagnostics {
  severity: RspackSeverity,
  compiler_reference: WeakReference<JsCompilation>,
}

impl Diagnostics {
  pub fn new(severity: RspackSeverity, compiler_reference: WeakReference<JsCompilation>) -> Self {
    Self {
      severity,
      compiler_reference,
    }
  }

  fn as_ref(&self) -> napi::Result<&Compilation> {
    match self.compiler_reference.get() {
      Some(wrapped_value) => Ok(wrapped_value.as_ref()?),
      None => Err(napi::Error::from_reason(
        "Unable to access compilation.errors now. The Compilation has been garbage collected by JavaScript."
      )),
    }
  }

  fn as_mut(&mut self) -> napi::Result<&mut Compilation> {
    match self.compiler_reference.get_mut() {
      Some(wrapped_value) => {
        Ok(wrapped_value.as_mut()?)
      },
      None => Err(napi::Error::from_reason(
        "Unable to access compilation.errors now. The Compilation has been garbage collected by JavaScript."
      )),
    }
  }

  pub fn to_reference(self, env: &Env) -> napi::Result<Reference<Self>> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    unsafe { Reference::<Self>::from_napi_value(raw_env, napi_val) }
  }

  pub fn to_jsobject(self, env: &Env) -> napi::Result<Object> {
    let raw_env = env.raw();
    let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, self)? };
    Ok(unsafe { Object::from_raw_unchecked(raw_env, napi_val) })
  }
}

#[napi]
impl Diagnostics {
  #[napi(getter)]
  pub fn length(&self) -> napi::Result<u32> {
    let compilation = self.as_ref()?;

    let diagnostics = compilation.diagnostics();
    let len = diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity() == self.severity)
      .count();
    Ok(len as u32)
  }

  #[napi]
  pub fn splice_with_array(
    &mut self,
    index: f64,
    delete_count: Option<f64>,
    new_items: Option<Vec<JsRspackError>>,
  ) -> napi::Result<Vec<JsRspackError>> {
    let severity = self.severity;
    let compilation = self.as_mut()?;
    let colors = compilation.options.stats.colors;

    let diagnostics = compilation.diagnostics_mut();

    let len = diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity() == severity)
      .count();
    let len_f64 = len as f64;

    let index = if index < 0f64 {
      (len_f64 + index).max(0f64)
    } else {
      index.min(len_f64)
    };

    let mut delete_count = match delete_count {
      Some(dc) => dc.min(len_f64 - index),
      None => len_f64 - index,
    } as usize;
    let index = index as usize;

    let mut to_insert = match new_items {
      Some(items) => items
        .into_iter()
        .map(|error| error.into_diagnostic(severity))
        .collect::<Vec<_>>(),
      None => vec![],
    };

    let mut result = Vec::with_capacity(delete_count);
    let mut new_diagnostics = Vec::with_capacity(len - delete_count + to_insert.len());
    let mut i = 0;
    for diagnostic in diagnostics.drain(..) {
      if diagnostic.severity() != severity {
        new_diagnostics.push(diagnostic);
        continue;
      }

      if i >= index && delete_count > 0 {
        delete_count -= 1;
        result.push(diagnostic);
      } else {
        new_diagnostics.push(diagnostic);
        if delete_count == 0 && !to_insert.is_empty() {
          for diagnostic in to_insert.drain(..) {
            new_diagnostics.push(diagnostic);
          }
        }
      }

      i += 1;
    }

    for diagnostic in to_insert.drain(..) {
      new_diagnostics.push(diagnostic);
    }

    *diagnostics = new_diagnostics;

    Ok(
      result
        .into_iter()
        .map(|d| JsRspackError::try_from_diagnostic(&d, colors))
        .collect::<napi::Result<Vec<_>>>()?,
    )
  }

  #[napi]
  pub fn push(&mut self, new_item: JsRspackError) -> napi::Result<()> {
    let severity = self.severity;
    let compilation = self.as_mut()?;

    let diagnostics = compilation.diagnostics_mut();
    diagnostics.push(new_item.into_diagnostic(severity));

    Ok(())
  }
}
