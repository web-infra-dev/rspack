use std::{ptr::NonNull, sync::Arc};

use napi::{Either, Env, JsString};
use napi_derive::napi;
use rspack_core::{Compilation, ModuleGraph, RuntimeSpec};
use rustc_hash::FxHashSet;

use crate::{
  DependencyObject, JsExportsInfo, JsModule, JsModuleGraphConnectionWrapper, JsModuleWrapper,
};

#[napi]
pub struct JsModuleGraph {
  compilation: NonNull<Compilation>,
}

impl JsModuleGraph {
  pub fn new(compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  fn as_ref(&self) -> napi::Result<(&'static Compilation, ModuleGraph<'static>)> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();

    Ok((compilation, module_graph))
  }
}

#[napi]
impl JsModuleGraph {
  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "JsModule | null"
  )]
  pub fn get_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<JsModuleWrapper>> {
    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    let (compilation, module_graph) = self.as_ref()?;
    let module = module_graph.get_module_by_dependency_id(&dependency_id);
    let js_module = module
      .map(|module| JsModuleWrapper::new(module.identifier(), None, compilation.compiler_id()));
    Ok(js_module)
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "JsModule | null"
  )]
  pub fn get_resolved_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(
      match module_graph.connection_by_dependency_id(&dependency_id) {
        Some(connection) => module_graph
          .module_by_identifier(&connection.resolved_module)
          .map(|module| JsModuleWrapper::new(module.identifier(), None, compilation.compiler_id())),
        None => None,
      },
    )
  }

  #[napi]
  pub fn get_used_exports(
    &self,
    env: Env,
    js_module: &JsModule,
    js_runtime: Either<String, Vec<String>>,
  ) -> napi::Result<Option<Either<bool, Vec<JsString>>>> {
    let (_, module_graph) = self.as_ref()?;

    let mut runtime: FxHashSet<Arc<str>> = FxHashSet::default();
    match js_runtime {
      Either::A(s) => {
        runtime.insert(Arc::from(s));
      }
      Either::B(vec) => {
        runtime.extend(vec.into_iter().map(Arc::from));
      }
    };
    let used_exports =
      module_graph.get_used_exports(&js_module.identifier, Some(&RuntimeSpec::new(runtime)));
    Ok(match used_exports {
      rspack_core::UsedExports::Null => None,
      rspack_core::UsedExports::Bool(b) => Some(Either::A(b)),
      rspack_core::UsedExports::Vec(vec) => Some(Either::B(
        vec
          .into_iter()
          .map(|atom| env.create_string(atom.as_str()))
          .collect::<napi::Result<Vec<_>>>()?,
      )),
    })
  }

  #[napi(ts_return_type = "JsModule | null")]
  pub fn get_issuer(&self, module: &JsModule) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    let issuer = module_graph.get_issuer(&module.identifier);
    Ok(
      issuer
        .map(|module| JsModuleWrapper::new(module.identifier(), None, compilation.compiler_id())),
    )
  }

  #[napi]
  pub fn get_exports_info(&self, module: &JsModule) -> napi::Result<JsExportsInfo> {
    let (compilation, module_graph) = self.as_ref()?;
    let exports_info = module_graph.get_exports_info(&module.identifier);
    Ok(JsExportsInfo::new(exports_info, compilation))
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "JsModuleGraphConnection | null"
  )]
  pub fn get_connection(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<JsModuleGraphConnectionWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(
      module_graph
        .connection_by_dependency_id(&dependency_id)
        .map(|connection| {
          JsModuleGraphConnectionWrapper::new(connection.dependency_id, compilation)
        }),
    )
  }

  #[napi(ts_return_type = "JsModuleGraphConnection[]")]
  pub fn get_outgoing_connections(
    &self,
    module: &JsModule,
  ) -> napi::Result<Vec<JsModuleGraphConnectionWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    Ok(
      module_graph
        .get_outgoing_connections(&module.identifier)
        .map(|connection| {
          JsModuleGraphConnectionWrapper::new(connection.dependency_id, compilation)
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsModuleGraphConnection[]")]
  pub fn get_outgoing_connections_in_order(
    &self,
    module: &JsModule,
  ) -> napi::Result<Vec<JsModuleGraphConnectionWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    Ok(
      module_graph
        .get_outgoing_connections_in_order(&module.identifier)
        .map(|dependency_id| JsModuleGraphConnectionWrapper::new(*dependency_id, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsModuleGraphConnection[]")]
  pub fn get_incoming_connections(
    &self,
    module: &JsModule,
  ) -> napi::Result<Vec<JsModuleGraphConnectionWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    Ok(
      module_graph
        .get_incoming_connections(&module.identifier)
        .map(|connection| {
          JsModuleGraphConnectionWrapper::new(connection.dependency_id, compilation)
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi(
    ts_args_type = "dependency: Dependency",
    ts_return_type = "JsModule | null"
  )]
  pub fn get_parent_module(
    &self,
    js_dependency: DependencyObject,
  ) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;

    let Some(dependency_id) = js_dependency.dependency_id() else {
      return Ok(None);
    };

    Ok(match module_graph.get_parent_module(&dependency_id) {
      Some(identifier) => compilation
        .module_by_identifier(identifier)
        .map(|module| JsModuleWrapper::new(module.identifier(), None, compilation.compiler_id())),
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

  #[napi]
  pub fn is_async(&self, module: &JsModule) -> napi::Result<bool> {
    let (compilation, _) = self.as_ref()?;
    Ok(ModuleGraph::is_async(compilation, &module.identifier))
  }
}
