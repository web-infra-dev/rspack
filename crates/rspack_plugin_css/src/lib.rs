// mod js_module;
// pub use js_module::*;
pub mod module;
pub mod plugin;
pub mod visitors;

use once_cell::sync::Lazy;

use rspack_core::PATH_START_BYTE_POS_MAP;
use swc_common::{input::SourceFileInput, sync::Lrc, FileName, FilePathMapping, SourceMap};

use std::sync::Arc;

use swc_css::codegen::{
  writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_css::parser::{lexer::Lexer, parser::ParserConfig};
use swc_css::{ast::Stylesheet, parser::parser::Parser};

pub use plugin::CssPlugin;

static SWC_COMPILER: Lazy<Arc<SwcCompiler>> = Lazy::new(|| Arc::new(SwcCompiler::new()));

#[derive(Default)]
pub struct SwcCompiler {}

static CM: Lazy<Lrc<SourceMap>> = Lazy::new(|| Lrc::new(SourceMap::new(FilePathMapping::empty())));

impl SwcCompiler {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse_file(&self, path: &str, source: String) -> anyhow::Result<Stylesheet> {
    let config: ParserConfig = Default::default();
    let cm = CM.clone();
    // let (handler, errors) = self::string_errors::new_handler(cm.clone(), treat_err_as_bug);
    // let result = swc_common::GLOBALS.set(&swc_common::Globals::new(), || op(cm, handler));

    // let fm = cm.load_file(Path::new(path))?;
    let fm = cm.new_source_file(FileName::Custom(path.to_string()), source);

    PATH_START_BYTE_POS_MAP.insert(path.to_string(), fm.start_pos.0);
    println!("{:?}", fm.start_pos);
    let lexer = Lexer::new(SourceFileInput::from(&*fm), config);
    let mut parser = Parser::new(lexer, config);
    let stylesheet = parser.parse_all();
    let _errors = parser.take_errors();
    stylesheet.ok().ok_or_else(|| anyhow::format_err!("failed"))
  }

  pub fn codegen(&self, ast: &Stylesheet) -> String {
    let _config: CodegenConfig = CodegenConfig { minify: false };
    let mut output = String::new();
    let wr = BasicCssWriter::new(
      &mut output,
      None, // Some(&mut src_map_buf),
      BasicCssWriterConfig::default(),
    );

    let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });

    gen.emit(ast).unwrap();

    output
  }
}
