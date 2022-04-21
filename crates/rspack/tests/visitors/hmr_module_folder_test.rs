use rspack::visitors::hmr_module_folder::HmrModuleFolder;
use swc_common::{chain, Mark};
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_transforms_base::resolver::resolver_with_mark;
use swc_ecma_transforms_testing::{test, test_fixture, Tester};
use swc_ecma_visit::Fold;

fn syntax() -> Syntax {
  Syntax::Es(EsConfig {
    ..Default::default()
  })
}

fn tr(_tester: &mut Tester<'_>, file_name: String) -> impl Fold {
  let top_level_mark = Mark::fresh(Mark::root());
  chain!(
    resolver_with_mark(top_level_mark),
    HmrModuleFolder {
      file_name,
      top_level_mark
    }
  )
}

test!(
  syntax(),
  |tester| tr(tester, "/a.js".to_string()),
  basic,
  r#"
  import a from './b';
  export { a };
  "#,
  r#"
  rs.define("/a.js", function(require, module, exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "a", {
        enumerable: true,
        get: function() {
            return _b.default;
        }
    });
    var _b = _interopRequireDefault(require("./b"));
  });
  "#
);
