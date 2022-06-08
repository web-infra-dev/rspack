use once_cell::sync::Lazy;
use rspack_swc::{swc, swc_common, swc_ecma_ast as ast, swc_ecma_parser};
use std::{path::Path, sync::Arc};
use swc::{config::IsModule, Compiler};
use tracing::instrument;

use swc_common::{FileName, FilePathMapping, SourceMap};
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{EsConfig, TsConfig};

use crate::{BundleOptions, Loader, NormalizedBundleOptions};

pub mod path;
pub mod plugin_hook;
pub mod swc_builder;
pub mod test_runner;

static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
  let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));

  Arc::new(Compiler::new(cm))
});

pub fn get_swc_compiler() -> Arc<Compiler> {
  COMPILER.clone()
}

#[instrument(skip(source_code))]
pub fn parse_file(source_code: &str, filename: &str, loader: &Loader) -> ast::Program {
  let syntax = syntax_by_loader(filename, loader);
  let compiler = get_swc_compiler();
  let fm = compiler.cm.new_source_file(
    FileName::Custom(filename.to_string()),
    source_code.to_string(),
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

pub fn syntax_by_ext(ext: &str) -> Syntax {
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

pub fn syntax_by_loader(filename: &str, loader: &Loader) -> Syntax {
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
    _ => {
      let ext = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("js");
      syntax_by_ext(ext)
    }
  }
}

pub fn inject_options(mut options: BundleOptions) -> NormalizedBundleOptions {
  use crate::BundleMode;

  match options.mode {
    BundleMode::Dev => {
      options.define.insert(
        "process.env.NODE_ENV".to_owned(),
        "\"development\"".to_owned(),
      );
    }
    BundleMode::Prod => {
      options.define.insert(
        "process.env.NODE_ENV".to_owned(),
        "\"production\"".to_owned(),
      );
    }
    BundleMode::None => (),
  };

  let loader = &mut options.loader;
  loader.entry("json".to_string()).or_insert(Loader::Json);
  loader.entry("js".to_string()).or_insert(Loader::Js);
  loader.entry("jsx".to_string()).or_insert(Loader::Jsx);
  loader.entry("ts".to_string()).or_insert(Loader::Ts);
  loader.entry("tsx".to_string()).or_insert(Loader::Tsx);
  loader.entry("css".to_string()).or_insert(Loader::Css);
  loader.entry("less".to_string()).or_insert(Loader::Less);
  loader.entry("sass".to_string()).or_insert(Loader::Sass);
  loader.entry("scss".to_string()).or_insert(Loader::Sass);
  options
}
