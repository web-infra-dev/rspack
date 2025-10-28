use rspack_javascript_compiler::JavaScriptCompiler;
use swc_core::ecma::ast::noop_pass;

fn main() {
  let source = "const a = 10;";

  let compiler = JavaScriptCompiler::new();
  let s = compiler.transform(
    source,
    Some(swc_core::common::FileName::Custom("test.js".to_string())),
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
