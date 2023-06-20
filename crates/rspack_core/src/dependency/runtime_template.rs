use swc_core::ecma::atoms::JsWord;

use crate::{
  to_identifier, CodeReplaceSourceDependencyContext, Compilation, DependencyId, ExportsType,
  InitFragment, InitFragmentStage, ModuleGraph, ModuleIdentifier, RuntimeGlobals,
};

pub fn export_from_import(
  code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  default_interop: bool,
  import_var: String,
  mut export_name: Vec<JsWord>,
  id: &DependencyId,
  is_call: bool,
) -> String {
  let CodeReplaceSourceDependencyContext {
    runtime_requirements,
    compilation,
    init_fragments,
    module,
    ..
  } = code_generatable_context;

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());

  if default_interop {
    if !export_name.is_empty()
        && let Some(first_export_name) = export_name.get(0) && first_export_name == "default"
      {
        match exports_type {
            ExportsType::Dynamic => {
              return format!("{import_var}_default{}", property_access(&export_name, 1));
            }
            ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
              export_name = export_name[1..].to_vec();
            }
            _ => {}
        }
      } else if !export_name.is_empty() {
        if matches!(exports_type, ExportsType::DefaultOnly) {
          return format!("/* non-default import from non-esm module */undefined\n{}", property_access(&export_name, 1));
        } else if !matches!(exports_type, ExportsType::Namespace)  && let Some(first_export_name) = export_name.get(0) && first_export_name == "__esModule" {
          return "/* __esModule */true".to_string();
        }
      } else if matches!(exports_type, ExportsType::DefaultOnly | ExportsType::DefaultWithNamed) {
        runtime_requirements.add(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
        init_fragments.push(InitFragment::new(
          format!(
            "var {import_var}_namespace_cache;\n",
          ),
          InitFragmentStage::STAGE_HARMONY_EXPORTS,
          None,
        ));
        return format!("/*#__PURE__*/ ({import_var}_namespace_cache || ({import_var}_namespace_cache = {}({import_var}{})))", RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT, if matches!(exports_type, ExportsType::DefaultOnly) { "" } else { ", 2" });
      }
  }

  if !export_name.is_empty() {
    // TODO check used
    let access = format!("{import_var}{}", property_access(&export_name, 0));
    if is_call {
      return format!("(0, {access})");
    }
    access
  } else {
    import_var
  }
}

pub fn get_exports_type(
  module_graph: &ModuleGraph,
  id: &DependencyId,
  parent_module: &ModuleIdentifier,
) -> ExportsType {
  let module = module_graph
    .module_identifier_by_dependency_id(id)
    .expect("should have module");
  let strict = module_graph
    .module_graph_module_by_identifier(parent_module)
    .expect("should have mgm")
    .get_strict_harmony_module();
  module_graph
    .module_graph_module_by_identifier(module)
    .expect("should have mgm")
    .get_exports_type(strict)
}

fn property_access(o: &Vec<JsWord>, mut start: usize) -> String {
  let mut str = String::default();
  while start < o.len() {
    let property = &o[start];
    str.push_str(format!(r#"["{property}"]"#).as_str());
    start += 1;
  }
  str
}

pub fn get_import_var(user_request: &str) -> String {
  // avoid './a' and '../a' generate different identifier
  let request = user_request.replace("..", "$");
  format!("{}__WEBPACK_IMPORTED_MODULE__", to_identifier(&request))
}

pub fn module_id_expr(request: &str, module_id: &str) -> String {
  format!(
    "/* {} */{}",
    request,
    serde_json::to_string(module_id).expect("should render module id")
  )
}

pub fn module_id(
  compilation: &Compilation,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation.module_graph.module_identifier_by_dependency_id(id)
        && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    module_id_expr(request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    unreachable!("runtime template no module id")
  }
}

pub fn import_statement(
  code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  id: &DependencyId,
  request: &str,
  update: bool, // whether a new variable should be created or the existing one updated
) -> (String, String) {
  let CodeReplaceSourceDependencyContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;

  let module_id = compilation
    .module_graph
    .module_graph_module_by_dependency_id(id)
    .map(|m| m.id(&compilation.chunk_graph))
    .expect("should have dependency id");

  runtime_requirements.add(RuntimeGlobals::REQUIRE);

  let import_var = get_import_var(request);

  let opt_declaration = if update { "" } else { "var " };

  let import_content = format!(
    "/* harmony import */{opt_declaration}{import_var} = {}({});\n",
    RuntimeGlobals::REQUIRE,
    module_id_expr(request, module_id)
  );

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());
  if matches!(exports_type, ExportsType::Dynamic) {
    runtime_requirements.add(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    return (
      import_content,
      format!(
        "/* harmony import */{opt_declaration}{import_var}_default = /*#__PURE__*/{}({import_var});\n",
        RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
      ),
    );
  }
  (import_content, "".to_string())
}
