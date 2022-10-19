use crate::utils::get_swc_compiler;
use rspack_core::Devtool;
use rspack_error::Result;
use swc::config::SourceMapsConfig;
use swc::TransformOutput;
use swc_common::GLOBALS;
use swc_ecma_ast::{EsVersion, Program};

pub fn stringify(ast: &Program, devtool: &Devtool) -> Result<TransformOutput> {
  GLOBALS
    .set(&Default::default(), || {
      get_swc_compiler().print(
        ast,
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
