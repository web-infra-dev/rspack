use std::{
  env, fs,
  path::{Path, PathBuf},
  str::FromStr,
};

use rspack_core::{
  CompilationContext, CompilerContext, CompilerOptions, Loader, LoaderRunner,
  LoaderRunnerAdditionalContext, ResourceData, SideEffectOption,
};
use rspack_loader_sass::{SassLoader, SassLoaderOptions};
use rspack_testing::{fixture, test_fixture};
use sass_embedded::Url;

// UPDATE_SASS_LOADER_TEST=1 cargo test --package rspack_loader_sass test_fn_name -- --exact --nocapture
async fn loader_test(actual: impl AsRef<Path>, expected: impl AsRef<Path>) {
  let tests_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"))).join("tests");
  let expected_path = tests_path.join(expected);
  let actual_path = tests_path.join(actual);

  let url = Url::from_file_path(actual_path.to_string_lossy().to_string()).expect("TODO:");
  let (result, _) = LoaderRunner::new(
    ResourceData {
      resource: actual_path.to_string_lossy().to_string(),
      resource_path: url.to_file_path().expect("bad url file path"),
      resource_query: url.query().map(|q| q.to_owned()),
      resource_fragment: url.fragment().map(|f| f.to_owned()),
      resource_description: None,
    },
    vec![],
  )
  .run(
    [&SassLoader::new(SassLoaderOptions::default())
      as &dyn Loader<CompilerContext, CompilationContext>],
    &LoaderRunnerAdditionalContext {
      compiler: &CompilerContext {
        options: std::sync::Arc::new(CompilerOptions {
          entry: std::collections::HashMap::default(),
          context: rspack_core::Context::default(),
          dev_server: rspack_core::DevServerOptions::default(),
          devtool: rspack_core::Devtool::default(),
          mode: rspack_core::Mode::None,
          output: rspack_core::OutputOptions {
            path: Default::default(),
            public_path: Default::default(),
            filename: rspack_core::Filename::from_str("").expect("TODO:"),
            asset_module_filename: rspack_core::Filename::from_str("").expect("TODO:"),
            chunk_filename: rspack_core::Filename::from_str("").expect("TODO:"),
            unique_name: Default::default(),
            css_chunk_filename: rspack_core::Filename::from_str("").expect("TODO:"),
            css_filename: rspack_core::Filename::from_str("").expect("TODO:"),
            library: None,
            enabled_library_types: None,
            strict_module_error_handling: false,
            global_object: "self".to_string(),
            import_function_name: "import".to_string(),
          },
          target: rspack_core::Target::new(&vec![String::from("web")]).expect("TODO:"),
          resolve: rspack_core::Resolve::default(),
          builtins: Default::default(),
          module: Default::default(),
          stats: Default::default(),
          cache: Default::default(),
          snapshot: Default::default(),
          experiments: Default::default(),
          node: Default::default(),
          optimization: rspack_core::Optimization {
            remove_available_modules: false,
            side_effects: SideEffectOption::False,
          },
        }),
        resolver_factory: Default::default(),
      },
      compilation: &(),
    },
  )
  .await
  .expect("TODO:")
  .split_into_parts();
  let result = result.content.try_into_string().expect("TODO:");

  if env::var("UPDATE_SASS_LOADER_TEST").is_ok() {
    fs::write(expected_path, result).expect("TODO:");
  } else {
    let expected = fs::read_to_string(expected_path).expect("TODO:");
    assert_eq!(result, expected);
  }
}

#[tokio::test]
async fn rspack_importer() {
  loader_test("scss/language.scss", "expected/rspack_importer.css").await;
}

#[fixture("tests/fixtures/*")]
fn sass(fixture_path: PathBuf) {
  test_fixture(&fixture_path);
}
