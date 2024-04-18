use rspack_core::{Compilation, Result, RuntimeGlobals, RuntimeModule};
use serde_json::json;

#[derive(Debug)]
pub struct FederationRuntimeModule {
  runtime_requirements: Set<String>,
  init_options_without_shared: NormalizedRuntimeInitOptionsWithOutShared,
}

impl FederationRuntimeModule {
  pub fn new(
    runtime_requirements: Set<String>,
    init_options_without_shared: NormalizedRuntimeInitOptionsWithOutShared,
  ) -> Self {
    Self {
      runtime_requirements,
      init_options_without_shared,
    }
  }
}

impl RuntimeModule for FederationRuntimeModule {
  fn name(&self) -> String {
    "federation runtime".to_string()
  }

  fn stage(&self) -> i32 {
    RuntimeModule::STAGE_NORMAL - 1
  }

  fn generate(&self, _compilation: &Compilation) -> Result<String> {
    let federation_global = format!(
      "{}.federation",
      RuntimeGlobals::REQUIRE.or("__webpack_require__")
    );
    let init_options_json = json!(self.init_options_without_shared).to_string();

    Ok(format!(
      r#"
      if(!{federation_global}){{
        {federation_global} = {{
          initOptions: {init_options_json},
          initialConsumes: undefined,
          bundlerRuntimeOptions: {{}}
        }};
      }}
      "#,
      federation_global = federation_global,
      init_options_json = init_options_json,
    ))
  }
}
