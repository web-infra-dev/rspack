use crate::utils::get_swc_compiler;
use rspack_core::{ast::javascript::Ast, Devtool};
use rspack_error::Result;
use swc::config::SourceMapsConfig;
use swc::TransformOutput;
use swc_ecma_ast::EsVersion;

pub fn stringify(ast: &Ast, devtool: &Devtool) -> Result<TransformOutput> {
  ast
    .transform(|program, _context| {
      get_swc_compiler().print(
        program.get_inner_program(),
        None,
        None,
        !devtool.no_sources(),
        EsVersion::Es2022,
        SourceMapsConfig::Bool(devtool.source_map()),
        &Default::default(),
        //orig,
        None,
        false,
        None,
        !devtool.cheap(),
        false,
      )
    })
    .map_err(|e| e.into())
}
