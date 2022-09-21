use rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};

pub const RUNTIME_PLACEHOLDER_INSTALLED_MODULES: &str = "{/* __INSTALLED_MODULES__*/}";
pub const RUNTIME_PLACEHOLDER_RSPACK_EXECUTE: &str = "/* RSPACK_EXECUTE */";

#[derive(Clone, Debug, Default)]
pub struct Runtime {
  pub sources: Vec<RawSource>,
  pub context_indent: String,
}

impl Runtime {
  pub fn generate(&self) -> BoxSource {
    let mut concat = self.sources.iter().fold(
      ConcatSource::new([RawSource::from("(function () { ")]),
      |mut concat, cur| {
        concat.add(cur.clone());
        concat
      },
    );
    concat.add(RawSource::from(" })();"));
    concat.boxed()
  }

  pub fn generate_rspack_execute(&self, namespace: &str, require_str: &str, id: &str) -> BoxSource {
    RawSource::from(format!(
      r#"{}["{}"].{}("{}");"#,
      self.context_indent, namespace, require_str, id
    ))
    .boxed()
  }

  pub fn generate_with_inline_modules(
    &mut self,
    modules_code: BoxSource,
    execute_code: BoxSource,
  ) -> BoxSource {
    let runtime_source = self.generate().source().to_string();
    let modules_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_INSTALLED_MODULES)
      .unwrap();
    let modules_code_end = modules_code_start + RUNTIME_PLACEHOLDER_INSTALLED_MODULES.len();
    let execute_code_start = runtime_source
      .find(RUNTIME_PLACEHOLDER_RSPACK_EXECUTE)
      .unwrap();
    let execute_code_end = execute_code_start + RUNTIME_PLACEHOLDER_RSPACK_EXECUTE.len();
    ConcatSource::new([
      // runtime_source is all runtime code, and it's RawSource, so use RawSource at here is fine.
      RawSource::from(&runtime_source[0..modules_code_start]).boxed(),
      RawSource::from("{\n").boxed(),
      modules_code,
      RawSource::from("}").boxed(),
      RawSource::from(&runtime_source[modules_code_end..execute_code_start]).boxed(),
      execute_code,
      RawSource::from(&runtime_source[execute_code_end..runtime_source.len()]).boxed(),
    ])
    .boxed()
  }
}
