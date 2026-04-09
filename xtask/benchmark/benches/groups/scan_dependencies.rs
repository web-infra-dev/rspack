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
  parser_and_generator::ParserRuntimeRequirementsData,
  visitors::scan_dependencies as run_scan_dependencies,
};
use rspack_tasks::within_compiler_context_for_testing_sync;
use rustc_hash::FxHashSet;
use swc_core::{
  base::config::IsModule,
  common::{BytePos, FileName, GLOBALS, Globals, Mark},
  ecma::{
    ast::EsVersion,
    parser::{EsSyntax, Syntax},
  },
};

struct ScanDependenciesCaseDefinition {
  name: &'static str,
  source: &'static str,
  resource: &'static str,
  module_type: ModuleType,
}

struct PreparedScanDependenciesCase {
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

pub fn scan_dependencies_benchmark(c: &mut Criterion) {
  within_compiler_context_for_testing_sync(|| {
    scan_dependencies_benchmark_inner(c);
  })
}

fn scan_dependencies_benchmark_inner(c: &mut Criterion) {
  GLOBALS.set(&Globals::new(), || {
    let compiler = create_benchmark_compiler();
    let cases = [prepare_case(
      &compiler,
      ScanDependenciesCaseDefinition {
        name: "rust@scan_dependencies@react_navbar",
        source: r#"import React from 'react';
import Icon from '@icon-park/react/es/all';

import Component__0 from './d0/f0.jsx';
import Component__1 from './d0/f1.jsx';
import Component__2 from './d0/f2.jsx';
import Component__3 from './d0/f3.jsx';
import Component__4 from './d0/f4.jsx';
import Component__5 from './d0/f5.jsx';
import Component__6 from './d0/f6.jsx';
import Component__7 from './d0/f7.jsx';
import Component__8 from './d0/f8.jsx';

function Navbar({ show }) {
  return (
    <div>
      9
      <Component__0 />
      <Component__1 />
      <Component__2 />
      <Component__3 />
      <Component__4 />
      <Component__5 />
      <Component__6 />
      <Component__7 />
      <Component__8 />
    </div>
  );
}

export default Navbar;"#,
        resource: "/src/index.jsx",
        module_type: ModuleType::JsEsm,
      },
    )];

    for case in &cases {
      bench_case(c, case);
    }
  });
}

fn bench_case(c: &mut Criterion, case: &PreparedScanDependenciesCase) {
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
        let result = run_scan_dependencies(
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
        .expect("scan_dependencies benchmark should not produce diagnostics");

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
  compiler: &Compiler,
  definition: ScanDependenciesCaseDefinition,
) -> PreparedScanDependenciesCase {
  let ScanDependenciesCaseDefinition {
    name,
    source,
    resource,
    module_type,
  } = definition;
  let (program, unresolved_mark) = parse_program(resource, source, &module_type);
  let parser_options = compiler
    .options
    .module
    .parser
    .as_ref()
    .and_then(|parser_map| parser_map.get("javascript"))
    .cloned()
    .expect("benchmark compiler should include javascript parser options");

  PreparedScanDependenciesCase {
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
      Syntax::Es(EsSyntax {
        jsx: resource.ends_with(".jsx"),
        allow_return_outside_function: matches!(
          module_type,
          ModuleType::JsDynamic | ModuleType::JsAuto
        ),
        explicit_resource_management: true,
        import_attributes: true,
        ..Default::default()
      }),
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

impl PreparedScanDependenciesCase {
  fn program_source(&self) -> &str {
    self.source
  }
}

criterion_group!(scan_dependencies, scan_dependencies_benchmark);
