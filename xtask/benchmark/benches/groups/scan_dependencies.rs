#![allow(clippy::unwrap_used)]

use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
  sync::Arc,
};

use criterion::{BatchSize, black_box};
use rspack::builder::{Builder as _, NodeOptionBuilder};
use rspack_benchmark::Criterion;
use rspack_core::{
  BuildInfo, BuildMeta, Compiler, CompilerOptions, Mode, ModuleCodeTemplate, ModuleIdentifier,
  ModuleType, Optimization, ParseMeta, ParserOptions, ResourceData, SideEffectOption,
};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin,
  parser_and_generator::ParserRuntimeRequirementsData,
  visitors::{
    ParsedJavaScriptAst, ScanDependenciesResult, scan_dependencies as run_scan_dependencies,
    semicolon::InsertedSemicolons,
  },
};
use rspack_tasks::within_compiler_context_for_testing_sync;
use rustc_hash::FxHashSet;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Comments, EsVersion, Program, VisitWith};
use swc_experimental_ecma_parser::{
  EsSyntax, Lexer, Parser, StringSource, Syntax, unstable::Capturing,
};
use swc_experimental_ecma_semantic::resolver::resolver;
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

const THREE_MODULE_BENCHMARK_ID: &str = "rust@scan_dependencies@three_module";
const THREE_MODULE_RESOURCE_PATH: &str = "/node_modules/three/build/three.module.js";
const THREE_MODULE_TARBALL_URL: &str = "https://registry.npmjs.org/three/-/three-0.183.2.tgz";
const THREE_MODULE_TAR_ENTRY: &str = "package/build/three.module.js";

struct ScanDependenciesBenchmarkCaseSpec {
  benchmark_id: &'static str,
  source_text: String,
  resource_path: &'static str,
  module_type: ModuleType,
}

struct PreparedScanDependenciesBenchmarkCase {
  benchmark_id: &'static str,
  source_text: String,
  compiler_options: Arc<CompilerOptions>,
  initial_semicolons: FxHashSet<u32>,
  parser_options: ParserOptions,
  parsed_ast: ParsedJavaScriptAst<'static>,
  module_identifier: ModuleIdentifier,
  module_type: ModuleType,
  resource_data: ResourceData,
  parser_runtime_requirements: ParserRuntimeRequirementsData,
}

struct PreparedScanDependenciesProgram {
  parsed_ast: ParsedJavaScriptAst<'static>,
  semicolons: FxHashSet<u32>,
}

#[derive(Default)]
struct ScanDependenciesIterationState {
  build_meta: BuildMeta,
  build_info: BuildInfo,
  semicolons: FxHashSet<u32>,
  parser_plugins: Vec<BoxJavascriptParserPlugin>,
  parse_meta: ParseMeta,
}

pub fn benchmark_scan_dependencies(c: &mut Criterion) {
  within_compiler_context_for_testing_sync(|| {
    register_scan_dependencies_benchmarks(c);
  })
}

fn register_scan_dependencies_benchmarks(c: &mut Criterion) {
  let compiler = create_scan_dependencies_compiler();
  let benchmark_cases = load_scan_dependencies_benchmark_specs()
    .into_iter()
    .map(|case_spec| prepare_scan_dependencies_benchmark_case(&compiler, case_spec))
    .collect::<Vec<_>>();

  for benchmark_case in &benchmark_cases {
    benchmark_case.assert_can_execute();
    register_scan_dependencies_benchmark_case(c, benchmark_case);
  }
}

fn register_scan_dependencies_benchmark_case(
  c: &mut Criterion,
  benchmark_case: &PreparedScanDependenciesBenchmarkCase,
) {
  c.bench_function(benchmark_case.benchmark_id, |b| {
    b.iter_batched_ref(
      || benchmark_case.build_iteration_state(),
      |iteration_state| {
        let result = benchmark_case.execute_scan_dependencies(iteration_state);
        black_box(result);
      },
      BatchSize::SmallInput,
    );
  });
}

fn create_scan_dependencies_compiler() -> Compiler {
  let mut optimization = Optimization::builder();
  optimization.inner_graph(true);
  optimization.side_effects(SideEffectOption::True);

  Compiler::builder()
    .context("/")
    .entry("main", "/src/index.js")
    .mode(Mode::Production)
    .optimization(optimization)
    .node(NodeOptionBuilder::default())
    .amd("amd".to_string())
    .build()
    .expect("scan_dependencies benchmark compiler should build")
}

fn load_scan_dependencies_benchmark_specs() -> Vec<ScanDependenciesBenchmarkCaseSpec> {
  vec![ScanDependenciesBenchmarkCaseSpec {
    benchmark_id: THREE_MODULE_BENCHMARK_ID,
    source_text: load_three_module_benchmark_source(),
    resource_path: THREE_MODULE_RESOURCE_PATH,
    module_type: ModuleType::JsEsm,
  }]
}

fn prepare_scan_dependencies_benchmark_case(
  compiler: &Compiler,
  case_spec: ScanDependenciesBenchmarkCaseSpec,
) -> PreparedScanDependenciesBenchmarkCase {
  let ScanDependenciesBenchmarkCaseSpec {
    benchmark_id,
    source_text,
    resource_path,
    module_type,
  } = case_spec;
  let PreparedScanDependenciesProgram {
    parsed_ast,
    semicolons,
  } = parse_benchmark_program(resource_path, &source_text, &module_type);
  let compiler_options = compiler.options.clone();
  let parser_options = compiler
    .options
    .module
    .parser
    .as_ref()
    .and_then(|parser_map| parser_map.get("javascript"))
    .cloned()
    .expect("scan_dependencies benchmark compiler should include javascript parser options");

  PreparedScanDependenciesBenchmarkCase {
    benchmark_id,
    source_text,
    compiler_options: compiler_options.clone(),
    initial_semicolons: semicolons,
    parser_options,
    parsed_ast,
    module_identifier: resource_path.into(),
    module_type,
    resource_data: ResourceData::new_with_resource(resource_path.to_string()),
    parser_runtime_requirements: ParserRuntimeRequirementsData::new(&ModuleCodeTemplate::new(
      compiler_options,
    )),
  }
}

fn parse_benchmark_program(
  resource_path: &str,
  source_text: &str,
  module_type: &ModuleType,
) -> PreparedScanDependenciesProgram {
  let source_text = Box::leak(source_text.to_string().into_boxed_str());
  let allocator = Box::leak(Box::new(Allocator::default()));
  let comments = Box::leak(Box::new(Comments::default()));
  let parser_lexer = Lexer::new(
    allocator,
    Syntax::Es(EsSyntax {
      jsx: resource_path.ends_with(".jsx"),
      allow_return_outside_function: matches!(
        module_type,
        ModuleType::JsDynamic | ModuleType::JsAuto
      ),
      explicit_resource_management: true,
      import_attributes: true,
      ..Default::default()
    }),
    EsVersion::EsNext,
    StringSource::new(source_text),
    Some(comments),
  );
  let parser_lexer = Capturing::new(parser_lexer);
  let mut parser = Parser::new_from(allocator, parser_lexer);
  let mut program = match module_type {
    ModuleType::JsEsm => parser
      .parse_module()
      .map(|module| Program::Module(allocator.boxed(module))),
    ModuleType::JsDynamic => parser
      .parse_commonjs()
      .map(|script| Program::Script(allocator.boxed(script))),
    _ => parser.parse_program(),
  }
  .expect("scan_dependencies benchmark source should parse");
  let parse_errors = parser.take_errors();
  assert!(
    parse_errors.is_empty(),
    "scan_dependencies benchmark source should parse without recoverable errors"
  );
  let tokens = parser.input().iter.tokens().to_vec();
  drop(parser);

  let mut semicolons = FxHashSet::default();
  remove_paren(&mut program, allocator, Some(comments));
  let program = Box::leak(Box::new(program));
  let semantic = Box::leak(Box::new(resolver(program)));
  program.visit_with(&mut InsertedSemicolons::new(&mut semicolons, &tokens));

  PreparedScanDependenciesProgram {
    parsed_ast: ParsedJavaScriptAst {
      allocator,
      comments,
      semantic,
      program,
    },
    semicolons,
  }
}

impl PreparedScanDependenciesBenchmarkCase {
  fn build_iteration_state(&self) -> ScanDependenciesIterationState {
    ScanDependenciesIterationState {
      semicolons: self.initial_semicolons.clone(),
      ..Default::default()
    }
  }

  fn execute_scan_dependencies(
    &self,
    iteration_state: &mut ScanDependenciesIterationState,
  ) -> ScanDependenciesResult {
    run_scan_dependencies(
      &self.source_text,
      &self.parsed_ast,
      &self.resource_data,
      self.compiler_options.as_ref(),
      &self.module_type,
      None,
      None,
      &mut iteration_state.build_meta,
      &mut iteration_state.build_info,
      self.module_identifier,
      Some(&self.parser_options),
      &mut iteration_state.semicolons,
      &mut iteration_state.parser_plugins,
      std::mem::take(&mut iteration_state.parse_meta),
      &self.parser_runtime_requirements,
    )
    .unwrap_or_else(|diagnostics| {
      panic!(
        "{} should execute without scan errors. Diagnostics: {diagnostics:#?}",
        self.benchmark_id
      )
    })
  }

  fn assert_can_execute(&self) {
    let mut iteration_state = self.build_iteration_state();
    let _ = self.execute_scan_dependencies(&mut iteration_state);
  }
}

fn load_three_module_benchmark_source() -> String {
  let cache_path = three_module_source_cache_path();
  if !cache_path.exists() {
    cache_three_module_benchmark_source(&cache_path);
  }

  fs::read_to_string(&cache_path).unwrap_or_else(|err| {
    panic!(
      "failed to read cached three.module.js benchmark source from {}: {err}",
      cache_path.display()
    )
  })
}

fn cache_three_module_benchmark_source(cache_path: &Path) {
  ensure_parent_directory_exists(cache_path);

  let tarball_path = cache_path.with_extension("tgz");
  let temporary_source_path = cache_path.with_extension("tmp");

  download_file(THREE_MODULE_TARBALL_URL, &tarball_path);
  extract_tarball_entry_to_file(
    &tarball_path,
    THREE_MODULE_TAR_ENTRY,
    &temporary_source_path,
  );
  fs::rename(&temporary_source_path, cache_path).unwrap_or_else(|err| {
    panic!(
      "failed to move extracted three.module.js benchmark source into {}: {err}",
      cache_path.display()
    )
  });
  let _ = fs::remove_file(&tarball_path);
}

fn ensure_parent_directory_exists(path: &Path) {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).unwrap_or_else(|err| {
      panic!(
        "failed to create benchmark cache directory {}: {err}",
        parent.display()
      )
    });
  }
}

fn three_module_source_cache_path() -> PathBuf {
  benchmark_target_dir()
    .join("rspack_benchmark")
    .join("scan_dependencies")
    .join("three_module.js")
}

fn benchmark_target_dir() -> PathBuf {
  std::env::var_os("CARGO_TARGET_DIR")
    .map_or_else(|| workspace_root().join("target"), PathBuf::from)
}

fn workspace_root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .ancestors()
    .nth(2)
    .expect("xtask/benchmark should live under the workspace root")
    .to_path_buf()
}

fn download_file(url: &str, destination: &Path) {
  let output = Command::new("curl")
    .args(["-fsSL", "-o"])
    .arg(destination)
    .arg(url)
    .output()
    .unwrap_or_else(|err| panic!("failed to spawn curl while downloading {url}: {err}"));

  if !output.status.success() {
    panic!(
      "failed to download {url} into {}: {}",
      destination.display(),
      String::from_utf8_lossy(&output.stderr)
    );
  }
}

fn extract_tarball_entry_to_file(archive_path: &Path, tar_entry: &str, destination: &Path) {
  let output = Command::new("tar")
    .args(["-xzOf"])
    .arg(archive_path)
    .arg(tar_entry)
    .output()
    .unwrap_or_else(|err| {
      panic!(
        "failed to spawn tar while extracting {tar_entry} from {}: {err}",
        archive_path.display()
      )
    });

  if !output.status.success() {
    panic!(
      "failed to extract {tar_entry} from {}: {}",
      archive_path.display(),
      String::from_utf8_lossy(&output.stderr)
    );
  }

  fs::write(destination, output.stdout).unwrap_or_else(|err| {
    panic!(
      "failed to write extracted benchmark source to {}: {err}",
      destination.display()
    )
  });
}
