use std::borrow::Cow;

use indexmap::IndexMap;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BoxDependency, BuildMetaExportsType, Dependency, DependencyId, DependencyType, ExportsArgument,
  GenerateContext, ImportPhase, Module, ModuleArgument, ModuleDependency, ModuleGraph,
  ModuleIdentifier, ModuleInitFragments, ParseContext, ParseResult, ParserAndGenerator,
  RuntimeGlobals, SourceType, StaticExportsDependency, StaticExportsSpec,
  rspack_sources::{BoxSource, RawStringSource, Source, SourceExt},
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_util::{itoa, json_stringify};
use swc_core::atoms::Atom;
use wasmparser::{Import, Parser, Payload};

use crate::dependency::WasmImportDependency;

#[cacheable]
#[derive(Debug)]
pub struct AsyncWasmParserAndGenerator;

pub(crate) static WASM_SOURCE_TYPE: &[SourceType; 2] = &[SourceType::Wasm, SourceType::JavaScript];

#[derive(Debug)]
struct DepModule<'a> {
  request: &'a str,
  import_var: String,
  deps: Vec<(DependencyId, Atom)>,
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl ParserAndGenerator for AsyncWasmParserAndGenerator {
  fn source_types(&self, _module: &dyn Module, _module_graph: &ModuleGraph) -> &[SourceType] {
    WASM_SOURCE_TYPE
  }

  async fn parse<'a>(
    &mut self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    parse_context.build_info.strict = true;
    parse_context.build_meta.has_top_level_await = true;
    parse_context.build_meta.exports_type = BuildMetaExportsType::Namespace;

    let source = parse_context.source;

    let mut exports = Vec::with_capacity(1);
    let mut dependencies: Vec<BoxDependency> = Vec::with_capacity(1);
    let mut diagnostic = Vec::with_capacity(1);

    for payload in Parser::new(0).parse_all(&source.buffer()) {
      match payload {
        Ok(payload) => match payload {
          Payload::ExportSection(s) => {
            for export in s {
              match export {
                Ok(export) => exports.push(export.name.to_string()),
                Err(err) => diagnostic.push(Diagnostic::error(
                  "Wasm Export Parse Error".into(),
                  err.to_string(),
                )),
              };
            }
          }
          Payload::ImportSection(s) => {
            for import in s {
              match import {
                Ok(Import { module, name, .. }) => {
                  dependencies.push(Box::new(WasmImportDependency::new(
                    module.into(),
                    name.into(),
                  )));
                }
                Err(err) => diagnostic.push(Diagnostic::error(
                  "Wasm Import Parse Error".into(),
                  err.to_string(),
                )),
              }
            }
          }
          _ => {}
        },
        Err(err) => {
          diagnostic.push(Diagnostic::error(
            "Wasm Parse Error".into(),
            err.to_string(),
          ));
        }
      }
    }

    dependencies.push(Box::new(StaticExportsDependency::new(
      StaticExportsSpec::Array(exports.iter().cloned().map(Atom::from).collect::<Vec<_>>()),
      false,
    )));

    Ok(
      ParseResult {
        dependencies,
        blocks: vec![],
        presentational_dependencies: vec![],
        code_generation_dependencies: vec![],
        source,
        side_effects_bailout: None,
      }
      .with_diagnostic(diagnostic),
    )
  }

  fn size(&self, module: &dyn Module, source_type: Option<&SourceType>) -> f64 {
    match source_type.unwrap_or(&SourceType::Wasm) {
      SourceType::JavaScript => {
        40.0
          + module
            .get_presentational_dependencies()
            .map_or(0.0, |i| i.len() as f64 * 10.0)
      }
      SourceType::Wasm => module.source().map_or(0, |source| source.size()) as f64,
      _ => 0.0,
    }
  }

  #[allow(clippy::unwrap_in_result)]
  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    let GenerateContext {
      compilation,
      runtime,
      runtime_template,
      ..
    } = generate_context;
    let hash = module
      .build_info()
      .hash
      .as_ref()
      .map(|hash| hash.rendered(16))
      .expect("should build info have hash");

    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let mut dep_modules = IndexMap::<ModuleIdentifier, DepModule>::new();
        let mut promises: Vec<String> = vec![];

        let module_graph = &compilation.get_module_graph();

        module
          .get_dependencies()
          .iter()
          .map(|id| module_graph.dependency_by_id(id))
          .filter(|dep| dep.dependency_type() == &DependencyType::WasmImport)
          .map(|dep| {
            (
              dep,
              module_graph.module_graph_module_by_dependency_id(dep.id()),
            )
          })
          .for_each(|(dep, mgm)| {
            if let Some(mgm) = mgm {
              let dep = dep
                .as_any()
                .downcast_ref::<WasmImportDependency>()
                .expect("should be wasm import dependency");
              let request = dep.request();
              let len = dep_modules.len();
              let dep_module = dep_modules.entry(mgm.module_identifier).or_insert_with(|| {
                let mut len_buffer = itoa::Buffer::new();
                let len_str = len_buffer.format(len);
                let import_var = format!("rspack_import_{len_str}");
                if ModuleGraph::is_async(
                  &compilation.async_modules_artifact,
                  &mgm.module_identifier,
                ) {
                  promises.push(import_var.clone());
                }
                DepModule {
                  request,
                  import_var,
                  deps: vec![],
                }
              });
              dep_module.deps.push((*dep.id(), dep.name().clone()));
            }
          });

        let (imports_code, imports_compat_code): (Vec<String>, Vec<String>) = dep_modules
          .iter()
          .map(|(_, dep_module)| {
            runtime_template.import_statement(
              module,
              compilation,
              &dep_module.deps[0].0,
              &dep_module.import_var,
              dep_module.request,
              ImportPhase::Evaluation,
              false,
            )
          })
          .unzip();
        let imports_code = imports_code.join("");
        let imports_compat_code = imports_compat_code.join("");

        let import_obj_request_items = dep_modules
          .into_values()
          .map(|dep_module| {
            let deps = dep_module
              .deps
              .into_iter()
              .map(|(dep_id, export_name)| {
                let export = runtime_template.export_from_import(
                  compilation,
                  &mut ModuleInitFragments::default(),
                  module.identifier(),
                  *runtime,
                  true,
                  dep_module.request,
                  &dep_module.import_var,
                  std::slice::from_ref(&export_name),
                  &dep_id,
                  false,
                  false,
                  Some(true),
                  ImportPhase::Evaluation,
                );
                let name = json_stringify(&export_name);
                format!("{name}: {export}")
              })
              .collect::<Vec<_>>()
              .join(",\n");

            format!("{}: {{\n{deps}\n}}", json_stringify(dep_module.request))
          })
          .collect::<Vec<_>>();

        let imports_obj = if !import_obj_request_items.is_empty() {
          Some(format!(
            ", {{\n{}\n}}",
            &import_obj_request_items.join(",\n")
          ))
        } else {
          None
        };

        let module_argument = runtime_template.render_module_argument(ModuleArgument::Module);
        let exports_argument = runtime_template.render_exports_argument(ExportsArgument::Exports);
        let instantiate_call = format!(
          r#"{}({exports_argument}, {}, "{}" {})"#,
          runtime_template.render_runtime_globals(&RuntimeGlobals::INSTANTIATE_WASM),
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_ID),
          &hash,
          imports_obj.unwrap_or_default()
        );

        let source = if !promises.is_empty() {
          let promises = promises.join(", ");
          let decl = format!(
            "var __rspack_instantiate__ = function ([{promises}]) {{\n{imports_compat_code}return {instantiate_call};\n}}\n",
          );
          let async_dependencies = format!(
"{}({module_argument}, async function (__rspack_load_async_deps, __rspack_async_done) {{
  try {{
{imports_code}
    var __rspack_async_deps = __rspack_load_async_deps([{promises}]);
    var [{promises}] = __rspack_async_deps.then ? (await __rspack_async_deps)() : __rspack_async_deps;
    {imports_compat_code}await {instantiate_call};

  __rspack_async_done();

  }} catch(e) {{ __rspack_async_done(e); }}
}}, 1);
",
            runtime_template.render_runtime_globals(&RuntimeGlobals::ASYNC_MODULE),
          );

          RawStringSource::from(format!("{decl}{async_dependencies}"))
        } else {
          RawStringSource::from(format!(
            "{imports_code}{imports_compat_code}{module_argument}.exports = {instantiate_call};"
          ))
        };

        Ok(source.boxed())
      }
      _ => Ok(source.clone()),
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    _module: &dyn Module,
    _mg: &rspack_core::ModuleGraph,
    _cg: &rspack_core::ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    Some("Module Concatenation is not implemented for AsyncWasmParserAndGenerator".into())
  }
}
