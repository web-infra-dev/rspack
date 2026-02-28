use rspack_cacheable::{
  cacheable,
  with::{AsOption, AsPreset},
};
use rspack_collections::Identifier;
use rspack_location::DependencyLocation;
use rspack_paths::Utf8PathBuf;

use crate::{Result, displayer::Renderer, error::Error};

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct Diagnostic {
  pub error: Error,

  // The following fields are only used to restore Diagnostic for Rspack.
  // If the current Diagnostic originates from Rust, these fields will be None.
  pub module_identifier: Option<Identifier>,
  pub loc: Option<DependencyLocation>,
  #[cacheable(with=AsOption<AsPreset>)]
  pub file: Option<Utf8PathBuf>,
  pub chunk: Option<u32>,
}

impl std::ops::Deref for Diagnostic {
  type Target = Error;
  fn deref(&self) -> &Self::Target {
    &self.error
  }
}

impl std::ops::DerefMut for Diagnostic {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.error
  }
}

impl From<Error> for Diagnostic {
  fn from(value: Error) -> Self {
    Self {
      error: value,
      ..Default::default()
    }
  }
}

impl Diagnostic {
  pub fn warn(code: String, message: String) -> Self {
    let mut error = Error::warning(message);
    error.code = Some(code);
    Self {
      error,
      ..Default::default()
    }
  }

  pub fn error(code: String, message: String) -> Self {
    let mut error = Error::error(message);
    error.code = Some(code);
    Self {
      error,
      ..Default::default()
    }
  }

  pub fn render_report(&self, colored: bool) -> Result<String> {
    let renderer = Renderer::new(colored);
    renderer.render(self)
  }
}
