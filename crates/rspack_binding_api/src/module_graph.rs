use napi::{Either, Env, JsString, bindgen_prelude::Array};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, ModuleGraph, RuntimeSpec};

use crate::{
  dependencies::DependencyObject,
  exports_info::JsExportsInfo,
  module::{ModuleObject, ModuleObjectRef},
  module_graph_connection::ModuleGraphConnectionWrapper,
  with_compilation,
};

#[napi]
pub struct JsModuleGraph {
  compilation_id: CompilationId,
  connection_vec_buffer: Vec<ModuleGraphConnectionWrapper>,
}

impl JsModuleGraph {
  pub fn new(compilation: &Compilation) -> Self {
    Self {
      compilation_id: compilation.id(),
      connection_vec_buffer: Vec::new(),
    }
  }

  fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &ModuleGraph) -> napi::Result<R>,
  ) -> napi::Result<R> {
    with_compilation(self.compilation_id, |compilation| {
      if compilation.build_module_graph_artifact.is_stolen() {
        return Err(napi::Error::from_reason(
          "ModuleGraph is not available during module graph building phase".to_string(),
        ));
      }
      let module_graph = compilation.get_module_graph();

      f(compilation, module_graph)
    })
  }
}

#[napi]
impl JsModuleGraph {
  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "Module | null"
  )]
  pub fn get_module(&self, js_dependency: DependencyObject) -> napi::Result<Option<ModuleObject>> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    self.with_ref(|compilation, module_graph| {
      let module = module_graph.get_module_by_dependency_id(&dependency_id);
      let js_module =
        module.map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()));
      Ok(js_module)
    })
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "Module | null"
  )]
  pub fn get_resolved_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleObject>> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    self.with_ref(|compilation, module_graph| {
      Ok(
        match module_graph.connection_by_dependency_id(&dependency_id) {
          Some(connection) => module_graph
            .module_by_identifier(&connection.resolved_module)
            .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
          None => None,
        },
      )
    })
  }

  #[napi(ts_args_type = "module: Module, runtime: string | string[]")]
  pub fn get_used_exports<'a>(
    &self,
    env: &'a Env,
    js_module: ModuleObjectRef,
    js_runtime: Either<String, Vec<String>>,
  ) -> napi::Result<Option<Either<bool, Vec<JsString<'a>>>>> {
    self.with_ref(|compilation, _| {
      let mut runtime = ustr::UstrSet::default();
      match js_runtime {
        Either::A(s) => {
          runtime.insert(s.into());
        }
        Either::B(vec) => {
          runtime.extend(vec.iter().map(String::as_str).map(ustr::Ustr::from));
        }
      };
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info_data(&js_module.identifier);
      let used_exports = exports_info.get_used_exports(Some(&RuntimeSpec::new(runtime)));
      Ok(match used_exports {
        rspack_core::UsedExports::Unknown => None,
        rspack_core::UsedExports::UsedNamespace(b) => Some(Either::A(b)),
        rspack_core::UsedExports::UsedNames(vec) => Some(Either::B(
          vec
            .into_iter()
            .map(|atom| env.create_string(atom.as_str()))
            .collect::<napi::Result<Vec<_>>>()?,
        )),
      })
    })
  }

  #[napi(
    ts_args_type = "module: Module",
    ts_return_type = "true | string[] | null"
  )]
  pub fn get_provided_exports<'a>(
    &self,
    env: &'a Env,
    js_module: ModuleObjectRef,
  ) -> napi::Result<Option<Either<bool, Vec<JsString<'a>>>>> {
    self.with_ref(|compilation, _| {
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info_data(&js_module.identifier);
      let provided = exports_info.get_provided_exports();
      Ok(match provided {
        rspack_core::ProvidedExports::Unknown => None,
        rspack_core::ProvidedExports::ProvidedAll => Some(Either::A(true)),
        rspack_core::ProvidedExports::ProvidedNames(vec) => Some(Either::B(
          vec
            .into_iter()
            .map(|atom| env.create_string(atom.as_str()))
            .collect::<napi::Result<Vec<_>>>()?,
        )),
      })
    })
  }

  #[napi(ts_args_type = "module: Module", ts_return_type = "Module | null")]
  pub fn get_issuer(&self, module: ModuleObjectRef) -> napi::Result<Option<ModuleObject>> {
    self.with_ref(|compilation, module_graph| {
      let issuer = module_graph.get_issuer(&module.identifier);
      Ok(issuer.map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())))
    })
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_exports_info(&self, module: ModuleObjectRef) -> napi::Result<JsExportsInfo> {
    self.with_ref(|compilation, _| {
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info(&module.identifier);
      Ok(JsExportsInfo::new(exports_info, compilation))
    })
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "ModuleGraphConnection | null"
  )]
  pub fn get_connection(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleGraphConnectionWrapper>> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    self.with_ref(|compilation, module_graph| {
      Ok(
        module_graph
          .connection_by_dependency_id(&dependency_id)
          .map(|connection| {
            ModuleGraphConnectionWrapper::new(connection.dependency_id, compilation)
          }),
      )
    })
  }

  #[napi(
    ts_args_type = "module: Module",
    ts_return_type = "ModuleGraphConnection[]"
  )]
  pub fn get_outgoing_connections<'a>(
    &'a mut self,
    env: &'a Env,
    module: ModuleObjectRef,
  ) -> napi::Result<Array<'a>> {
    let compilation_id = self.compilation_id;
    let vec = &mut self.connection_vec_buffer;
    with_compilation(compilation_id, |compilation| {
      if compilation.build_module_graph_artifact.is_stolen() {
        return Err(napi::Error::from_reason(
          "ModuleGraph is not available during module graph building phase".to_string(),
        ));
      }
      let module_graph = compilation.get_module_graph();
      for connection in module_graph.get_outgoing_connections(&module.identifier) {
        vec.push(ModuleGraphConnectionWrapper::new(
          connection.dependency_id,
          compilation,
        ));
      }
      let mut arr = env.create_array(vec.len() as u32)?;
      for (i, v) in vec.drain(..).enumerate() {
        arr.set(i as u32, v)?;
      }
      Ok(arr)
    })
  }

  #[napi(
    ts_args_type = "module: Module",
    ts_return_type = "ModuleGraphConnection[]"
  )]
  pub fn get_outgoing_connections_in_order<'a>(
    &'a mut self,
    env: &'a Env,
    module: ModuleObjectRef,
  ) -> napi::Result<Array<'a>> {
    let compilation_id = self.compilation_id;
    let vec = &mut self.connection_vec_buffer;
    with_compilation(compilation_id, |compilation| {
      if compilation.build_module_graph_artifact.is_stolen() {
        return Err(napi::Error::from_reason(
          "ModuleGraph is not available during module graph building phase".to_string(),
        ));
      }
      let module_graph = compilation.get_module_graph();
      for dependency_id in module_graph.get_outgoing_deps_in_order(&module.identifier) {
        vec.push(ModuleGraphConnectionWrapper::new(
          *dependency_id,
          compilation,
        ));
      }
      let mut arr = env.create_array(vec.len() as u32)?;
      for (i, v) in vec.drain(..).enumerate() {
        arr.set(i as u32, v)?;
      }
      Ok(arr)
    })
  }

  #[napi(
    ts_args_type = "module: Module",
    ts_return_type = "ModuleGraphConnection[]"
  )]
  pub fn get_incoming_connections<'a>(
    &'a mut self,
    env: &'a Env,
    module: ModuleObjectRef,
  ) -> napi::Result<Array<'a>> {
    let compilation_id = self.compilation_id;
    let vec = &mut self.connection_vec_buffer;
    with_compilation(compilation_id, |compilation| {
      if compilation.build_module_graph_artifact.is_stolen() {
        return Err(napi::Error::from_reason(
          "ModuleGraph is not available during module graph building phase".to_string(),
        ));
      }
      let module_graph = compilation.get_module_graph();
      for connection in module_graph.get_incoming_connections(&module.identifier) {
        vec.push(ModuleGraphConnectionWrapper::new(
          connection.dependency_id,
          compilation,
        ));
      }
      let mut arr = env.create_array(vec.len() as u32)?;
      for (i, v) in vec.drain(..).enumerate() {
        arr.set(i as u32, v)?;
      }
      Ok(arr)
    })
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "Module | null"
  )]
  pub fn get_parent_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleObject>> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    self.with_ref(|compilation, module_graph| {
      Ok(match module_graph.get_parent_module(&dependency_id) {
        Some(identifier) => compilation
          .module_by_identifier(identifier)
          .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
        None => None,
      })
    })
  }

  #[napi(ts_args_type = "dependency: Dependency")]
  pub fn get_parent_block_index(&self, js_dependency: DependencyObject) -> napi::Result<i64> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(-1);
    };

    self.with_ref(|_, module_graph| {
      Ok(match module_graph.get_parent_block_index(&dependency_id) {
        Some(block_index) => block_index as i64,
        None => -1,
      })
    })
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn is_async(&self, module: ModuleObjectRef) -> napi::Result<bool> {
    self.with_ref(|compilation, _| {
      Ok(ModuleGraph::is_async(
        &compilation.async_modules_artifact,
        &module.identifier,
      ))
    })
  }
}
