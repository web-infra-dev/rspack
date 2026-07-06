struct BuildResult;
struct CodeGenerationResult;
struct ParseResult;

fn bad_large_return() -> Result<BuildResult, ()> {
  todo!()
}

fn bad_large_option() -> Option<ParseResult> {
  todo!()
}

fn good_large_return() -> Result<Box<BuildResult>, ()> {
  todo!()
}

fn bad_large_param(_: CodeGenerationResult) {}
fn good_large_param(_: &CodeGenerationResult) {}

fn main() {
  let _ = bad_large_return;
  let _ = bad_large_option;
  let _ = good_large_return;
  let _ = bad_large_param;
  let _ = good_large_param;
}
