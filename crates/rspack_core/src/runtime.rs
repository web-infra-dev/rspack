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

  pub fn generate_with_inline_modules(&mut self, modules_code: &str, execute_code: &str) -> String {
    self
      .generate()
      .source()
      .replace(
        RUNTIME_PLACEHOLDER_INSTALLED_MODULES,
        &format!("{{{}}}", modules_code),
      )
      .replace(RUNTIME_PLACEHOLDER_RSPACK_EXECUTE, execute_code)
  }
}
