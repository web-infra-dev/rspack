use std::{cell::RefCell, ptr::NonNull};

use napi::bindgen_prelude::ToNapiValue;
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, ConnectionState, DependencyId, ModuleGraph, internal};
use rspack_napi::OneShotRef;
use rustc_hash::FxHashMap;

use crate::{dependency::DependencyWrapper, module::ModuleObject};

#[napi]
pub struct ModuleGraphConnection {
  compilation: NonNull<Compilation>,
  dependency_id: DependencyId,
}

impl ModuleGraphConnection {
  fn as_ref(&self) -> napi::Result<(&'static Compilation, &'static ModuleGraph)> {
    let compilation = unsafe { self.compilation.as_ref() };
    let module_graph = compilation.get_module_graph();

    Ok((compilation, module_graph))
  }
}

#[napi]
impl ModuleGraphConnection {
  #[napi(getter, ts_return_type = "Dependency")]
  pub fn dependency(&self) -> napi::Result<DependencyWrapper> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(dependency) = internal::try_dependency_by_id(module_graph, &self.dependency_id) {
      Ok(DependencyWrapper::new(
        (&**dependency) as &dyn rspack_core::Dependency,
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

  #[napi(getter, ts_return_type = "Module | null")]
  pub fn module(&self) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      let module = module_graph.module_by_identifier(connection.module_identifier());
      Ok(module.map(|m| ModuleObject::with_ref(m.as_ref(), compilation.compiler_id())))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter, ts_return_type = "Module | null")]
  pub fn resolved_module(&self) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      let module = module_graph.module_by_identifier(&connection.resolved_module);
      Ok(module.map(|m| ModuleObject::with_ref(m.as_ref(), compilation.compiler_id())))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter, ts_return_type = "Module | null")]
  pub fn origin_module(&self) -> napi::Result<Option<ModuleObject>> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      Ok(match connection.original_module_identifier {
        Some(original_module_identifier) => module_graph
          .module_by_identifier(&original_module_identifier)
          .map(|m| ModuleObject::with_ref(m.as_ref(), compilation.compiler_id())),
        None => None,
      })
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter)]
  pub fn active(&self) -> napi::Result<bool> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      if let Some(module_graph_cache) = compilation.module_graph_cache_artifact.try_read() {
        Ok(connection.is_active(
          module_graph,
          None,
          module_graph_cache,
          &compilation.exports_info_artifact,
        ))
      } else {
        // Fallback: when module_graph_cache is not available (e.g., during build phase),
        // use is_active without runtime which handles unconditional connections
        Ok(connection.is_active(
          module_graph,
          None,
          &Default::default(),
          &compilation.exports_info_artifact,
        ))
      }
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(getter)]
  pub fn conditional(&self) -> napi::Result<bool> {
    let (_compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      Ok(connection.is_conditional())
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }

  #[napi(ts_args_type = "runtime: string | string[] | undefined")]
  pub fn get_active_state(
    &self,
    runtime: Option<napi::Either<String, Vec<String>>>,
  ) -> napi::Result<String> {
    let (compilation, module_graph) = self.as_ref()?;
    if let Some(connection) = module_graph.connection_by_dependency_id(&self.dependency_id) {
      let runtime_spec = runtime.and_then(|r| {
        let mut set = ustr::UstrSet::default();
        match r {
          napi::Either::A(s) => {
            set.insert(s.into());
          }
          napi::Either::B(vec) => {
            set.extend(vec.iter().map(String::as_str).map(ustr::Ustr::from));
          }
        }
        Some(rspack_core::RuntimeSpec::new(set))
      });
      let module_graph_cache = compilation
        .module_graph_cache_artifact
        .try_read()
        .map(|r| r as &rspack_core::ModuleGraphCacheArtifact);
      let default_cache = Default::default();
      let cache = module_graph_cache.unwrap_or(&default_cache);
      let state = connection.active_state(
        module_graph,
        runtime_spec.as_ref(),
        cache,
        &compilation.exports_info_artifact,
      );
      Ok(match state {
        ConnectionState::Active(true) => "true".to_string(),
        ConnectionState::Active(false) => "false".to_string(),
        ConnectionState::CircularConnection => "circular".to_string(),
        ConnectionState::TransitiveOnly => "transitive-only".to_string(),
      })
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access ModuleGraphConnection with id = {:#?} now. The ModuleGraphConnection have been removed on the Rust side.",
        self.dependency_id
      )))
    }
  }
}

type ModuleGraphConnectionRefs = FxHashMap<DependencyId, OneShotRef>;

type ModuleGraphConnectionRefsByCompilationId =
  RefCell<FxHashMap<CompilationId, ModuleGraphConnectionRefs>>;

thread_local! {
  static MODULE_GRAPH_CONNECTION_INSTANCE_REFS: ModuleGraphConnectionRefsByCompilationId = Default::default();
}

pub struct ModuleGraphConnectionWrapper {
  compilation_id: CompilationId,
  compilation: NonNull<Compilation>,
  dependency_id: DependencyId,
}

impl ModuleGraphConnectionWrapper {
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

impl ToNapiValue for ModuleGraphConnectionWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      MODULE_GRAPH_CONNECTION_INSTANCE_REFS.with(|refs| {
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(val.compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = FxHashMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.dependency_id) {
          std::collections::hash_map::Entry::Occupied(occupied_entry) => {
            let r = occupied_entry.get();
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let js_dependency = ModuleGraphConnection {
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
}
