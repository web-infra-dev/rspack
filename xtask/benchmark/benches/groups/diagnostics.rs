use rspack_core::Compilation;

pub(crate) fn assert_no_compilation_errors(compilation: &Compilation, context: &str) {
  let errors = compilation
    .get_errors()
    .map(|diagnostic| {
      diagnostic
        .render_report(false)
        .unwrap_or_else(|render_error| {
          format!("{diagnostic:#?}\nFailed to render compilation diagnostic: {render_error}")
        })
    })
    .collect::<Vec<_>>();

  assert!(
    errors.is_empty(),
    "{context} should not produce compilation errors:\n{}",
    errors.join("\n\n")
  );
}
