use std::ptr::NonNull;

use napi::{Either, Env, JsString, bindgen_prelude::Array};
use napi_derive::napi;
use rspack_core::{Compilation, ModuleGraph, ModuleGraphRef, PrefetchExportsInfoMode, RuntimeSpec};

use crate::{
  dependencies::DependencyObject,
  exports_info::JsExportsInfo,
  module::{ModuleObject, ModuleObjectRef},
  module_graph_connection::ModuleGraphConnectionWrapper,
};

#[napi]
pub struct JsModuleGraph {
  compilation: NonNull<Compilation>,
  connection_vec_buffer: Vec<ModuleGraphConnectionWrapper>,
}

impl JsModuleGraph {
  pub fn new(compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
      connection_vec_buffer: Vec::new(),
    }
  }

  fn as_ref(&self) -> napi::Result<(&'static Compilation, ModuleGraphRef<'static>)> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();

    Ok((compilation, module_graph))
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

    let (compilation, module_graph) = self.as_ref()?;
    let module = module_graph.get_module_by_dependency_id(&dependency_id);
    let js_module =
      module.map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()));
    Ok(js_module)
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "Module | null"
  )]
  pub fn get_resolved_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(
      match module_graph.connection_by_dependency_id(&dependency_id) {
        Some(connection) => module_graph
          .module_by_identifier(&connection.resolved_module)
          .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
        None => None,
      },
    )
  }

  #[napi(ts_args_type = "module: Module, runtime: string | string[]")]
  pub fn get_used_exports<'a>(
    &self,
    env: &'a Env,
    js_module: ModuleObjectRef,
    js_runtime: Either<String, Vec<String>>,
  ) -> napi::Result<Option<Either<bool, Vec<JsString<'a>>>>> {
    let (_, module_graph) = self.as_ref()?;

    let mut runtime = ustr::UstrSet::default();
    match js_runtime {
      Either::A(s) => {
        runtime.insert(s.into());
      }
      Either::B(vec) => {
        runtime.extend(vec.iter().map(String::as_str).map(ustr::Ustr::from));
      }
    };
    let exports_info = module_graph
      .get_prefetched_exports_info(&js_module.identifier, PrefetchExportsInfoMode::Default);
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
  }

  #[napi(ts_args_type = "module: Module", ts_return_type = "Module | null")]
  pub fn get_issuer(&self, module: ModuleObjectRef) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;
    let issuer = module_graph.get_issuer(&module.identifier);
    Ok(issuer.map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())))
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn get_exports_info(&self, module: ModuleObjectRef) -> napi::Result<JsExportsInfo> {
    let (compilation, module_graph) = self.as_ref()?;
    let exports_info = module_graph.get_exports_info(&module.identifier);
    Ok(JsExportsInfo::new(exports_info, compilation))
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "ModuleGraphConnection | null"
  )]
  pub fn get_connection(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleGraphConnectionWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(
      module_graph
        .connection_by_dependency_id(&dependency_id)
        .map(|connection| ModuleGraphConnectionWrapper::new(connection.dependency_id, compilation)),
    )
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
    let (compilation, module_graph) = self.as_ref()?;
    let vec = &mut self.connection_vec_buffer;
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
    let (compilation, module_graph) = self.as_ref()?;

    let vec = &mut self.connection_vec_buffer;
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
    let (compilation, module_graph) = self.as_ref()?;

    let vec = &mut self.connection_vec_buffer;
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
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "Module | null"
  )]
  pub fn get_parent_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(match module_graph.get_parent_module(&dependency_id) {
      Some(identifier) => compilation
        .module_by_identifier(identifier)
        .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id())),
      None => None,
    })
  }

  #[napi(ts_args_type = "dependency: Dependency")]
  pub fn get_parent_block_index(&self, js_dependency: DependencyObject) -> napi::Result<i64> {
    let (_, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(-1);
    };

    Ok(match module_graph.get_parent_block_index(&dependency_id) {
      Some(block_index) => block_index as i64,
      None => -1,
    })
  }

  #[napi(ts_args_type = "module: Module")]
  pub fn is_async(&self, module: ModuleObjectRef) -> napi::Result<bool> {
    let (compilation, _) = self.as_ref()?;
    Ok(ModuleGraph::is_async(compilation, &module.identifier))
  }
}
