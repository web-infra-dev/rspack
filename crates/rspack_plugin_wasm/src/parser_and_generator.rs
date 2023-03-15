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
use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
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

    let source = parse_context.source;

    let mut exports = Vec::with_capacity(1);
    let mut dependencies = Vec::with_capacity(1);

    let buf = source.buffer();
    for payload in Parser::new(0).parse_all(&*buf) {
      match payload.expect("TODO:") {
        Payload::ExportSection(s) => {
          for export in s {
            let export = export.expect("expected a correctly wasm export");

            exports.push(export.name.to_string());
          }
        }
        Payload::ImportSection(s) => {
          for import in s {
            let Import { module, name, ty } = import.expect("expected a correctly wasm import");

            let dep = box WasmImportDependency::new(module.into(), name.into(), ty)
              as Box<dyn ModuleDependency>;

            dependencies.push(dep);
          }
        }
        _ => {}
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
      .with_empty_diagnostic(),
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
      hash.clone(),
    );
    self
      .module_id_to_filename
      .insert(module.identifier(), wasm_filename.clone());

    match generate_context.requested_source_type {
      SourceType::JavaScript => {
        generate_context
          .data
          .insert("filename".into(), wasm_filename);

        let runtime_requirements = &mut generate_context.runtime_requirements;

        runtime_requirements.insert(runtime_globals::MODULE);
        runtime_requirements.insert(runtime_globals::MODULE_ID);
        runtime_requirements.insert(runtime_globals::INSTANTIATE_WASM);
        let dep_modules = DashMap::<ModuleIdentifier, String>::new();

        if let Some(dependencies) = module.get_code_generation_dependencies() {
          let module_graph = &compilation.module_graph;

          dependencies
                        .iter()
                        .filter(|dep| dep.dependency_type() == &WasmImport && dep.id().is_some())
                        .map(|dep| {
                            (
                                module_graph.module_identifier_by_dependency_id(dep.id().unwrap()),
                                dep,
                            )
                        })
                        .for_each(|(id, dep)| {
                            if let Some(id) = id && !dep_modules.contains_key(id) {
                                let import_var = &format!("WEBPACK_IMPORTED_MODULE_${}", dep_modules.len() + 1);
                                runtime_requirements.insert(runtime_globals::REQUIRE);
                                dep_modules.insert(*id, render_import_stmt(import_var, id));
                            }
                        })
        }

        let imports_code = dep_modules
          .into_iter()
          .map(|(_, s)| s)
          .collect::<Vec<_>>()
          .join("");

        let instantiate_call = format!(
          "{}(exports, module.id, {})",
          runtime_globals::INSTANTIATE_WASM,
          serde_json::to_string(&hash).expect("hash should be serializable")
        );

        Ok(GenerationResult {
          ast_or_source: RawSource::from(format!(
            "{imports_code} module.exports = {instantiate_call};"
          ))
          .boxed()
          .into(),
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

fn render_import_stmt(import_var: &str, module_id: &ModuleIdentifier) -> String {
  let module_id = &module_id.as_str();
  /// dynamic import
  //  [
  // importContent,
  // `/* harmony import */ ${optDeclaration}${importVar}_default = /*#__PURE__*/${RuntimeGlobals.compatGetDefaultExport}(${importVar});\n`
  // ];
  format!("/* harmony import */ var ${import_var} = __webpack_require__(${module_id});\n",)
}

fn hash_for_ast_or_source(ast_or_source: &AstOrSource) -> String {
  let mut hasher = DefaultHasher::new();
  ast_or_source.hash(&mut hasher);
  format!("{:x}", hasher.finish())
}
