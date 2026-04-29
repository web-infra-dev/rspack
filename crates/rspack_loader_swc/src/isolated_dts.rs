use rspack_core::RunnerContext;
use rspack_error::{Diagnostic, Error, Result, Severity};
use rspack_javascript_compiler::{IsolatedDtsDiagnostic, render_pretty_span_diagnostic};
use rspack_loader_runner::LoaderContext;
use rspack_paths::Utf8Path;
use serde_json::{Map, Value};

use crate::SWC_LOADER_IDENTIFIER;

pub(crate) fn set_build_info(
  build_info: &mut rspack_core::BuildInfo,
  resource_path: String,
  code: String,
) {
  build_info.extras.insert(
    "rspack-swc-isolated-dts-emit".to_string(),
    Value::Object(Map::from_iter([
      ("resource_path".to_string(), Value::String(resource_path)),
      ("code".to_string(), Value::String(code)),
    ])),
  );
}

pub(crate) fn handle_isolated_dts_diagnostics(
  loader_context: &mut LoaderContext<RunnerContext>,
  resource_path: &Utf8Path,
  source_code: &str,
  diagnostics: Vec<IsolatedDtsDiagnostic>,
) -> Result<()> {
  if !diagnostics.is_empty() {
    return Err(create_isolated_dts_abort_error(
      diagnostics
        .into_iter()
        .map(|diagnostic| {
          create_isolated_dts_traceable_error(
            resource_path,
            source_code,
            &diagnostic,
            Severity::Error,
          )
        })
        .collect(),
    ));
  }

  for diagnostic in diagnostics {
    let error = create_isolated_dts_traceable_error(
      resource_path,
      source_code,
      &diagnostic,
      Severity::Warning,
    );
    loader_context.emit_diagnostic(Diagnostic::from(error));
  }

  Ok(())
}

fn create_isolated_dts_traceable_error(
  resource_path: &Utf8Path,
  source_code: &str,
  diagnostic: &IsolatedDtsDiagnostic,
  severity: Severity,
) -> Error {
  let mut error = match severity {
    Severity::Error => Error::error(render_pretty_span_diagnostic(
      source_code,
      resource_path.as_std_path(),
      diagnostic.start,
      diagnostic.end,
      &diagnostic.message,
      Severity::Error,
    )),
    Severity::Warning => Error::warning(render_pretty_span_diagnostic(
      source_code,
      resource_path.as_std_path(),
      diagnostic.start,
      diagnostic.end,
      &diagnostic.message,
      Severity::Warning,
    )),
  };
  error.code = Some(SWC_LOADER_IDENTIFIER.to_string());
  error
}

fn create_isolated_dts_abort_error(mut errors: Vec<Error>) -> Error {
  let mut error = Error::error("Failed to generate declaration files.".to_string());
  error.code = Some(SWC_LOADER_IDENTIFIER.to_string());

  if let Some(first_error) = errors.drain(..1).next() {
    error.source_error = Some(Box::new(first_error));
  }

  if !errors.is_empty() {
    error.help = Some(
      errors
        .into_iter()
        .map(|error| error.message.clone())
        .collect::<Vec<_>>()
        .join("\n"),
    );
  }

  error
}
