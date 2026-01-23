use napi_derive::napi;
use rspack_core::Compilation;

#[napi]
pub struct JsDependencies {
  pub(crate) compilation: &'static Compilation,
}

impl JsDependencies {
  pub(crate) fn new(compilation: &'static Compilation) -> Self {
    Self { compilation }
  }
}

#[napi]
impl JsDependencies {
  #[napi(getter)]
  pub fn file_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .file_dependencies()
      .0
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn added_file_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .file_dependencies()
      .1
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn removed_file_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .file_dependencies()
      .3
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(getter)]
  pub fn context_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .context_dependencies()
      .0
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn added_context_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .context_dependencies()
      .1
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn removed_context_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .context_dependencies()
      .3
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(getter)]
  pub fn missing_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .missing_dependencies()
      .0
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn added_missing_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .missing_dependencies()
      .1
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn removed_missing_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .missing_dependencies()
      .3
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }

  #[napi(getter)]
  pub fn build_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .build_dependencies()
      .0
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn added_build_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .build_dependencies()
      .1
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
  #[napi(getter)]
  pub fn removed_build_dependencies(&self) -> Vec<String> {
    self
      .compilation
      .build_dependencies()
      .3
      .map(|i| i.to_string_lossy().to_string())
      .collect()
  }
}
