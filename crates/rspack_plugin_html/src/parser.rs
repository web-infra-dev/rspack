use std::sync::Arc;

use cow_utils::CowUtils;
use rspack_core::Compilation;
use rspack_error::{
  IntoTWithDiagnosticArray, Result, TWithDiagnosticArray, ToStringResultToRspackResultExt,
};
use rspack_util::SpanExt;
use swc_core::common::{
  DUMMY_SP, FileName, FilePathMapping, GLOBALS, SourceFile, SourceMap, sync::Lrc,
};
use swc_html::{
  ast::{Document, DocumentFragment, DocumentMode, Element, Namespace},
  codegen::{
    CodeGenerator, CodegenConfig, Emit,
    writer::basic::{BasicHtmlWriter, BasicHtmlWriterConfig},
  },
  parser::{
    error::Error, parse_file_as_document, parse_file_as_document_fragment, parser::ParserConfig,
  },
  visit::{VisitMut, VisitMutWith},
};
use swc_html_minifier::{
  MinifyCss, minify_document_fragment_with_custom_css_minifier,
  minify_document_with_custom_css_minifier, option::MinifyOptions,
};

use crate::config::HtmlRspackPluginOptions;

#[derive(Debug)]
pub enum CompiledDocument {
  Document(Document),
  DocumentWithoutDoctype(Document),
  DocumentFragment(DocumentFragment),
}

impl CompiledDocument {
  pub fn visit_mut_with<V: VisitMut>(&mut self, visitor: &mut V) {
    match self {
      CompiledDocument::Document(ast) => ast.visit_mut_with(visitor),
      CompiledDocument::DocumentWithoutDoctype(ast) => ast.visit_mut_with(visitor),
      CompiledDocument::DocumentFragment(ast) => ast.visit_mut_with(visitor),
    }
  }

  pub fn emit_to_codegen(
    &self,
    codegen: &mut CodeGenerator<'_, BasicHtmlWriter<'_, &mut String>>,
  ) -> Result<()> {
    match self {
      CompiledDocument::Document(ast) => codegen.emit(ast).to_rspack_result(),
      CompiledDocument::DocumentWithoutDoctype(ast) => codegen.emit(ast).to_rspack_result(),
      CompiledDocument::DocumentFragment(ast) => codegen.emit(ast).to_rspack_result(),
    }
  }
}

pub struct HtmlCompiler<'a> {
  config: &'a HtmlRspackPluginOptions,
}

impl<'a> HtmlCompiler<'a> {
  pub fn new(config: &'a HtmlRspackPluginOptions) -> Self {
    Self { config }
  }

  pub fn parse_file(
    &self,
    path: &str,
    source: String,
  ) -> Result<TWithDiagnosticArray<CompiledDocument>> {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let doc = source.trim_start().cow_to_ascii_lowercase();
    let has_doctype = doc.starts_with("<!doctype");
    let is_document = has_doctype
      || source.is_empty()
      || doc.starts_with("<html")
      || (doc.contains("<body") && doc.contains("</body>"))
      || (doc.contains("<head") && doc.contains("</head>"));
    let fm = cm.new_source_file(
      Arc::new(FileName::Custom(path.to_string())),
      if is_document && !has_doctype {
        format!("<!DOCTYPE html>{source}")
      } else {
        source
      },
    );

    let mut errors = vec![];

    if is_document {
      let document = parse_file_as_document(fm.as_ref(), ParserConfig::default(), &mut errors);
      let diagnostics: Vec<rspack_error::Diagnostic> = errors
        .into_iter()
        .flat_map(|error| vec![html_parse_error_to_traceable_error(error, &fm).into()])
        .collect();
      document
        .map(|doc| {
          if has_doctype {
            CompiledDocument::Document(doc).with_diagnostic(diagnostics)
          } else {
            CompiledDocument::DocumentWithoutDoctype(doc).with_diagnostic(diagnostics)
          }
        })
        .map_err(|e| html_parse_error_to_traceable_error(e, &fm))
    } else {
      let context_element = create_html_content_element();
      let document_fragment = parse_file_as_document_fragment(
        fm.as_ref(),
        &context_element,
        DocumentMode::Quirks,
        None,
        ParserConfig::default(),
        &mut errors,
      );
      let diagnostics: Vec<rspack_error::Diagnostic> = errors
        .into_iter()
        .flat_map(|error| vec![html_parse_error_to_traceable_error(error, &fm).into()])
        .collect();
      document_fragment
        .map(|doc| CompiledDocument::DocumentFragment(doc).with_diagnostic(diagnostics))
        .map_err(|e| html_parse_error_to_traceable_error(e, &fm))
    }
  }

  pub fn codegen(&self, ast: &mut CompiledDocument, compilation: &Compilation) -> Result<String> {
    let writer_config = BasicHtmlWriterConfig::default();
    let minify = self.config.minify.unwrap_or(matches!(
      compilation.options.mode,
      rspack_core::Mode::Production
    ));
    let codegen_config = CodegenConfig {
      minify,
      quotes: Some(true),
      tag_omission: Some(false),
      ..Default::default()
    };
    if minify {
      // Minify can't leak to user land because it doesn't implement `ToNapiValue` Trait
      GLOBALS.set(&Default::default(), || match ast {
        CompiledDocument::Document(ast) | CompiledDocument::DocumentWithoutDoctype(ast) => {
          minify_document_with_custom_css_minifier(
            ast,
            &MinifyOptions::<()>::default(),
            &NoopCssMinifier,
          )
        }
        CompiledDocument::DocumentFragment(ast) => {
          let context_element = create_html_content_element();
          minify_document_fragment_with_custom_css_minifier(
            ast,
            &context_element,
            &MinifyOptions::<()>::default(),
            &NoopCssMinifier,
          )
        }
      })
    }

    let mut output = String::new();
    let wr = BasicHtmlWriter::new(&mut output, None, writer_config);
    let mut r#gen = CodeGenerator::new(wr, codegen_config);
    ast.emit_to_codegen(&mut r#gen)?;
    if matches!(ast, CompiledDocument::DocumentWithoutDoctype(_)) {
      Ok(output.cow_replace("<!DOCTYPE html>", "").to_string())
    } else {
      Ok(output)
    }
  }
}

fn create_html_content_element() -> Element {
  Element {
    span: DUMMY_SP,
    tag_name: "html".into(),
    namespace: Namespace::HTML,
    attributes: vec![],
    children: vec![],
    content: None,
    is_self_closing: false,
  }
}

pub fn html_parse_error_to_traceable_error(error: Error, fm: &SourceFile) -> rspack_error::Error {
  let message = error.message();
  let error = error.into_inner();
  let span = error.0;
  rspack_error::Error::from_string(
    Some(fm.src.clone().into_string()),
    span.real_lo() as usize,
    span.real_hi() as usize,
    "HTML parse error".to_string(),
    message.to_string(),
  )
}

struct NoopCssMinifier;

impl MinifyCss for NoopCssMinifier {
  type Options = ();

  fn minify_css(
    &self,
    _options: &swc_html_minifier::option::MinifyCssOption<Self::Options>,
    data: String,
    _mode: swc_html_minifier::CssMinificationMode,
  ) -> Option<String> {
    Some(data)
  }
}
