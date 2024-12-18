use rspack_error::miette;

use crate::fs::{BatchStorageFSError, StorageFSError};

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
enum StorageErrorReason {
  Reason(String),
  Detail(InvalidDetail),
  Error(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug)]
pub enum StorageErrorType {
  Validate,
  Save,
  Load,
}

impl std::fmt::Display for StorageErrorType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      StorageErrorType::Validate => write!(f, "validate"),
      StorageErrorType::Save => write!(f, "save"),
      StorageErrorType::Load => write!(f, "load"),
    }
  }
}

#[derive(Debug)]
pub struct StorageError {
  r#type: Option<StorageErrorType>,
  scope: Option<&'static str>,
  inner: StorageErrorReason,
}

impl From<StorageFSError> for StorageError {
  fn from(e: StorageFSError) -> Self {
    Self {
      r#type: None,
      scope: None,
      inner: StorageErrorReason::Error(Box::new(e)),
    }
  }
}

impl From<BatchStorageFSError> for StorageError {
  fn from(e: BatchStorageFSError) -> Self {
    Self {
      r#type: None,
      scope: None,
      inner: StorageErrorReason::Error(Box::new(e)),
    }
  }
}

impl StorageError {
  pub fn from_detail(
    r#type: Option<StorageErrorType>,
    scope: Option<&'static str>,
    detail: InvalidDetail,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: StorageErrorReason::Detail(detail),
    }
  }
  pub fn from_error(
    r#type: Option<StorageErrorType>,
    scope: Option<&'static str>,
    error: Box<dyn std::error::Error + Send + Sync>,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: StorageErrorReason::Error(error),
    }
  }
  pub fn from_reason(
    r#type: Option<StorageErrorType>,
    scope: Option<&'static str>,
    reason: String,
  ) -> Self {
    Self {
      r#type,
      scope,
      inner: StorageErrorReason::Reason(reason),
    }
  }
}

impl std::fmt::Display for StorageError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(t) = &self.r#type {
      write!(f, "{} ", t)?;
    }
    if let Some(scope) = self.scope {
      write!(f, "scope `{}` ", scope)?;
    }
    write!(f, "failed due to")?;

    match &self.inner {
      StorageErrorReason::Detail(detail) => {
        write!(f, " {}", detail.reason)?;
        let mut pack_info_lines = detail
          .packs
          .iter()
          .map(|p| format!("- {}", p))
          .collect::<Vec<_>>();
        if pack_info_lines.len() > 5 {
          pack_info_lines.truncate(5);
          pack_info_lines.push("...".to_string());
        }
        if !pack_info_lines.is_empty() {
          write!(f, ":\n{}", pack_info_lines.join("\n"))?;
        }
      }
      StorageErrorReason::Error(e) => {
        write!(f, " {}", e)?;
      }
      StorageErrorReason::Reason(e) => {
        write!(f, " {}", e)?;
      }
    }
    Ok(())
  }
}

impl std::error::Error for StorageError {}

impl miette::Diagnostic for StorageError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new(format!(
      "StorageError::{}",
      self
        .r#type
        .as_ref()
        .map_or("".to_string(), |t| t.to_string())
    )))
  }
  fn severity(&self) -> Option<miette::Severity> {
    Some(miette::Severity::Warning)
  }
}

pub type Result<T> = std::result::Result<T, StorageError>;
