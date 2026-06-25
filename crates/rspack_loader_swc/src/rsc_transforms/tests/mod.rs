// The fixture cases in this module are adapted from the Next.js
// `next-custom-transforms` RSC and Server Actions tests. Next.js is MIT
// licensed; keep this provenance visible when syncing cases. Expected output
// files are regenerated for Rspack semantics instead of copied from upstream.

use std::{
  borrow::Cow,
  cell::RefCell,
  env, fs,
  path::{Path, PathBuf},
  rc::Rc,
  sync::Arc,
};

use rspack_core::{RscMeta, RscModuleType};
use rspack_javascript_compiler::{JavaScriptCompiler, transform::SwcOptions};
use swc_core::{
  base::config::{IsModule, SourceMapsConfig},
  common::{FileName, comments::SingleThreadedComments},
  ecma::{
    ast::{EsVersion, Pass, noop_pass},
    parser::{EsSyntax, Syntax, TsSyntax},
    transforms::testing::{FixtureTestConfig, test_fixture},
  },
};

use super::{RscTransformOptions, rsc_transform};

const UPDATE_ENV: &str = "RSPACK_UPDATE_RSC_FIXTURES";

#[test]
fn rsc_transforms_fixture_outputs() {
  run_cases("fixture", false);
}

#[test]
fn rsc_transforms_error_outputs() {
  run_cases("errors", true);
}

fn run_cases(kind: &str, expect_error: bool) {
  let root = fixture_root().join(kind);
  let inputs = collect_inputs(&root);
  assert!(!inputs.is_empty(), "no {kind} rsc transform fixtures found");

  for input in inputs {
    run_case(&input, expect_error);
  }
}

fn run_case(input: &Path, expect_error: bool) {
  if expect_error {
    run_error_case(input);
    return;
  }

  run_fixture_case(input);
}

fn run_error_case(input: &Path) {
  let rsc_meta = RefCell::new(None);
  let output = expected_output_path(input);

  test_fixture(
    syntax_for_input(input),
    &|tr| build_rsc_transform(input, tr.comments.clone(), &rsc_meta),
    input,
    &output,
    FixtureTestConfig {
      allow_error: true,
      module: Some(true),
      ..Default::default()
    },
  );
}

fn run_fixture_case(input: &Path) {
  let (code, meta, error) = transform_fixture(input);
  let output = expected_output_path(input);
  let meta_output = input
    .parent()
    .expect("input has parent")
    .join("output.meta.txt");

  if let Some(error) = error {
    panic!("unexpected error for {}:\n{error}", input.display());
  }

  compare_or_update(&output, &code);
  compare_or_update(&meta_output, &format_meta(meta.as_ref()));
}

fn transform_fixture(input: &Path) -> (String, Option<RscMeta>, Option<String>) {
  let source = fs::read_to_string(input)
    .unwrap_or_else(|error| panic!("failed to read {}: {error}", input.display()));
  let compiler = JavaScriptCompiler::new();
  let comments = Rc::new(SingleThreadedComments::default());
  let rsc_meta = RefCell::new(None);
  let mut options = SwcOptions::default();
  options.config.jsc.syntax = Some(syntax_for_input(input));
  options.config.jsc.target = Some(EsVersion::Es2022);
  options.config.is_module = Some(IsModule::Bool(true));
  options.config.source_maps = Some(SourceMapsConfig::Bool(false));

  let source_filename = Arc::new(FileName::Real(input.to_path_buf()));
  let result = compiler.transform(
    source,
    Some(source_filename),
    comments.clone(),
    options,
    None,
    |_, _| {},
    |_| (build_rsc_transform(input, comments, &rsc_meta), noop_pass()),
  );

  match result {
    Ok(output) => (output.code, rsc_meta.into_inner(), None),
    Err(error) => (
      String::new(),
      rsc_meta.into_inner(),
      Some(error.to_string()),
    ),
  }
}

fn build_rsc_transform<'a>(
  input: &Path,
  comments: Rc<SingleThreadedComments>,
  rsc_meta: &'a RefCell<Option<RscMeta>>,
) -> impl Pass + 'a {
  let (resource_path, module_resource, pass_filename) = resource_for_input(input);
  let is_react_server_layer = input
    .iter()
    .any(|segment| segment.to_str() == Some("server-graph"));
  let is_development = input
    .iter()
    .any(|segment| segment.to_str() == Some("development"));

  rsc_transform(
    pass_filename,
    resource_path,
    module_resource,
    comments,
    rsc_meta,
    RscTransformOptions {
      is_react_server_layer,
      enable_server_entry: true,
      disable_client_api_checks: false,
      is_development,
      hash_salt: String::new(),
    },
  )
}

fn resource_for_input(input: &Path) -> (String, String, Arc<FileName>) {
  let resource = if input
    .iter()
    .any(|segment| segment.to_str() == Some("server-actions"))
  {
    "/app/item.js"
  } else {
    "/some-project/src/some-file.js"
  };

  (
    resource.to_string(),
    resource.to_string(),
    Arc::new(FileName::Real(PathBuf::from(resource))),
  )
}

fn syntax_for_input(input: &Path) -> Syntax {
  match input.extension().and_then(|extension| extension.to_str()) {
    Some("ts") => Syntax::Typescript(TsSyntax {
      tsx: false,
      ..Default::default()
    }),
    Some("tsx") => Syntax::Typescript(TsSyntax {
      tsx: true,
      ..Default::default()
    }),
    _ => Syntax::Es(EsSyntax {
      jsx: true,
      import_attributes: true,
      ..Default::default()
    }),
  }
}

fn expected_output_path(input: &Path) -> PathBuf {
  let extension = input
    .extension()
    .and_then(|extension| extension.to_str())
    .unwrap_or("js");
  input
    .parent()
    .expect("input has parent")
    .join(format!("output.{extension}"))
}

fn compare_or_update(expected_path: &Path, actual: &str) {
  let actual = normalize_newlines(actual);

  if should_update() {
    fs::write(expected_path, actual.as_bytes())
      .unwrap_or_else(|error| panic!("failed to write {}: {error}", expected_path.display()));
    return;
  }

  let expected = fs::read_to_string(expected_path).unwrap_or_else(|error| {
    panic!(
      "failed to read {}: {error}\nrun with {UPDATE_ENV}=1 to create expected output",
      expected_path.display()
    )
  });
  assert_eq!(
    normalize_newlines(&expected).as_ref(),
    actual.as_ref(),
    "fixture output mismatch for {}",
    expected_path.display()
  );
}

fn should_update() -> bool {
  env::var_os(UPDATE_ENV).is_some_and(|value| value != "0")
    || env::var_os("UPDATE").is_some_and(|value| value != "0")
}

fn normalize_newlines(value: &str) -> Cow<'_, str> {
  let Some(mut index) = value.find("\r\n") else {
    return Cow::Borrowed(value);
  };

  let mut normalized = String::with_capacity(value.len());
  let mut rest = value;
  loop {
    normalized.push_str(&rest[..index]);
    normalized.push('\n');
    rest = &rest[index + 2..];
    let Some(next_index) = rest.find("\r\n") else {
      normalized.push_str(rest);
      return Cow::Owned(normalized);
    };
    index = next_index;
  }
}

fn format_meta(meta: Option<&RscMeta>) -> String {
  let Some(meta) = meta else {
    return "None\n".to_string();
  };

  let action_ids = meta
    .action_ids
    .iter()
    .map(|(id, name)| format!("{id}:{name}"))
    .collect::<Vec<_>>()
    .join(",");

  format!(
    "module_type: {}\nserver_refs: [{}]\nclient_refs: [{}]\nimport_meta_rsc: {}\nis_cjs: {}\naction_ids: [{}]\n",
    module_type(meta.module_type),
    format_atoms(&meta.server_refs),
    format_atoms(&meta.client_refs),
    meta.import_meta_rsc,
    meta.is_cjs,
    action_ids
  )
}

fn module_type(module_type: RscModuleType) -> &'static str {
  match module_type {
    RscModuleType::ServerEntry => "ServerEntry",
    RscModuleType::Server => "Server",
    RscModuleType::Client => "Client",
  }
}

fn format_atoms(values: &[swc_core::atoms::Wtf8Atom]) -> String {
  values
    .iter()
    .filter_map(|value| value.as_str())
    .collect::<Vec<_>>()
    .join(",")
}

fn collect_inputs(root: &Path) -> Vec<PathBuf> {
  let mut inputs = vec![];
  collect_inputs_in(root, &mut inputs);
  inputs.sort();
  inputs
}

fn collect_inputs_in(path: &Path, inputs: &mut Vec<PathBuf>) {
  let entries = fs::read_dir(path)
    .unwrap_or_else(|error| panic!("failed to read fixture dir {}: {error}", path.display()));

  for entry in entries {
    let entry = entry.expect("fixture dir entry should be readable");
    let path = entry.path();
    if path.is_dir() {
      collect_inputs_in(&path, inputs);
    } else if is_input_file(&path) {
      inputs.push(path);
    }
  }
}

fn is_input_file(path: &Path) -> bool {
  matches!(
    path.file_name().and_then(|file_name| file_name.to_str()),
    Some("input.js" | "input.ts" | "input.tsx" | "page.js" | "route.js")
  )
}

fn fixture_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/rsc_transforms/tests/fixture")
}
