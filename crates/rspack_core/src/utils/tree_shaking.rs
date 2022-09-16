use std::sync::Arc;

use dashmap::DashMap;
use swc::SwcComments;
use swc_common::{
  comments::{self, Comment},
  BytePos, Globals, Mark, SourceMap, SyntaxContext, DUMMY_SP, GLOBALS,
};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_transforms::resolver;
use swc_ecma_visit::{noop_visit_mut_type, swc_ecma_ast::Module, VisitMut, VisitMutWith};

struct Noop {
  trailing: Arc<DashMap<BytePos, Vec<Comment>, ahash::RandomState>>,
}
impl Noop {}
impl VisitMut for Noop {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, n: &mut swc_ecma_visit::swc_ecma_ast::Ident) {
    let ctxt = n.span.ctxt;
    let hi = n.span.hi;
    if SyntaxContext::empty() != ctxt {
      match self.trailing.entry(hi) {
        dashmap::mapref::entry::Entry::Occupied(mut value) => {
          value.get_mut().insert(
            1,
            Comment {
              kind: comments::CommentKind::Block,
              span: DUMMY_SP,
              text: format!("#{}", ctxt.as_u32()).into(),
            },
          );
        }
        dashmap::mapref::entry::Entry::Vacant(entry) => {
          entry.insert(vec![Comment {
            kind: comments::CommentKind::Block,
            span: DUMMY_SP,
            text: format!("#{}", ctxt.as_u32()).into(),
          }]);
        }
      };
    }
  }
}

pub fn dump_ast_with_ctxt(
  module: &mut Module,
  global: &Globals,
  comment: SwcComments,
  cm: Arc<SourceMap>,
) -> String {
  GLOBALS.set(global, || {
    module.visit_mut_with(&mut resolver(Mark::new(), Mark::new(), false));
    let mut noop = Noop {
      trailing: comment.trailing.clone(),
    };
    noop.visit_mut_module(module);
  });
  let code = {
    let mut buf = vec![];

    {
      let mut emitter = Emitter {
        cfg: swc_ecma_codegen::Config {
          ..Default::default()
        },
        cm: cm.clone(),
        comments: Some(&comment),
        wr: JsWriter::new(cm, "\n", &mut buf, None),
      };

      emitter.emit_module(module).unwrap();
    }

    String::from_utf8_lossy(&buf).to_string()
  };
  code
}
#[cfg(test)]
mod test_tree_shaking {
  use std::{path::Path, sync::Arc};

  use super::*;
  use once_cell::sync::Lazy;
  use swc::{config::IsModule, Compiler, SwcComments};
  use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, FilePathMapping, Globals, SourceMap,
  };

  use swc_ecma_parser::{EsConfig, Syntax, TsConfig};

  pub static COMPILER: Lazy<Arc<Compiler>> = Lazy::new(|| {
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    Arc::new(Compiler::new(cm))
  });

  pub fn parse_file(
    source_code: String,
    filename: &str,
    comments: &SwcComments,
  ) -> Result<Module, anyhow::Error> {
    let compiler = COMPILER.clone();
    let fm = compiler
      .cm
      .new_source_file(FileName::Custom(filename.to_owned()), source_code);
    let handler =
      Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(compiler.cm.clone()));
    let p = Path::new(filename);
    let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
    let syntax = if ext == "ts" || ext == "tsx" {
      Syntax::Typescript(TsConfig {
        decorators: false,
        tsx: ext == "tsx",
        ..Default::default()
      })
    } else {
      Syntax::Es(EsConfig {
        private_in_object: true,
        import_assertions: true,
        jsx: ext == "jsx",
        export_default_from: true,
        decorators_before_export: true,
        decorators: true,
        fn_bind: true,
        ..Default::default()
      })
    };
    compiler
      .parse_js(
        fm,
        &handler,
        swc_ecma_visit::swc_ecma_ast::EsVersion::Es2022,
        syntax,
        IsModule::Bool(true),
        Some(&comments),
      )
      .map(|prog| prog.expect_module())
  }
  #[test]
  fn test_tree_shaking_dump() {
    let source = r#"
import { test } from "./test.js";
test;
{
  var b = 1000;
}
b;
// err = 3;
let a = 10;

function another() {
  let a = 100;
}

another;
    "#;
    let expected = r#"
import { test/*#1*/  } from "./test.js";
test /*#1*/ ;
{
    var b /*#1*/  = 1000;
}b /*#1*/ ;
// err = 3;
let a /*#1*/  = 10;
function another() {
    let a /*#2*/  = 100;
}
another /*#1*/ ;
"#;
    let comment = SwcComments::default();
    let globals = Globals::default();
    let mut module = parse_file(source.to_string(), "dojo.js", &comment).unwrap();
    let code = dump_ast_with_ctxt(&mut module, &globals, comment, COMPILER.cm.clone());

    assert_eq!(code.trim(), expected.trim());
  }
}
