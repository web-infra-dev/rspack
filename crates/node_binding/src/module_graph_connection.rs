use std::cell::RefCell;

use napi::bindgen_prelude::{ToNapiValue, WeakReference};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, CompilerId, DependencyId, ModuleGraph};
use rspack_napi::OneShotRef;
use rustc_hash::FxHashMap as HashMap;

use crate::{JsDependencyWrapper, JsModuleWrapper, Rspack, COMPILER_REFERENCES};

#[napi]
pub struct JsModuleGraphConnection {
  compiler_reference: WeakReference<Rspack>,
  dependency_id: DependencyId,
}

impl JsModuleGraphConnection {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, ModuleGraph<'static>)> {
    match self.compiler_reference.get() {
      Some(reference) => {
        let compilation = unsafe {
          std::mem::transmute::<&Compilation, &'static Compilation>(&reference.compiler.compilation)
        };
        let module_graph = compilation.get_module_graph();

        Ok((compilation, module_graph))
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to access compiler. The compiler have been GC on the JavaScript side."
      ))),
    }
  }
}

#[napi]
impl JsModuleGraphConnection {
  #[napi(getter, ts_return_type = "JsDependency")]
  pub fn dependency(&self) -> napi::Result<JsDependencyWrapper> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(dependency) = module_graph.dependency_by_id(&self.dependency_id) {
      Ok(JsDependencyWrapper::new(
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
      Ok(module.map(|m| JsModuleWrapper::new(m.as_ref(), compilation.id(), Some(compilation))))
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
      Ok(module.map(|m| JsModuleWrapper::new(m.as_ref(), compilation.id(), Some(compilation))))
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
          .map(|m| JsModuleWrapper::new(m.as_ref(), compilation.id(), Some(compilation))),
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

type ModuleGraphConnectionRefs = HashMap<DependencyId, OneShotRef<JsModuleGraphConnection>>;

type ModuleGraphConnectionRefsByCompilerId =
  RefCell<HashMap<CompilerId, ModuleGraphConnectionRefs>>;

thread_local! {
  static MODULE_GRAPH_CONNECTION_INSTANCE_REFS: ModuleGraphConnectionRefsByCompilerId = Default::default();
}

pub struct JsModuleGraphConnectionWrapper {
  compiler_id: CompilerId,
  compiler_reference: WeakReference<Rspack>,
  dependency_id: DependencyId,
}

impl JsModuleGraphConnectionWrapper {
  pub fn new(dependency_id: DependencyId, compiler_id: CompilerId) -> Self {
    let compiler_reference = COMPILER_REFERENCES.with(|ref_cell| {
      let references = ref_cell.borrow();
      #[allow(clippy::unwrap_used)]
      references.get(&compiler_id).unwrap().clone()
    });
    Self {
      dependency_id,
      compiler_id,
      compiler_reference,
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    // MODULE_GRAPH_CONNECTION_INSTANCE_REFS.with(|refs| {
    //   let mut refs_by_compilation_id = refs.borrow_mut();
    //   refs_by_compilation_id.remove(&compilation_id)
    // });
  }
}

impl ToNapiValue for JsModuleGraphConnectionWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    MODULE_GRAPH_CONNECTION_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      let entry = refs_by_compilation_id.entry(val.compiler_id);
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
            compiler_reference: val.compiler_reference,
            dependency_id: val.dependency_id,
          };
          let r = vacant_entry.insert(OneShotRef::new(env, js_dependency)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
