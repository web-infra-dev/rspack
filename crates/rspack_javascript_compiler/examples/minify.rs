use rspack_javascript_compiler::JavaScriptCompiler;

fn main() {
  let source = "const a = 10;";

  let compiler = JavaScriptCompiler::new();
  let output = compiler.minify(
    swc_core::common::FileName::Custom("test.js".to_string()),
    source,
    &Default::default(),
    None::<&dyn Fn(&swc_core::common::comments::SingleThreadedComments)>,
  );

  match output {
    Ok(o) => {
      println!("Minified output: {o:?}");
    }
    Err(err) => {
      let e = err
        .into_inner()
        .into_iter()
        .map(|e| format!("{e:?}"))
        .collect::<Vec<_>>()
        .join("\n");
      eprintln!("{e:?}");
    }
  }
}
