use once_cell::sync::Lazy;
use rspack_swc::{swc, swc_common, swc_ecma_ast as ast, swc_ecma_parser};
use std::{path::Path, sync::Arc};
use swc::{config::IsModule, Compiler};
use tracing::instrument;

use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

use crate::{BundleOptions, Loader, NormalizedBundleOptions, ResolvedLoadedFile};

mod loader;
mod options;
pub mod path;
pub mod plugin_hook;
pub mod swc_builder;
pub mod test_runner;
pub use loader::*;
pub use options::*;

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(Compiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<Compiler> {
  COMPILER.clone()
}

#[instrument(skip_all)]
pub fn parse_file(loaded_file: ResolvedLoadedFile, filename: &str) -> ast::Program {
  let loaded_js_file = interpret_loaded_file_to_js(loaded_file, filename);
  let syntax = syntax(&loaded_js_file.loader);
  let compiler = get_swc_compiler();
  let fm = compiler.cm.new_source_file(
    FileName::Custom(filename.to_string()),
    loaded_js_file.content,
  );
  swc::try_with_handler(compiler.cm.clone(), Default::default(), |handler| {
    compiler.parse_js(
      fm,
      handler,
      ast::EsVersion::Es2022,
      syntax,
      IsModule::Bool(true),
      None,
    )
  })
  .unwrap()
}

pub fn syntax(loader: &Loader) -> Syntax {
  match loader {
    Loader::Js | Loader::Jsx => Syntax::Es(EsConfig {
      private_in_object: true,
      import_assertions: true,
      jsx: matches!(loader, Loader::Jsx),
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
      allow_super_outside_method: true,
    }),
    Loader::Ts | Loader::Tsx => Syntax::Typescript(TsConfig {
      decorators: false,
      tsx: matches!(loader, Loader::Tsx),
      ..Default::default()
    }),
    _ => unreachable!(),
  }
}
