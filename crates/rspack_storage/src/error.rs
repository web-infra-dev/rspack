use crate::fs::{BatchFSError, FSError};

#[derive(Debug)]
pub struct InvalidDetail {
  pub reason: String,
  pub packs: Vec<String>,
}

#[derive(Debug)]
pub enum ValidateResult {
  NotExists,
  Valid,
  Invalid(InvalidDetail),
}

impl ValidateResult {
  pub fn invalid(reason: &str) -> Self {
    Self::Invalid(InvalidDetail {
      reason: reason.to_string(),
      packs: vec![],
    })
  }
  pub fn invalid_with_packs(reason: &str, packs: Vec<String>) -> Self {
    Self::Invalid(InvalidDetail {
      reason: reason.to_string(),
      packs,
    })
  }
  pub fn is_valid(&self) -> bool {
    matches!(self, ValidateResult::Valid)
  }
}

#[derive(Debug)]
pub(crate) enum ErrorReason {
  Reason(String),
  Detail(InvalidDetail),
  Error(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for ErrorReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorReason::Detail(detail) => {
        write!(f, "{}", detail.reason)?;
        for line in detail.packs.iter().take(5) {
          write!(f, "\n{line}")?;
        }
        if detail.packs.len() > 5 {
          write!(f, "\n...")?;
        }
      }
      ErrorReason::Error(e) => {
        if let Some(e) = e.downcast_ref::<Error>() {
          write!(f, "{}", e.inner)?;
        } else {
          write!(f, "{e}")?;
        }
      }
      ErrorReason::Reason(e) => {
        write!(f, "{e}")?;
      }
    };
    Ok(())
  }
}

#[derive(Debug)]
pub enum ErrorType {
  Validate,
  Save,
  Load,
}

impl std::fmt::Display for ErrorType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorType::Validate => write!(f, "validate"),
      ErrorType::Save => write!(f, "save"),
      ErrorType::Load => write!(f, "load"),
    }
  }
}

#[derive(Debug)]
pub struct Error {
  r#type: Option<ErrorType>,
  scope: Option<&'static str>,
  inner: ErrorReason,
}

impl From<FSError> for Error {
  fn from(e: FSError) -> Self {
    Self {
      r#type: None,
      scope: None,
      inner: ErrorReason::Error(Box::new(e)),
    }
  }
}

impl From<BatchFSError> for Error {
  fn from(e: BatchFSError) -> Self {
    Self {
      r#type: None,
      scope: None,
      inner: ErrorReason::Error(Box::new(e)),
    }
  }
}

impl Error {
  pub fn from_detail(
    r#type: Option<ErrorType>,
    scope: Option<&'static str>,
    detail: InvalidDetail,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: ErrorReason::Detail(detail),
    }
  }
  pub fn from_error(
    r#type: Option<ErrorType>,
    scope: Option<&'static str>,
    error: Box<dyn std::error::Error + Send + Sync>,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: ErrorReason::Error(error),
    }
  }
  pub fn from_reason(
    r#type: Option<ErrorType>,
    scope: Option<&'static str>,
    reason: String,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: ErrorReason::Reason(reason),
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(t) = &self.r#type {
      write!(f, "{t} ")?;
      if let Some(scope) = self.scope {
        write!(f, "scope `{scope}` ")?;
      }
      write!(f, "failed due to")?;
      write!(f, " {}", self.inner)?;
    } else {
      write!(f, "{}", self.inner)?;
    }

    Ok(())
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match &self.inner {
      ErrorReason::Error(error) => error.source(),
      _ => None,
    }
  }
}

impl From<Error> for rspack_error::Error {
  fn from(value: Error) -> rspack_error::Error {
    let mut error = rspack_error::Error::warning(value.to_string());
    error.code = Some(format!(
      "Error::{}",
      value
        .r#type
        .as_ref()
        .map_or(String::new(), |t| t.to_string())
    ));
    error
  }
}

pub type Result<T> = std::result::Result<T, Error>;
