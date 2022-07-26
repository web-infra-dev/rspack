use anyhow::Result;
use hashbrown::HashSet;
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
}

impl AssetSourceModule {
  fn new(buf: Option<Vec<u8>>) -> Self {
    Self { buf }
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
  ) -> HashSet<SourceType> {
    HashSet::from_iter([SourceType::JavaScript])
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    _compilation: &rspack_core::Compilation,
  ) -> Result<Option<ModuleRenderResult>> {
    let result = match requested_source_type {
      SourceType::JavaScript => {
        if let Some(buf) = &self.buf {
          if buf.is_empty() {
            None
          } else {
            Some(ModuleRenderResult::JavaScript(format!(
              r#"rs.define("{}", function(__rspack_require__, module, exports) {{
              "use strict";
              module.exports = {:?};
            }});
            "#,
              module.id,
              // Align to Node's `toString("utf-8")`: If encoding is 'utf8' and a byte sequence in the input is not valid UTF-8, then each invalid byte is replaced with the replacement character U+FFFD.
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
