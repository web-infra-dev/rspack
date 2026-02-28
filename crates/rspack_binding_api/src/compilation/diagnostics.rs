use napi::{Either, bindgen_prelude::WeakReference};
use rspack_core::Compilation;
use rspack_error::Severity;

use crate::{compilation::JsCompilation, error::RspackError};

#[napi]
pub struct Diagnostics {
  severity: Severity,
  compiler_reference: WeakReference<JsCompilation>,
}

impl Diagnostics {
  pub fn new(severity: Severity, compiler_reference: WeakReference<JsCompilation>) -> Self {
    Self {
      severity,
      compiler_reference,
    }
  }

  fn as_ref(&self) -> napi::Result<&Compilation> {
    match self.compiler_reference.get() {
      Some(wrapped_value) => Ok(wrapped_value.as_ref()?),
      None => Err(napi::Error::from_reason(
        "Unable to access compilation.errors now. The Compilation has been garbage collected by JavaScript.",
      )),
    }
  }

  fn as_mut(&mut self) -> napi::Result<&mut Compilation> {
    match self.compiler_reference.get_mut() {
      Some(wrapped_value) => Ok(wrapped_value.as_mut()?),
      None => Err(napi::Error::from_reason(
        "Unable to access compilation.errors now. The Compilation has been garbage collected by JavaScript.",
      )),
    }
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
      .filter(|diagnostic| diagnostic.severity == self.severity)
      .count();
    Ok(len as u32)
  }

  #[napi]
  pub fn values(&self) -> napi::Result<Vec<RspackError>> {
    let compilation = self.as_ref()?;

    let diagnostics = compilation.diagnostics();
    diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity == self.severity)
      .map(|diagnostic| RspackError::try_from_diagnostic(compilation, diagnostic))
      .collect::<napi::Result<Vec<RspackError>>>()
  }

  #[napi]
  pub fn get(&self, index: f64) -> napi::Result<Either<RspackError, ()>> {
    if index < 0f64 || index.is_infinite() || index.abs() != index {
      return Ok(Either::B(()));
    }

    let compilation = self.as_ref()?;
    let diagnostics = compilation.diagnostics();
    let diagnostic = diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity == self.severity)
      .nth(index as usize);
    Ok(match diagnostic {
      Some(diagnostic) => {
        let js_rspack_error = RspackError::try_from_diagnostic(compilation, diagnostic)?;
        Either::A(js_rspack_error)
      }
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn set(&mut self, index: f64, error: RspackError) -> napi::Result<()> {
    if index < 0f64 || index.is_infinite() || index.abs() != index {
      return Ok(());
    }

    let severity = self.severity;
    let compilation = self.as_mut()?;
    let diagnostics = compilation.diagnostics_mut();
    let len = diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity == severity)
      .count();

    let index = index as usize;
    if index > len {
      return Ok(());
    }

    if index == len {
      diagnostics.push(error.into_diagnostic(severity));
      return Ok(());
    }

    let mut i = 0;
    for diagnostic in diagnostics.iter_mut() {
      if diagnostic.severity == severity {
        if i == index {
          *diagnostic = error.into_diagnostic(severity);
          break;
        }
        i += 1;
      }
    }

    Ok(())
  }

  #[napi]
  pub fn splice_with_array(
    &mut self,
    index: f64,
    delete_count: Option<f64>,
    new_items: Option<Vec<RspackError>>,
  ) -> napi::Result<Vec<RspackError>> {
    let severity = self.severity;
    let compilation = self.as_mut()?;

    let diagnostics = compilation.diagnostics_mut();

    let len = diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.severity == severity)
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

    let mut removed = Vec::with_capacity(delete_count);
    let mut new_diagnostics = Vec::with_capacity(len - delete_count + to_insert.len());
    let mut i = 0;
    for diagnostic in diagnostics.drain(..) {
      if diagnostic.severity != severity {
        new_diagnostics.push(diagnostic);
        continue;
      }

      if i >= index && delete_count > 0 {
        delete_count -= 1;
        removed.push(diagnostic);
      } else {
        new_diagnostics.push(diagnostic);
      }

      if i == index && delete_count == 0 && !to_insert.is_empty() {
        new_diagnostics.append(&mut to_insert);
      }

      i += 1;
    }

    for diagnostic in to_insert {
      new_diagnostics.push(diagnostic);
    }

    *diagnostics = new_diagnostics;

    removed
      .into_iter()
      .map(|d| RspackError::try_from_diagnostic(compilation, &d))
      .collect::<napi::Result<Vec<_>>>()
  }
}
