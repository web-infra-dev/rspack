mod utils;
use rspack::bundler::BundleOptions;
use rspack_core::RuntimeOptions;
use std::collections::HashMap;
use utils::{assert_inline_sourcemap_in_pos, compile_with_options};

#[test]
fn define() {
  let runtime = RuntimeOptions {
    hmr: false,
    module: false,
    polyfill: false,
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
      ..Default::default()
    },
    vec![],
  );

  let assets = bundler.bundle.context.assets.lock().unwrap();
  let dist = assets.get(0).unwrap();
  let source = &dist.source;

  assert!(!source.contains("TRUE"));
  assert!(source.contains("true"));
  assert!(!source.contains("FALSE"));
  assert!(source.contains("false"));
  assert!(!source.contains("NUMBER_ADD"));
  assert!(source.contains("3 + 2")); // TODO: optimized it to `5`
  assert!(!source.contains("NULL"));
  assert!(source.contains("null"));
  assert!(!source.contains("UNDEFINED; // tags"));
  assert!(source.contains("undefined"));
  assert!(!source.contains("NUMBER"));
  assert!(source.contains("100.05"));
  assert!(!source.contains("ZERO; // tags")); // `;` and `// tags` in arg is for positioning.
  assert!(source.contains('0'));
  assert!(!source.contains("({ZERO:0}.ZERO);"));
  assert!(!source.contains("({ZERO:0}).ZERO;"));
  assert!(!source.contains("({ZERO:0})[ZERO];"));
  assert!(!source.contains("({ZERO:0})[0];"));
  assert!(!source.contains("({ZERO:0})['ZERO'];"));
  // TODO: should optimized test case structure.
  assert!(source.contains("ZERO: 0"));
  assert!(source.contains("}).ZERO;"));
  assert!(source.contains("})[0];"));
  assert!(source.contains(&format!("}})[{}];", "\"ZERO\"")));
  // ...
  assert!(!source.contains("BIGINT"));
  assert!(source.contains("BigInt(10000)")); // TODO: `BigInt` could be changed to it suffix with `n`.
  assert!(!source.contains("BIGINT2"));
  assert!(source.contains("100000000000n"));
  assert!(!source.contains("POSITIVE_ZERO"));
  assert!(source.contains("+0")); // TODO unary could be optimized
  assert!(!source.contains("NEGATIVE_ZERO"));
  assert!(source.contains("-0"));
  assert!(!source.contains("POSITIVE_NUMBER"));
  assert!(source.contains("+100.25"));
  assert!(!source.contains("NEGATIVE_NUMBER"));
  assert!(source.contains("-100.25"));
  assert!(!source.contains("EMPTY_STRING"));
  assert!(source.contains("\"\""));
  assert!(!source.contains("REGEXP; // tags"));
  assert!(!source.contains("ZERO.REGEXP"));
  assert!(source.contains("0..ABC"));
  assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO"));
  assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP"));
  assert!(source.contains("/abc/i"));
  assert!(!source.contains("ARRAY;")); // ';` is just for positioning.
  let array = "    [\n        300,\n        [\n            \"six\"\n        ]\n    ]";
  assert!(source.contains(&array));
  assert!(!source.contains("ARRAY[0]"));
  assert!(source.contains(&format!("{}[0]", array))); // TODO: maybe could continue optimized.
  assert!(!source.contains("ARRAY[0][1]"));
  assert!(source.contains(&format!("{}[0][1]", array))); // TODO: maybe could continue optimized.
  assert!(!source.contains("ARRAY[1]"));
  assert!(source.contains(&format!("{}[1]", array))); // TODO: maybe could continue optimized.
  assert!(!source.contains("ARRAY[1][0]"));
  assert!(source.contains(&format!("{}[1][0]", array))); // TODO: maybe could continue optimized.
  assert!(!source.contains("ARRAY[1][0][0]"));
  assert!(source.contains(&format!("{}[1][0][0]", array))); // TODO: maybe could continue optimized.
  assert!(!source.contains("ARRAY[ARRAY]"));
  assert!(source.contains(&format!("{}[{}]", array, &array[4..])));
  assert!(!source.contains("OBJECT; // tags"));
  assert!(!source.contains("OBJECT.OBJ;"));
  assert!(!source.contains("OBJECT.OBJ.NUM;"));
  assert!(!source.contains("OBJECT.UNDEFINED;"));
  assert!(!source.contains("OBJECT.REGEXP;"));
  assert!(!source.contains("OBJECT.STR;"));
  let obj = "    ({\n        UNDEFINED: undefined,\n        REGEXP: /def/i,\n        STR: \"string\",\n        OBJ: {\n            NUM: 1\n        }\n    })";
  assert!(source.contains(&format!("{}.OBJ", obj)));
  assert!(source.contains(&format!("{}.OBJ.NUM", obj)));
  assert!(source.contains(&format!("{}.UNDEFINED", obj)));
  assert!(source.contains(&format!("{}.REGEXP", obj)));
  assert!(source.contains(&format!("{}.AAA.BBB", obj)));

  assert!(source.contains("301, 301"));
  assert!(source.contains(&format!("{}, {}", "\"302\"", "\"302\"")));
  assert!(source.contains("303, 303"));
  assert!(source.contains("304, 304"));
  assert!(source.contains("303..P4"));
  assert!(source.contains("P4.P1"));
  assert!(source.contains(&format!("{}.P1.P2", "\"302\"")));
  assert!(source.contains(&format!("{}.P3", "\"302\"")));
  assert!(source.contains(&format!("{}.P4", "\"302\"")));

  assert!(!source.contains("DO_NOT_CONVERTED_TAG"));
  assert!(!source.contains("SHOULD_CONVERTED"));
  assert!(source.contains("205 = 205"));
  assert!(source.contains("205 = 205 = 205"));
  assert!(source.contains("aa = 205"));
  assert!(source.contains("205 == 206"));
  assert!(source.contains("207 == 205"));
  assert!(!source.contains("M1.M2.M3.DO_NOT_CONVERTED6"));
  assert!(source.contains("M1, undefined"));
  assert!(source.contains("{}.DO_NOT_CONVERTED5"));
  assert!(source.contains("{}.DO_NOT_CONVERTED6"));
  assert!(source.contains("equal(IN_BLOCK, 2)"));
  assert!(source.contains("SHOULD_BE_CONVERTED_IN_UNDEFINED_BLOCK"));
  assert!(!source.contains("CONVERTED_TO_MEMBER"));
  assert!(source.contains("A1.A2.A3"));

  // identifier
  assert_inline_sourcemap_in_pos(source, 74, 4, "TRUE");
  // member// 2
  assert_inline_sourcemap_in_pos(source, 107, 4, "ARRAY");
  assert_inline_sourcemap_in_pos(source, 223, 4, "P1.P2.P4");
  // assign
  assert_inline_sourcemap_in_pos(source, 269, 4, "SHOULD_CONVERTED");
}
