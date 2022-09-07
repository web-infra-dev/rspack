use swc_common::BytePos;

use swc_css::ast::Stylesheet;
use swc_css::parser::{self, parser::ParserConfig};
fn main() {
  let source = r#"
    
    .test {
  tes-test-test: string;
}
    "#;
  //   let ast: Stylesheet = parser::parse_str(
  //     source,
  //     BytePos(0),
  //     BytePos(0),
  //     ParserConfig::default(),
  //     &mut vec![],
  //   )
  //   .unwrap();
  let hrx_file = r#"<===> test someting

 something else

  
  "#;
}
