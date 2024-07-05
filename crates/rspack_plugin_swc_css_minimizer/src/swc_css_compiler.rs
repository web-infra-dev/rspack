use std::sync::Arc;

use rspack_core::rspack_sources::{self, SourceExt};
use rspack_error::{error, Result};
use swc_core::common::{input::SourceFileInput, source_map::SourceMapGenConfig, FileName};
use swc_core::common::{Globals, GLOBALS};
use swc_core::css::codegen::{
  writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_core::css::minifier;
use swc_core::css::parser::{lexer::Lexer, parser::ParserConfig};
use swc_core::css::{ast::Stylesheet, parser::parser::Parser};

use crate::SwcCssMinimizerRspackPluginOptions;

#[derive(Default)]
pub struct SwcCssCompiler {
  cm: Arc<swc_core::common::SourceMap>,
}

impl SwcCssCompiler {
  pub fn parse_file(&self, path: &str, source: String, config: ParserConfig) -> Result<Stylesheet> {
    let fm = self
      .cm
      .new_source_file(FileName::Custom(path.to_string()), source);

    let lexer = Lexer::new(SourceFileInput::from(&*fm), None, config);
    let mut parser = Parser::new(lexer, config);
    let stylesheet = parser.parse_all();
    stylesheet.map_err(|e| error!("Css parsing failed {}", e.message()))
  }

  fn codegen_impl(
    &self,
    ast: &Stylesheet,
    gen_source_map: SwcCssSourceMapGenConfig,
    minify: bool,
  ) -> Result<(String, Option<Vec<u8>>)> {
    let mut output = String::new();
    let mut src_map_buf: Option<Vec<_>> = gen_source_map.enable.then(Vec::new);
    let wr = BasicCssWriter::new(
      &mut output,
      src_map_buf.as_mut(),
      BasicCssWriterConfig::default(),
    );

    let mut gen = CodeGenerator::new(wr, CodegenConfig { minify });
    gen.emit(ast).map_err(|e| error!(e.to_string()))?;

    if let Some(src_map_buf) = &mut src_map_buf {
      let map = self
        .cm
        .build_source_map_with_config(src_map_buf, None, gen_source_map);
      let mut raw_map = Vec::new();
      map
        .to_writer(&mut raw_map)
        .map_err(|e| error!(e.to_string()))?;
      Ok((output, Some(raw_map)))
    } else {
      Ok((output, None))
    }
  }

  pub fn minify(
    &self,
    filename: &str,
    input_source: String,
    input_source_map: Option<rspack_sources::SourceMap>,
    gen_source_map: SwcCssSourceMapGenConfig,
  ) -> Result<rspack_sources::BoxSource> {
    let mut ast = self.parse_file(filename, input_source.clone(), Default::default())?;
    // ignore errors since css in webpack is tolerant, and diagnostics already reported in parse.
    GLOBALS.set(&Globals::default(), || {
      minifier::minify(&mut ast, minifier::options::MinifyOptions::default());
    });
    let (code, source_map) = self.codegen_impl(&ast, gen_source_map, true)?;
    if let Some(source_map) = source_map {
      let source = rspack_sources::SourceMapSource::new(rspack_sources::SourceMapSourceOptions {
        value: code,
        name: filename,
        source_map: rspack_sources::SourceMap::from_slice(&source_map)
          .expect("should be able to generate source-map"),
        original_source: Some(input_source),
        inner_source_map: input_source_map,
        remove_original_source: true,
      })
      .boxed();
      Ok(source)
    } else {
      Ok(rspack_sources::RawSource::from(code).boxed())
    }
  }
}

pub fn match_object(obj: &SwcCssMinimizerRspackPluginOptions, str: &str) -> Result<bool> {
  if let Some(condition) = &obj.test {
    if !condition.try_match(str)? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.include {
    if !condition.try_match(str)? {
      return Ok(false);
    }
  }
  if let Some(condition) = &obj.exclude {
    if condition.try_match(str)? {
      return Ok(false);
    }
  }
  Ok(true)
}

#[derive(Debug, Clone)]
pub struct SwcCssSourceMapGenConfig {
  pub enable: bool,
  pub emit_columns: bool,
  pub inline_sources_content: bool,
}

impl SourceMapGenConfig for SwcCssSourceMapGenConfig {
  fn file_name_to_source(&self, f: &FileName) -> String {
    let f = f.to_string();
    if f.starts_with('<') && f.ends_with('>') {
      f[1..f.len() - 1].to_string()
    } else {
      f
    }
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.emit_columns
  }
}
