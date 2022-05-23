mod testing {
  use async_trait::async_trait;
  use rspack::bundler::{BundleContext, BundleOptions, Bundler};

  use rspack_core::{LoadArgs, Loader, ResolveArgs, ResolveOption};
  use rspack_core::{
    Plugin, PluginLoadHookOutput, PluginResolveHookOutput, PluginTransformAstHookOutput,
  };

  use rspack_swc::swc_ecma_ast;
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env;
  use std::ffi::OsString;
  use std::fs;
  use std::path::Path;
  use std::sync::atomic::AtomicBool;
  use std::sync::Arc;
  use std::sync::Once;

  static INIT: Once = Once::new();

  fn compile(fixture_path: &str, plugins: Vec<Box<dyn Plugin>>) -> Bundler {
    INIT.call_once(|| {
      let default_panic = std::panic::take_hook();
      std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
      }));
    });
    compile_with_options(fixture_path, Default::default(), plugins)
  }

  fn compile_with_options(
    fixture_path: &str,
    options: BundleOptions,
    plugins: Vec<Box<dyn Plugin>>,
  ) -> Bundler {
    compile_with_options_inner(fixture_path, options, plugins)
  }

  #[tokio::main]
  async fn compile_with_options_inner(
    fixture_path: &str,
    options: BundleOptions,
    plugins: Vec<Box<dyn Plugin>>,
  ) -> Bundler {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = dir.join("fixtures").join(fixture_path);
    let pkg_path = fixtures_dir.join("package.json");
    let pkg_content = fs::read_to_string(pkg_path);
    let mut pkg: Value = Value::default();
    if pkg_content.is_ok() {
      pkg = serde_json::from_str(&pkg_content.unwrap()).unwrap();
    }
    // use pkg.rspack.entry if set otherwise use index.js as entry
    let pkg_entry = pkg["rspack"].clone()["entry"].clone();
    let entry = {
      if pkg_entry.is_object() {
        let obj: HashMap<String, String> = serde_json::from_value(pkg_entry).unwrap();
        obj
          .into_iter()
          .map(|(_id, value)| {
            let resolve_path = fixtures_dir.join(value).display().to_string();
            resolve_path
          })
          .collect()
      } else {
        let default_entry = fixtures_dir.join("index.js").to_str().unwrap().to_string();
        vec![default_entry]
      }
    };
    let svgr = pkg["rspack"].clone()["svgr"].as_bool().unwrap_or(false);
    let dist = fixtures_dir.join("dist");
    println!("entry: {:?}", entry);
    println!("options: \n {:?}", options);
    let mut bundler = Bundler::new(
      BundleOptions {
        entries: entry.into_iter().map(From::from).collect(),
        outdir: dist.to_str().unwrap().to_string(),
        svgr,
        ..options
      },
      plugins,
    );
    bundler.build(None).await;
    bundler.write_assets_to_disk();
    bundler
  }

  fn assert_inline_sourcemap_in_pos(
    dist_code: &String,
    line_in_dist: u32,
    column_in_dist: u32,
    expected_in_source: &str,
  ) {
    const DATA_PREAMBLE: &str = "data:application/json;charset=utf-8;base64,";
    // TODO: should find last DATA_PREAMBLE.
    let index = dist_code.find(DATA_PREAMBLE).unwrap();
    let data_b64 = &dist_code[index + DATA_PREAMBLE.len()..];
    let data = base64::decode(data_b64).unwrap();
    let decoded_map = sourcemap::decode_slice(&data).unwrap();
    let token = decoded_map
      .lookup_token(line_in_dist, column_in_dist)
      .unwrap();
    let source_view = token.get_source_view().unwrap();
    let actual = source_view
      .get_line_slice(
        token.get_src_line(),
        token.get_src_col(),
        expected_in_source.len() as u32,
      )
      .unwrap();
    assert_eq!(actual, expected_in_source);
  }

  #[test]
  fn single_entry() {
    compile("single-entry", vec![]);
  }

  #[test]
  fn multi_entry() {
    compile("multi-entry", vec![]);
  }

  #[test]
  fn cycle_dep() {
    compile("cycle-dep", vec![]);
  }

  #[derive(Debug)]
  struct TestPlugin {
    call_resolve: Arc<AtomicBool>,
    call_load: Arc<AtomicBool>,
    call_transform: Arc<AtomicBool>,
  }

  #[async_trait]
  impl Plugin for TestPlugin {
    fn name(&self) -> &'static str {
      "rspack_test"
    }

    async fn resolve(&self, _ctx: &BundleContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
      self
        .call_resolve
        .store(true, std::sync::atomic::Ordering::SeqCst);
      None
    }

    #[inline]
    async fn load(&self, _ctx: &BundleContext, _args: &LoadArgs) -> PluginLoadHookOutput {
      self
        .call_load
        .store(true, std::sync::atomic::Ordering::SeqCst);
      None
    }

    #[inline]
    fn transform_ast(
      &self,
      _ctx: &BundleContext,
      _path: &Path,
      ast: swc_ecma_ast::Module,
    ) -> PluginTransformAstHookOutput {
      self
        .call_transform
        .store(true, std::sync::atomic::Ordering::SeqCst);
      ast
    }
  }

  #[test]
  fn plugin_test() {
    let call_resolve: Arc<AtomicBool> = Default::default();
    let call_load: Arc<AtomicBool> = Default::default();
    let call_transform: Arc<AtomicBool> = Default::default();
    let test_plugin = Box::new(TestPlugin {
      call_resolve: call_resolve.clone(),
      call_load: call_load.clone(),
      call_transform: call_transform.clone(),
    });
    compile("single-entry", vec![test_plugin]);
    assert!(call_load.load(std::sync::atomic::Ordering::SeqCst));
    assert!(call_resolve.load(std::sync::atomic::Ordering::SeqCst));
    assert!(call_transform.load(std::sync::atomic::Ordering::SeqCst));
  }

  #[test]
  fn dynamic_import() {
    compile("dynamic-import", vec![]);
  }

  #[test]
  fn basic_css() {
    let bundler = compile("basic-css", vec![]);
    println!(
      "plugin_name -> \n {:#?}",
      bundler
        .plugin_driver
        .plugins
        .iter()
        .map(|x| x.name().to_string())
        .collect::<Vec<String>>()
    );
    assert!(bundler
      .plugin_driver
      .plugins
      .iter()
      .find(|plugin| plugin.name() == rspack_plugin_stylesource::plugin::PLUGIN_NAME)
      .is_some())
  }

  #[test]
  #[ignore = "not support npm yet"]
  fn npm() {
    compile("npm", vec![]);
  }

  #[test]
  fn cjs() {
    compile("cjs", vec![]);
  }

  #[test]
  fn css_bundle_test() {
    compile_with_options(
      "css",
      BundleOptions {
        loader: HashMap::from_iter([
          ("css".to_string(), Loader::Css),
          ("less".to_string(), Loader::Less),
          ("sass".to_string(), Loader::Sass),
          ("scss".to_string(), Loader::Sass),
          ("svg".to_string(), Loader::DataURI),
        ]),
        ..Default::default()
      },
      vec![],
    );

    pub fn path_resolve(path: &str) -> String {
      let work_cwd = env!("CARGO_MANIFEST_DIR");
      let os_work_cwd = OsString::from(work_cwd);
      Path::new(&os_work_cwd)
        .join(path)
        .into_os_string()
        .into_string()
        .unwrap()
    }

    let _dist_css_file1 = path_resolve("fixtures/css/dist/index.css");
    let _dist_css_file2 = path_resolve("fixtures/css/dist/liba.css");
    // FIXME: The output filename of chunk is not stable now, should not rely on it.
    // assert_eq!(Path::new(dist_css_file1.as_str()).exists(), true);
    // assert_eq!(Path::new(dist_css_file2.as_str()).exists(), true);
  }

  #[test]
  fn disable_code_splitting() {
    let bundler = compile_with_options(
      "basic",
      BundleOptions {
        code_splitting: None,
        ..Default::default()
      },
      vec![],
    );
    let chunk_len = bundler.bundle.context.assets.lock().unwrap().len();
    assert_eq!(chunk_len, 2);
  }

  #[test]
  fn enable_code_splitting() {
    let bundler = compile_with_options(
      "basic",
      BundleOptions {
        code_splitting: Some(Default::default()),
        ..Default::default()
      },
      vec![],
    );
    let chunk_len = bundler.bundle.context.assets.lock().unwrap().len();
    assert_eq!(chunk_len, 3);
  }

  #[test]
  fn basic_ts() {
    compile("basic-ts", vec![]);
  }

  #[test]
  fn svgr() {
    compile("svgr", vec![]);
  }

  #[test]
  fn splitting() {
    compile("code-splitting", vec![]);
  }

  #[test]
  fn loader() {
    compile_with_options(
      "loader",
      BundleOptions {
        loader: vec![
          ("svg".to_string(), Loader::DataURI),
          // Json is supported by default
          // ("json".to_string(), Loader::Json),
          ("txt".to_string(), Loader::Text),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
      },
      vec![],
    );
  }

  #[test]
  fn alias() {
    let bundler = compile_with_options(
      "alias",
      BundleOptions {
        resolve: ResolveOption {
          alias: vec![
            ("./wrong".to_string(), Some("./ok".to_string())),
            ("@/".to_string(), Some("./src/".to_string())),
          ],
          ..Default::default()
        },
        ..Default::default()
      },
      vec![],
    );
    let assets = bundler.bundle.context.assets.lock().unwrap();
    let dist = assets.get(0).unwrap();
    let source = &dist.source;
    println!("assets {:#?}", assets);
    assert!(!source.contains("wrong.js"));
    assert!(!source.contains('@'));
    assert!(source.contains("ok.js"));
    assert!(source.contains("at.js"));
  }

  #[test]
  fn stack_overflow_mockjs() {
    compile("stack_overflow_mockjs", vec![]);
  }

  #[test]
  fn define() {
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
        ("M1.M2.M3", "{}"),
        ("SHOULD_CONVERTED", "205"),
      ]
      .map(|(k, v)| (k.to_string(), v.to_string())),
    );
    let bundler = compile_with_options(
      "define",
      BundleOptions {
        define,
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
    assert!(source.contains("0.ABC"));
    assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO"));
    assert!(source.contains("MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP"));
    assert!(source.contains("/abc/i"));
    assert!(!source.contains("ARRAY;")); // ';` is just for positioning.
    assert!(source.contains(&format!("[300, [{}]]", "\"six\"")));
    assert!(!source.contains("ARRAY[0]"));
    assert!(source.contains(&format!("[300, [{}]][0]", "\"six\""))); // TODO: maybe could continue optimized.
    assert!(!source.contains("ARRAY[0][1]"));
    assert!(source.contains(&format!("[300, [{}]][0][1]", "\"six\"")));
    assert!(!source.contains("ARRAY[1]"));
    assert!(source.contains(&format!("[300, [{}]][1]", "\"six\"")));
    assert!(!source.contains("ARRAY[1][0]"));
    assert!(source.contains(&format!("[300, [{}]][1][0]", "\"six\"")));
    assert!(!source.contains("ARRAY[1][0][0]"));
    assert!(source.contains(&format!("[300, [{}]][1][0][0]", "\"six\"")));
    assert!(!source.contains("ARRAY[ARRAY]"));
    assert!(source.contains(&format!("[300, [{}]][[300, [{}]]]", "\"six\"", "\"six\"")));
    assert!(!source.contains("OBJECT; // tags"));
    assert!(!source.contains("OBJECT.OBJ;"));
    assert!(!source.contains("OBJECT.OBJ.NUM;"));
    assert!(!source.contains("OBJECT.UNDEFINED;"));
    assert!(!source.contains("OBJECT.REGEXP;"));
    assert!(!source.contains("OBJECT.STR;"));
    let obj = format!(
      "({{UNDEFINED: undefined, REGEXP: /def/i, STR: {}, OBJ: {{ NUM: 1}}}})",
      "\"string\""
    );
    assert!(source.contains(&format!("{}.OBJ", obj)));
    assert!(source.contains(&format!("{}.OBJ.NUM", obj)));
    assert!(source.contains(&format!("{}.UNDEFINED", obj)));
    assert!(source.contains(&format!("{}.REGEXP", obj)));
    assert!(source.contains(&format!("{}.AAA.BBB", obj)));

    assert!(source.contains("301, 301"));
    assert!(source.contains(&format!("{}, {}", "\"302\"", "\"302\"")));
    assert!(source.contains("303, 303"));
    assert!(source.contains("304, 304"));
    assert!(source.contains("303.P4"));
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
    assert!(source.contains("({}).DO_NOT_CONVERTED6"));
    assert!(source.contains("{}.DO_NOT_CONVERTED5"));

    // identifier
    assert_inline_sourcemap_in_pos(source, 74, 4, "TRUE");
    // member// 2
    assert_inline_sourcemap_in_pos(source, 107, 4, "ARRAY");
    assert_inline_sourcemap_in_pos(source, 129, 4, "P1.P2.P4");
    // assign
    assert_inline_sourcemap_in_pos(source, 164, 4, "SHOULD_CONVERTED");
  }
}
