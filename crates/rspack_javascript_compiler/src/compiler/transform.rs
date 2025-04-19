use rspack_error::BatchErrors;
use swc_core::{
  base::config::IsModule,
  common::FileName,
  ecma::{ast::EsVersion, parser::Syntax},
};
use swc_node_comments::SwcComments;

use super::{JavaScriptCompiler, TransformOutput};

impl JavaScriptCompiler {
  pub fn transform<S: Into<String>>(
    &self,
    filename: FileName,
    source: S,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<SwcComments>,
  ) -> Result<TransformOutput, BatchErrors> {
    todo!("Implment transform");

    // self.run(|| -> Result<TransformOutput, BatchErrors> {
    //   with_rspack_error_handler(
    //     "Transform Error".to_string(),
    //     DiagnosticKind::JavaScript,
    //     self.cm.clone(),
    //     |handler| {
    //       // let fm = self.cm.new_source_file(Arc::new(filename), source.into());
    //       // let lexer = Lexer::new(
    //       //   syntax,
    //       //   target,
    //       //   SourceFileInput::from(&*fm),
    //       //   comments.as_ref().map(|c| c as &dyn Comments),
    //       // );

    //       // let program = parse_with_lexer(lexer, is_module)?;

    //       let mut cm = self.cm.clone();
    //       let mut codegen_config = CodegenConfig::default();
    //       codegen_config.minify = false;
    //       codegen_config.source_maps = Some(SourceMapConfig::default());

    //       let mut buf = Vec::new();
    //       let mut map = None;

    //       if let Some(source_map) = codegen_config.source_maps {
    //         map = Some(source_map);
    //       }

    //       let mut output = String::new();
    //       print(&mut output, &program, &cm, &codegen_config)?;

    //       Ok(TransformOutput { code: output, map })
    //     },
    //   )
    // })
  }
}
