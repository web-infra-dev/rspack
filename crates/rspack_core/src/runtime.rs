pub const RUNTIME_PLACEHOLDER_INSTALLED_MODULES: &str = "{/* __INSTALLED_MODULES__*/}";
pub const RUNTIME_PLACEHOLDER_RSPACK_EXECUTE: &str = "/* RSPACK_EXECUTE */";

#[derive(Clone, Debug)]
pub struct RuntimeSourceNode {
  pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct Runtime {
  pub sources: Vec<RuntimeSourceNode>,
  pub context_indent: String,
}

impl Runtime {
  pub fn generate(&self) -> String {
    self
      .sources
      .iter()
      .fold(String::from("(function () { "), |prev, cur| {
        prev + &cur.content
      })
      + " })();"
  }

  pub fn generate_rspack_execute(&self, namespace: &str, require_str: &str, id: &str) -> String {
    format!(
      r#"{}["{}"].{}("{}");"#,
      self.context_indent, namespace, require_str, id
    )
  }

  pub fn generate_with_inline_modules(&self, modules_code: &str, execute_code: &str) -> String {
    self
      .generate()
      .replace(
        RUNTIME_PLACEHOLDER_INSTALLED_MODULES,
        &format!("{{{}}}", modules_code),
      )
      .replace(RUNTIME_PLACEHOLDER_RSPACK_EXECUTE, execute_code)
  }
}
