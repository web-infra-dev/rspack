use std::{rc::Rc, sync::Arc};

use rspack_javascript_compiler::JavaScriptCompiler;
use swc_core::{common::comments::SingleThreadedComments, ecma::ast::noop_pass};

fn main() {
  let source = "const a = 10;";

  let compiler = JavaScriptCompiler::new();
  let comments = Rc::new(SingleThreadedComments::default());
  let s = compiler.transform(
    source,
    Some(Arc::new(swc_core::common::FileName::Custom(
      "test.js".to_string(),
    ))),
    comments,
    Default::default(),
    None,
    |_, _| {},
    |_| noop_pass(),
  );

  match s {
    Ok(output) => {
      println!("Transformed output: {output:?}");
    }
    Err(err) => {
      eprintln!("{err}");
    }
  }
}
