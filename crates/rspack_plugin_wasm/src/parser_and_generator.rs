use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::Path;

use dashmap::DashMap;
use rspack_core::rspack_sources::{RawSource, Source, SourceExt};
use rspack_core::DependencyType::WasmImport;
use rspack_core::{
  runtime_globals, AstOrSource, Context, Dependency, Filename, FilenameRenderOptions,
  GenerateContext, GenerationResult, Module, ModuleDependency, ModuleIdentifier, NormalModule,
  ParseContext, ParseResult, ParserAndGenerator, SourceType, StaticExportsDependency,
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifier;
use rspack_plugin_asset::ModuleIdToFileName;
use sugar_path::SugarPath;
use wasmparser::{Import, Parser, Payload};

use crate::dependency::WasmImportDependency;

#[derive(Debug)]
pub struct AsyncWasmParserAndGenerator {
  pub(crate) module_id_to_filename: ModuleIdToFileName,
}

pub(crate) static WASM_SOURCE_TYPE: &[SourceType; 2] = &[SourceType::Wasm, SourceType::JavaScript];

impl ParserAndGenerator for AsyncWasmParserAndGenerator {
  fn source_types(&self) -> &[SourceType] {
    WASM_SOURCE_TYPE
  }

  fn parse(&mut self, parse_context: ParseContext) -> Result<TWithDiagnosticArray<ParseResult>> {
    parse_context.build_info.strict = true;
    parse_context.build_meta.is_async = true;

    let source = parse_context.source;

    let mut exports = Vec::with_capacity(1);
    let mut dependencies = Vec::with_capacity(1);
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
                  0,
                  0,
                )),
              };
            }
          }
          Payload::ImportSection(s) => {
            for import in s {
              match import {
                Ok(Import { module, name, ty }) => {
                  let dep = box WasmImportDependency::new(module.into(), name.into(), ty)
                    as Box<dyn ModuleDependency>;

                  dependencies.push(dep);
                }
                Err(err) => diagnostic.push(Diagnostic::error(
                  "Wasm Import Parse Error".into(),
                  err.to_string(),
                  0,
                  0,
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
            0,
            0,
          ));
        }
      }
    }

    dependencies
      .push(box StaticExportsDependency::new(exports, false) as Box<dyn ModuleDependency>);

    Ok(
      ParseResult {
        dependencies,
        presentational_dependencies: vec![],
        ast_or_source: source.into(),
      }
      .with_diagnostic(diagnostic),
    )
  }

  fn size(&self, module: &dyn Module, source_type: &SourceType) -> f64 {
    let base = module.size(source_type);
    match source_type {
      SourceType::JavaScript => 40.0 + base,
      SourceType::Wasm => base,
      _ => 0.0,
    }
  }

  #[allow(clippy::unwrap_in_result)]
  fn generate(
    &self,
    ast_or_source: &AstOrSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<GenerationResult> {
    let compilation = generate_context.compilation;
    let wasm_filename_template = &compilation.options.output.webassembly_module_filename;
    let hash = hash_for_ast_or_source(ast_or_source);
    let normal_module = module.as_normal_module();
    let wasm_filename = render_wasm_name(
      &compilation.options.context,
      normal_module,
      wasm_filename_template,
      hash,
    );

    self
      .module_id_to_filename
      .insert(module.identifier(), wasm_filename.clone());

    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        let runtime_requirements = &mut generate_context.runtime_requirements;
        runtime_requirements.insert(runtime_globals::MODULE);
        runtime_requirements.insert(runtime_globals::MODULE_ID);
        runtime_requirements.insert(runtime_globals::INSTANTIATE_WASM);

        let dep_modules = DashMap::<ModuleIdentifier, (String, &str)>::new();
        let wasm_deps_by_request = DashMap::<&str, Vec<(Identifier, String)>>::new();
        let mut promises: Vec<String> = vec![];

        let module_graph = &compilation.module_graph;
        let chunk_graph = &compilation.chunk_graph;

        if let Some(dependencies) = module_graph
          .module_graph_module_by_identifier(&module.identifier())
          .map(|mgm| &mgm.dependencies)
        {
          dependencies
            .iter()
            .map(|id| module_graph.dependency_by_id(id).expect("should be ok"))
            .filter(|dep| dep.dependency_type() == &WasmImport)
            .map(|dep| {
              (
                dep,
                module_graph.module_graph_module_by_dependency_id(&dep.id().expect("should be ok")),
              )
            })
            .for_each(|(dep, mgm)| {
              if let Some(mgm) = mgm {
                if !dep_modules.contains_key(&mgm.module_identifier) {
                  let import_var = format!("WEBPACK_IMPORTED_MODULE_{}", dep_modules.len());
                  let val = (import_var.clone(), mgm.id(chunk_graph));

                  if let Some(meta)=&mgm.build_meta&&meta.is_async{
                    promises.push(import_var);
                  }
                  dep_modules.insert(mgm.module_identifier, val);
                }

                let dep = dep
                  .as_any()
                  .downcast_ref::<WasmImportDependency>()
                  .expect("should be wasm import dependency");

                let dep_name = serde_json::to_string(dep.name()).expect("should be ok.");
                let request = dep.request();
                let val = (mgm.module_identifier, dep_name);
                if let Some(deps) = &mut wasm_deps_by_request.get_mut(&request) {
                  deps.value_mut().push(val);
                } else {
                  wasm_deps_by_request.insert(request, vec![val]);
                }
              }
            })
        }

        let imports_code = dep_modules
          .iter()
          .map(|val| render_import_stmt(&val.value().0, val.value().1))
          .collect::<Vec<_>>()
          .join("");

        let import_obj_request_items = wasm_deps_by_request
          .into_iter()
          .map(|(request, deps)| {
            let deps = deps
              .into_iter()
              .map(|(id, name)| {
                let import_var = dep_modules.get(&id).expect("should be ok");
                let import_var = &import_var.value().0;
                format!("{name}: {import_var}[{name}]")
              })
              .collect::<Vec<_>>()
              .join(",\n");

            format!(
              "{}: {{{deps}}}",
              serde_json::to_string(request).expect("should be ok")
            )
          })
          .collect::<Vec<_>>();

        let imports_obj = if !import_obj_request_items.is_empty() {
          Some(format!(", {{{}}}", &import_obj_request_items.join(",\n")))
        } else {
          None
        };

        let instantiate_call = format!(
          "{}(exports, module.id, {} {})",
          runtime_globals::INSTANTIATE_WASM,
          serde_json::to_string(&wasm_filename).expect("should be ok"),
          imports_obj.unwrap_or_default()
        );

        let source = if !promises.is_empty() {
          generate_context
            .runtime_requirements
            .insert(runtime_globals::ASYNC_MODULE);
          let promises = promises.join(", ");
          let decl = format!(
            "var __webpack_instantiate__=function([{promises}]){{ return {instantiate_call}}}\n",
          );
          let async_dependencies = format!(
            "{}(module, async function (__webpack_handle_async_dependencies__, __webpack_async_result__){{ 
                  try {{ 
                    {imports_code}
                    var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([{promises}]);
                    var [{promises}] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__;
                    await {instantiate_call};

                  __webpack_async_result__();

                  }} catch(e) {{ __webpack_async_result__(e); }}
                }}, 1);
          ",
            runtime_globals::ASYNC_MODULE,
          );

          RawSource::from(format!("{decl}{async_dependencies}"))
        } else {
          RawSource::from(format!(
            "{imports_code} module.exports = {instantiate_call};"
          ))
        };

        Ok(GenerationResult {
          ast_or_source: source.boxed().into(),
        })
      }
      _ => Ok(ast_or_source.clone().into()),
    }
  }
}

fn render_wasm_name(
  ctx: &Context,
  normal_module: Option<&NormalModule>,
  wasm_filename_template: &Filename,
  hash: String,
) -> String {
  wasm_filename_template.render(FilenameRenderOptions {
    name: normal_module.and_then(|m| {
      let p = Path::new(&m.resource_resolved_data().resource_path);
      p.file_stem().map(|s| s.to_string_lossy().to_string())
    }),
    path: normal_module.map(|m| {
      Path::new(&m.resource_resolved_data().resource_path)
        .relative(ctx)
        .to_string_lossy()
        .to_string()
    }),
    extension: normal_module.and_then(|m| {
      Path::new(&m.resource_resolved_data().resource_path)
        .extension()
        .and_then(OsStr::to_str)
        .map(|str| format!("{}{}", ".", str))
    }),
    contenthash: Some(hash.clone()),
    chunkhash: Some(hash.clone()),
    hash: Some(hash),
    ..Default::default()
  })
}

fn render_import_stmt(import_var: &str, module_id: &str) -> String {
  let module_id = serde_json::to_string(&module_id).expect("TODO");
  format!("var {import_var} = __webpack_require__({module_id});\n",)
}

fn hash_for_ast_or_source(ast_or_source: &AstOrSource) -> String {
  let mut hasher = DefaultHasher::new();
  ast_or_source.hash(&mut hasher);
  format!("{:x}", hasher.finish())
}
