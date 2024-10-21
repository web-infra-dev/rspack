use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetFullHashRuntimeModule {
  id: Identifier,
}

impl GetFullHashRuntimeModule {
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let generated_code = include_str!("runtime/get_full_hash.js")
      .cow_replace("$HASH$", compilation.get_hash().unwrap_or("XXXX"))
      .to_string();
    Ok(generated_code)
  }
}

impl Default for GetFullHashRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/get_full_hash"))
  }
}

impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn cacheable(&self) -> bool {
    false
  }

  fn full_hash(&self) -> bool {
    true
  }
}
