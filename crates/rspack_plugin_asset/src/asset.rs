use std::{ffi::OsStr, path::Path};

use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  AssetParserDataUrlOption, BoxModule, FilenameRenderOptions, Module, ModuleType, Parser,
  SourceType,
};
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
#[derive(Debug)]
enum DataUrlOption {
  True,
  False,
  Option(Option<AssetParserDataUrlOption>),
}

#[derive(Debug)]
pub struct AssetParser {
  data_url: DataUrlOption,
}

impl AssetParser {
  pub fn with_auto(option: Option<AssetParserDataUrlOption>) -> Self {
    Self {
      data_url: DataUrlOption::Option(option),
    }
  }

  pub fn with_inline() -> Self {
    Self {
      data_url: DataUrlOption::True,
    }
  }

  pub fn with_resource() -> Self {
    Self {
      data_url: DataUrlOption::False,
    }
  }
}

// Webpack's default parser.dataUrlCondition.maxSize
const DEFAULT_MAX_SIZE: u32 = 8096;

impl Parser for AssetParser {
  fn parse(
    &self,
    module_type: ModuleType,
    args: rspack_core::ParseModuleArgs,
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    let source = args.source;

    let size = source.size();

    let is_inline = match &self.data_url {
      DataUrlOption::True => true,
      DataUrlOption::False => false,
      DataUrlOption::Option(option) => {
        let limit_size = option
          .as_ref()
          .and_then(|x| x.max_size)
          .unwrap_or(DEFAULT_MAX_SIZE);
        size <= limit_size as usize
      }
    };
    let module: BoxModule = Box::new(AssetModule::new(module_type, is_inline, source));

    Ok(module.with_empty_diagnostic())
  }
}
static ASSET_MODULE_SOURCE_TYPE_LIST: &[SourceType; 2] =
  &[SourceType::Asset, SourceType::JavaScript];

#[derive(Debug)]
struct AssetModule {
  module_type: ModuleType,
  inline: bool, // if the module is not inlined, then it will be regarded as a resource
  source: BoxSource,
  source_type_list: &'static [SourceType; 2],
}

impl AssetModule {
  fn new(module_type: ModuleType, inline: bool, source: BoxSource) -> Self {
    Self {
      module_type,
      inline,
      source,
      source_type_list: ASSET_MODULE_SOURCE_TYPE_LIST,
    }
  }
}

impl Module for AssetModule {
  #[inline(always)]
  fn module_type(&self) -> ModuleType {
    self.module_type
  }

  #[inline(always)]
  fn source_types(&self) -> &[SourceType] {
    if self.inline {
      &self.source_type_list[1..]
    } else {
      self.source_type_list.as_ref()
    }
  }

  fn original_source(&self) -> &dyn Source {
    self.source.as_ref()
  }

  fn render(
    &self,
    requested_source_type: SourceType,
    module: &rspack_core::ModuleGraphModule,
    compilation: &rspack_core::Compilation,
  ) -> Result<Option<BoxSource>> {
    let result = match requested_source_type {
      SourceType::JavaScript => Some(
        RawSource::from(format!(
          r#"function (module, exports, __rspack_require__, __rspack_dynamic_require__) {{
  "use strict";
  module.exports = "{}";
}};
"#,
          if self.inline {
            format!(
              "data:{};base64,{}",
              mime_guess::MimeGuess::from_path(Path::new(&module.uri))
                .first()
                .ok_or_else(|| anyhow::format_err!("failed to guess mime type of {}", module.id))?,
              base64::encode(&self.source.buffer())
            )
          } else {
            let path = Path::new(&module.id);

            let file_name =
              compilation
                .options
                .output
                .asset_module_filename
                .render(FilenameRenderOptions {
                  filename: Some(
                    path
                      .file_stem()
                      .and_then(OsStr::to_str)
                      .ok_or_else(|| anyhow::anyhow!("failed"))?
                      .to_owned(),
                  ),
                  extension: Some(
                    path
                      .extension()
                      .and_then(OsStr::to_str)
                      .map(|str| format!("{}{}", ".", str))
                      .ok_or_else(|| anyhow::anyhow!("failed"))?,
                  ),
                  id: None,
                  contenthash: None,
                  chunkhash: None,
                  hash: None,
                });
            let public_path = compilation.options.output.public_path.public_path();
            format!("{}{}", public_path, file_name)
          }
        ))
        .boxed(),
      ),
      SourceType::Asset => {
        if self.inline {
          None
        } else {
          Some(RawSource::from(self.source.buffer().to_vec()).boxed())
        }
      }
      _ => None,
    };

    Ok(result)
  }
}
