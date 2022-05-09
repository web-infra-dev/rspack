use once_cell::sync::Lazy;
use std::{path::Path, sync::Arc};
use swc::{config::IsModule, Compiler};
use tracing::instrument;

use ast::{ModuleDecl, ModuleItem};

use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

use crate::{BundleOptions, NormalizedBundleOptions};

pub mod path;
pub mod plugin_hook;

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(Compiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<Compiler> {
  COMPILER.clone()
}

#[instrument(skip(source_code))]
pub fn parse_file(source_code: String, filename: &str) -> ast::Program {
  let syntax = syntax(filename);
  let compiler = get_swc_compiler();
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom(filename.to_string()), source_code);
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

pub fn syntax(filename: &str) -> Syntax {
  let p = Path::new(filename);
  let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
  match ext == "ts" || ext == "tsx" {
    true => Syntax::Typescript(TsConfig {
      decorators: false,
      tsx: ext == "tsx",
      ..Default::default()
    }),
    false => Syntax::Es(EsConfig {
      private_in_object: true,
      import_assertions: true,
      jsx: ext == "jsx",
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
      allow_super_outside_method: true,
    }),
  }
}

pub fn normalize_bundle_options(options: BundleOptions) -> NormalizedBundleOptions {
  NormalizedBundleOptions {
    react: options.react,
    loader: options.loader,
    mode: options.mode,
    entries: options.entries,
    minify: options.minify,
    outdir: options.outdir,
    entry_file_names: options.entry_file_names,
    code_splitting: options.code_splitting,
    root: options.root,
  }
}
