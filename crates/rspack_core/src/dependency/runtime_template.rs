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
    miss_module(request)
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

  let module_id_expr = module_id(&compilation, id, request, false);

  runtime_requirements.add(RuntimeGlobals::REQUIRE);

  let import_var = get_import_var(request);

  let opt_declaration = if update { "" } else { "var " };

  let import_content = format!(
    "/* harmony import */{opt_declaration}{import_var} = {}({module_id_expr});\n",
    RuntimeGlobals::REQUIRE
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

pub fn module_namespace_promise(
  code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  let CodeReplaceSourceDependencyContext {
    runtime_requirements,
    compilation,
    module,
    ..
  } = code_generatable_context;

  let module_id_expr = module_id(&compilation, id, request, weak);

  let exports_type = get_exports_type(&compilation.module_graph, id, &module.identifier());

  let header = if weak {
    runtime_requirements.add(RuntimeGlobals::MODULE_FACTORIES);
    Some(format!(
      "if(!{}[{module_id_expr}]) {{\n {} \n}}",
      RuntimeGlobals::MODULE_FACTORIES,
      weak_error(request)
    ))
  } else {
    None
  };
  let mut fake_type = 16;
  let mut appending;
  match exports_type {
    ExportsType::Namespace => {
      if let Some(header) = header {
        appending = format!(
          ".then(function() {{ {header}\nreturn {}}})",
          module_raw(compilation, runtime_requirements, id, request, weak)
        )
      } else {
        runtime_requirements.add(RuntimeGlobals::REQUIRE);
        appending =
          format!(".then(__webpack_require__.bind(__webpack_require__, {module_id_expr}))");
      }
    }
    _ => {
      if matches!(exports_type, ExportsType::Dynamic) {
        fake_type |= 4;
      }
      if matches!(exports_type, ExportsType::DefaultWithNamed) {
        fake_type |= 2;
      }
      runtime_requirements.add(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
      if compilation.module_graph.is_async(&module.identifier()) {
        if let Some(header) = header {
          appending = format!(
            ".then(function() {{\n {header}\nreturn {}\n}})",
            module_raw(compilation, runtime_requirements, id, request, weak)
          )
        } else {
          runtime_requirements.add(RuntimeGlobals::REQUIRE);
          appending =
            format!(".then(__webpack_require__.bind(__webpack_require__, {module_id_expr}))");
        }
        appending.push_str(
          format!(
            ".then(function(m){{\n {}(m, {fake_type}) \n}})",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          )
          .as_str(),
        );
      } else {
        fake_type |= 1;
        if let Some(header) = header {
          let expr = format!(
            "{}({module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          );
          appending = format!(".then(function() {{\n {header} return {expr};\n}})");
        } else {
          appending = format!(
            ".then({}.bind(__webpack_require__, {module_id_expr}, {fake_type}))",
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
          );
        }
      }
    }
  }

  format!(
    "{}{appending}",
    block_promise(&module_id_expr, runtime_requirements)
  )
}

pub fn block_promise(module_id_str: &str, runtime_requirements: &mut RuntimeGlobals) -> String {
  runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
  runtime_requirements.insert(RuntimeGlobals::LOAD_CHUNK_WITH_MODULE);
  format!(
    "{}({module_id_str})",
    RuntimeGlobals::LOAD_CHUNK_WITH_MODULE
  )
}

pub fn module_raw(
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation.module_graph.module_identifier_by_dependency_id(id)
        && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier)
  {
    runtime_requirements.add(RuntimeGlobals::REQUIRE);
     format!(
      "{}({})",
      RuntimeGlobals::REQUIRE,
      module_id_expr(request, module_id)
    )
  } else if weak {
    weak_error(request)
  } else {
    miss_module(request)
  }
}

pub fn miss_module(request: &str) -> String {
  format!("Object({}())", throw_missing_module_error_function(request))
}

pub fn throw_missing_module_error_function(request: &str) -> String {
  format!(
    "function webpackMissingModule() {{ {} }}",
    throw_missing_module_error_block(request)
  )
}

pub fn throw_missing_module_error_block(request: &str) -> String {
  format!(
    "var e = new Error('Cannot find module '{request}''); e.code = 'MODULE_NOT_FOUND'; throw e;"
  )
}

pub fn weak_error(request: &str) -> String {
  let msg = format!("Module is not available (weak dependency), request is {request}.");
  format!("var e = new Error('{msg}'); e.code = 'MODULE_NOT_FOUND'; throw e;")
}
