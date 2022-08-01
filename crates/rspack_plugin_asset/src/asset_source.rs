use anyhow::Result;
use rspack_core::{BoxModule, Module, ModuleRenderResult, ModuleType, Parser, SourceType};

#[derive(Debug, Default)]
pub struct AssetSourceParser {}

impl Parser for AssetSourceParser {
  fn parse(
    &self,
    _module_type: ModuleType,
    args: rspack_core::ParseModuleArgs,
  ) -> Result<BoxModule> {
    Ok(Box::new(AssetSourceModule::new(
      args.source.map(|content| content.into_bytes()),
    )))
  }
}

#[derive(Debug)]
struct AssetSourceModule {
  buf: Option<Vec<u8>>,
  source_type_vec: &'static [SourceType; 1],
}

impl AssetSourceModule {
  fn new(buf: Option<Vec<u8>>) -> Self {
    Self {
      buf,
      source_type_vec: &[SourceType::JavaScript],
    }
  }
}

impl Module for AssetSourceModule {
  fn module_type(&self) -> ModuleType {
    ModuleType::Asset
  }

  fn source_types(
    &self,
    _module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> &[SourceType] {
    self.source_type_vec.as_ref()
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let namespace = &compilation.options.output.namespace;
    let result = match requested_source_type {
      SourceType::JavaScript => {
        if let Some(buf) = &self.buf {
          if buf.is_empty() {
            None
          } else {
            Some(ModuleRenderResult::JavaScript(format!(
              r#"self["{}"].__rspack_register__(["{}"], {{"{}": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {{
  "use strict";
  module.exports = {:?};
}}}});
"#,
              namespace,
              module.id,
              module.id,
              // Align to Node's `Buffer.prototype.toString("utf-8")`: If encoding is 'utf8' and a byte sequence in the input is not valid UTF-8, then each invalid byte is replaced with the replacement character U+FFFD.
              String::from_utf8_lossy(buf)
            )))
          }
        } else {
          None
        }
      }
      _ => None,
    };

    Ok(result)
  }
}
