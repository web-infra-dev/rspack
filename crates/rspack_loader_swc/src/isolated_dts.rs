use rspack_core::{BuildInfo, IsolatedDts};
use rspack_error::{Error, Result};

use crate::SWC_LOADER_IDENTIFIER;

pub(crate) fn set_build_info(build_info: &mut BuildInfo, resource_path: String, code: String) {
  build_info.isolated_dts = Some(IsolatedDts {
    resource_path,
    code,
  });
}

pub(crate) fn handle_isolated_dts_diagnostics(diagnostics: Vec<String>) -> Result<()> {
  let mut diagnostics = diagnostics.into_iter();
  let Some(first) = diagnostics.next() else {
    return Ok(());
  };

  let mut error = Error::error("Failed to generate declaration files.".to_string());
  error.code = Some(SWC_LOADER_IDENTIFIER.to_string());
  error.source_error = Some(Box::new(create_isolated_dts_error(first)));
  let remaining = diagnostics.collect::<Vec<_>>();
  if !remaining.is_empty() {
    error.help = Some(remaining.join("\n"));
  }

  Err(error)
}

fn create_isolated_dts_error(diagnostic: String) -> Error {
  let mut error = Error::error(diagnostic);
  error.code = Some(SWC_LOADER_IDENTIFIER.to_string());
  error
}
