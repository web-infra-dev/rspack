mod utils;
use rspack::bundler::BundleOptions;
use rspack_core::RuntimeOptions;
use std::collections::HashMap;
use utils::{assert_inline_sourcemap_in_pos, compile_with_options, run_js_asset_in_node};

#[test]
fn define() {
  let runtime = RuntimeOptions {
    hmr: false,
    module: true,
  };
  let define = HashMap::from_iter(
    [
      ("TRUE", "true"),
      ("FALSE", "false"),
      ("NUMBER_ADD", "3 + 2"),
      // ( // TODO: function
      //   "FUNCTION",
      //   "function(a) {return a + 1;}",
      // ),
      ("NULL", "null"),
      ("UNDEFINED", "undefined"),
      ("NUMBER", "100.05"),
      ("ZERO", "0"),
      ("ONE", "1"),
      ("BIGINT", "BigInt(10000)"),
      ("BIGINT2", "100000000000n"),
      ("POSITIVE_ZERO", "+0"),
      ("NEGATIVE_ZERO", "-0"),
      ("POSITIVE_NUMBER", "+100.25"),
      ("NEGATIVE_NUMBER", "-100.25"),
      ("STRING", "\"string\""),
      ("EMPTY_STRING", "\"\""),
      ("REGEXP", "/abc/i"),
      // TODO: should use `"(2 + 2)"`
      (
        "OBJECT",
        "{UNDEFINED: undefined, REGEXP: /def/i, STR: \"string\", OBJ: { NUM: 1}}",
      ),
      ("ARRAY", &format!("[300, [{}]]", "\"six\"")),
      ("P1.P2.P3", "301"),
      ("P1.P2.P4", "\"302\""),
      ("P1", "303"),
      ("P1.P2", "304"),
      // TODO: recursively
      // ("wurst", "suppe"),
      // ("suppe", "wurst"),
      ("DO_NOT_CONVERTED", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED2", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED3", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED4", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED5", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED6", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED7", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED8", "DO_NOT_CONVERTED_TAG"),
      ("DO_NOT_CONVERTED9", "DO_NOT_CONVERTED_TAG"),
      ("IN_BLOCK", "SHOULD_BE_CONVERTED_IN_UNDEFINED_BLOCK"),
      ("M1.M2.M3", "{}"),
      ("SHOULD_CONVERTED", "205"),
      ("CONVERTED_TO_MEMBER", "A1.A2.A3"),
    ]
    .map(|(k, v)| (k.to_string(), v.to_string())),
  );
  let bundler = compile_with_options(
    "define",
    BundleOptions {
      define,
      runtime,
      platform: rspack_core::Platform::Node,
      ..Default::default()
    },
    vec![],
  );

  let assets = bundler.bundle.context.assets.lock().unwrap();
  let js_assets = assets.get(0).unwrap();

  run_js_asset_in_node(js_assets);
  let source = &js_assets.source;
  assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO"));
  assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP"));
  assert!(source.contains("P4.P1"));
  assert!(source.contains(&format!("{}.P1", "\"302\"")));
  assert!(source.contains(&format!("{}.P3", "\"302\"")));
  assert!(source.contains(&format!("{}.P4", "\"302\"")));
  assert!(source.contains("SHOULD_BE_CONVERTED_IN_UNDEFINED_BLOCK"));
  assert!(source.contains("M1"));
  assert!(source.contains("aa = 205"));
  assert!(source.contains("DO_NOT_CONVERTED4"));
  assert!(!source.contains("DO_NOT_CONVERTED_TAG"));
  assert!(!source.contains("SHOULD_CONVERTED"));
  assert!(!source.contains("CONVERTED_TO_MEMBER"));

  // comment temporary
  // identifier
  // assert_inline_sourcemap_in_pos(source, 74, 4, "TRUE");
  // member// 2
  // assert_inline_sourcemap_in_pos(source, 107, 4, "ARRAY");
  // assert_inline_sourcemap_in_pos(source, 223, 4, "P1.P2.P4");
  // assign
  // assert_inline_sourcemap_in_pos(source, 269, 4, "SHOULD_CONVERTED");
}
