#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use criterion::{BatchSize, black_box, criterion_group};
use rspack::builder::{Builder as _, NodeOptionBuilder};
use rspack_benchmark::Criterion;
use rspack_core::{
  BuildInfo, BuildMeta, Compiler, CompilerOptions, Mode, ModuleCodeTemplate, ModuleIdentifier,
  ModuleType, Optimization, ParseMeta, ParserOptions, ResourceData, SideEffectOption,
};
use rspack_javascript_compiler::{JavaScriptCompiler, ast::Program};
use rspack_plugin_javascript::{
  parser_and_generator::ParserRuntimeRequirementsData, visitors::scan_dependencies,
};
use rspack_tasks::within_compiler_context_for_testing_sync;
use rustc_hash::FxHashSet;
use swc_core::{
  base::config::IsModule,
  common::{BytePos, FileName, GLOBALS, Globals, Mark},
  ecma::{ast::EsVersion, parser::Syntax},
};

struct PreparedParserCase {
  name: &'static str,
  source: &'static str,
  compiler_options: Arc<CompilerOptions>,
  parser_options: ParserOptions,
  program: Program,
  module_identifier: ModuleIdentifier,
  module_type: ModuleType,
  resource_data: ResourceData,
  unresolved_mark: Mark,
  parser_runtime_requirements: ParserRuntimeRequirementsData,
}

pub fn javascript_parser_benchmark(c: &mut Criterion) {
  within_compiler_context_for_testing_sync(|| {
    javascript_parser_benchmark_inner(c);
  })
}

fn javascript_parser_benchmark_inner(c: &mut Criterion) {
  GLOBALS.set(&Globals::new(), || {
    let compiler = create_benchmark_compiler();
    let esm_case = prepare_case(
      "rust@javascript_parser_esm",
      &compiler,
      ModuleType::JsEsm,
      "/src/esm.js",
      r#"
        import value, { named as alias } from "./dep.js";
        export { alias };
        export * from "./other.js";
        const metaUrl = import.meta.url;
        const asset = new URL("./asset.svg", import.meta.url);
        const lazy = import("./async.js");
        export default value ?? asset.href ?? metaUrl;
      "#,
    );
    let cjs_case = prepare_case(
      "rust@javascript_parser_cjs",
      &compiler,
      ModuleType::JsDynamic,
      "/src/cjs.js",
      r#"
        const fs = require("fs");
        const ctx = require.context("./dir", false, /\.js$/);
        const ensured = require.ensure(["./async.js"], function () {
          return require("./other.js");
        });
        module.exports = fs ? ctx : ensured;
        exports.named = require("./named.js");
        require.resolve("./maybe.js");
      "#,
    );
    let mixed_case = prepare_case(
      "rust@javascript_parser_mixed",
      &compiler,
      ModuleType::JsAuto,
      "/src/mixed.js",
      r#"
        import { value } from "./esm.js";
        export { value };
        const runtime = require("./runtime.js");
        const asset = new URL("./asset.txt", import.meta.url);
        const worker = new Worker(new URL("./worker.js", import.meta.url));
        if (typeof require !== "undefined") {
          module.exports = runtime;
        }
        class Derived extends runtime.Base {
          field = import("./async.js");
        }
        export const finalValue = /*#__PURE__*/ runtime.make(asset, worker);
      "#,
    );

    for case in [&esm_case, &cjs_case, &mixed_case] {
      bench_case(c, case);
    }
  });
}

fn bench_case(c: &mut Criterion, case: &PreparedParserCase) {
  c.bench_function(case.name, |b| {
    b.iter_batched_ref(
      || {
        (
          BuildMeta::default(),
          BuildInfo::default(),
          FxHashSet::<BytePos>::default(),
          Vec::new(),
          ParseMeta::default(),
        )
      },
      |(build_meta, build_info, semicolons, parser_plugins, parse_meta)| {
        let result = scan_dependencies(
          case.program_source(),
          &case.program,
          &case.resource_data,
          case.compiler_options.as_ref(),
          &case.module_type,
          None,
          None,
          build_meta,
          build_info,
          case.module_identifier,
          Some(&case.parser_options),
          semicolons,
          case.unresolved_mark,
          parser_plugins,
          std::mem::take(parse_meta),
          &case.parser_runtime_requirements,
        )
        .expect("javascript parser benchmark should not produce diagnostics");

        black_box(result);
      },
      BatchSize::SmallInput,
    );
  });
}

fn create_benchmark_compiler() -> Compiler {
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
    .expect("benchmark compiler should build")
}

fn prepare_case(
  name: &'static str,
  compiler: &Compiler,
  module_type: ModuleType,
  resource: &str,
  source: &'static str,
) -> PreparedParserCase {
  let (program, unresolved_mark) = parse_program(resource, source, &module_type);
  let parser_options = compiler
    .options
    .module
    .parser
    .as_ref()
    .and_then(|parser_map| parser_map.get("javascript"))
    .cloned()
    .expect("benchmark compiler should include javascript parser options");

  PreparedParserCase {
    name,
    source,
    compiler_options: compiler.options.clone(),
    parser_options,
    program,
    module_identifier: resource.into(),
    module_type,
    resource_data: ResourceData::new_with_resource(resource.to_string()),
    unresolved_mark,
    parser_runtime_requirements: ParserRuntimeRequirementsData::new(&ModuleCodeTemplate::new(
      compiler.options.clone(),
    )),
  }
}

fn parse_program(resource: &str, source: &str, module_type: &ModuleType) -> (Program, Mark) {
  let compiler = JavaScriptCompiler::new();
  let ast = compiler
    .parse(
      FileName::Real(resource.into()),
      source,
      EsVersion::latest(),
      Syntax::Es(Default::default()),
      module_type_to_is_module(module_type),
      None,
    )
    .expect("benchmark source should parse");

  ast.visit(|program, context| (program.clone(), context.unresolved_mark))
}

fn module_type_to_is_module(module_type: &ModuleType) -> IsModule {
  match module_type {
    ModuleType::JsEsm => IsModule::Bool(true),
    ModuleType::JsDynamic => IsModule::Bool(false),
    _ => IsModule::Unknown,
  }
}

impl PreparedParserCase {
  fn program_source(&self) -> &str {
    self.source
  }
}

criterion_group!(javascript_parser, javascript_parser_benchmark);
