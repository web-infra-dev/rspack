use std::{cell::RefCell, ptr::NonNull};

use napi::bindgen_prelude::ToNapiValue;
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, DependencyId, ModuleGraph};
use rspack_napi::OneShotRef;
use rustc_hash::FxHashMap as HashMap;

use crate::{DependencyWrapper, JsModuleWrapper};

#[napi]
pub struct JsModuleGraphConnection {
  compilation: NonNull<Compilation>,
  dependency_id: DependencyId,
}

impl JsModuleGraphConnection {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, ModuleGraph<'static>)> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();

    Ok((compilation, module_graph))
  }
}

#[napi]
impl JsModuleGraphConnection {
  #[napi(getter, ts_return_type = "Dependency")]
  pub fn dependency(&self) -> napi::Result<DependencyWrapper> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(dependency) = module_graph.dependency_by_id(&self.dependency_id) {
      Ok(DependencyWrapper::new(
        dependency.as_ref(),
        compilation.id(),
        Some(compilation),
      ))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access Dependency with id = {:#?} now. The Dependency have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter, ts_return_type = "JsModule | null")]
  pub fn module(&self) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      let module = module_graph.module_by_identifier(connection.module_identifier());
      Ok(module.map(|m| JsModuleWrapper::new(m.identifier(), None, compilation.compiler_id())))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter, ts_return_type = "JsModule | null")]
  pub fn resolved_module(&self) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      let module = module_graph.module_by_identifier(&connection.resolved_module);
      Ok(module.map(|m| JsModuleWrapper::new(m.identifier(), None, compilation.compiler_id())))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter, ts_return_type = "JsModule | null")]
  pub fn origin_module(&self) -> napi::Result<Option<JsModuleWrapper>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      Ok(match connection.original_module_identifier {
        Some(original_module_identifier) => module_graph
          .module_by_identifier(&original_module_identifier)
          .map(|m| JsModuleWrapper::new(m.identifier(), None, compilation.compiler_id())),
        None => None,
      })
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }
}

type ModuleGraphConnectionRefs = HashMap<DependencyId, OneShotRef>;

type ModuleGraphConnectionRefsByCompilationId =
  RefCell<HashMap<CompilationId, ModuleGraphConnectionRefs>>;

thread_local! {
  static MODULE_GRAPH_CONNECTION_INSTANCE_REFS: ModuleGraphConnectionRefsByCompilationId = Default::default();
}

pub struct JsModuleGraphConnectionWrapper {
  compilation_id: CompilationId,
  compilation: NonNull<Compilation>,
  dependency_id: DependencyId,
}

impl JsModuleGraphConnectionWrapper {
  pub fn new(dependency_id: DependencyId, compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      dependency_id,
      compilation_id: compilation.id(),
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    MODULE_GRAPH_CONNECTION_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for JsModuleGraphConnectionWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    MODULE_GRAPH_CONNECTION_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      let entry = refs_by_compilation_id.entry(val.compilation_id);
      let refs = match entry {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
          let refs = HashMap::default();
          entry.insert(refs)
        }
      };

      match refs.entry(val.dependency_id) {
        std::collections::hash_map::Entry::Occupied(occupied_entry) => {
          let r = occupied_entry.get();
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
          let js_dependency = JsModuleGraphConnection {
            compilation: val.compilation,
            dependency_id: val.dependency_id,
          };
          let r = vacant_entry.insert(OneShotRef::new(env, js_dependency)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
