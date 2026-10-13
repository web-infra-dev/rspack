use std::cell::RefCell;

use napi::{
  Either, Env,
  bindgen_prelude::{Array, Object, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, DependencyId, DependencyRef, ModuleGraph, internal};
use rspack_napi::OneShotInstanceRef;
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{module::ModuleObject, with_compilation};

// A dependency remains readable while its JS wrapper owns a shared reference.
// Graph-derived properties still require the compilation that supplied it.
#[napi]
pub struct Dependency {
  compilation_id: Option<CompilationId>,
  pub(crate) dependency_id: DependencyId,
  dependency: DependencyRef,
}

impl Dependency {
  fn with_graph<R>(
    &self,
    f: impl FnOnce(&Compilation, &ModuleGraph, &dyn rspack_core::Dependency) -> napi::Result<R>,
  ) -> napi::Result<Option<R>> {
    let Some(compilation_id) = self.compilation_id else {
      return Ok(None);
    };
    with_compilation(compilation_id, |compilation| {
      let module_graph = compilation.try_get_module_graph().ok_or_else(|| {
        napi::Error::from_reason("ModuleGraph is not available during module graph building phase")
      })?;
      let dependency = internal::try_dependency_by_id(module_graph, &self.dependency_id).ok_or_else(
        || {
          napi::Error::from_reason(format!(
            "Unable to access dependency with id = {:?} now. The dependency has been removed on the Rust side.",
            self.dependency_id
          ))
        },
      )?;
      f(compilation, module_graph, dependency).map(Some)
    })
  }
}

#[napi]
impl Dependency {
  #[napi(
    getter,
    js_name = "_parentModule",
    ts_return_type = "Module | undefined"
  )]
  pub fn parent_module(&mut self) -> napi::Result<Option<ModuleObject>> {
    self
      .with_graph(|compilation, module_graph, dependency| {
        let parent_module = module_graph
          .get_parent_module(dependency.id())
          .and_then(|m| compilation.module_by_identifier(m))
          .map(|m| ModuleObject::with_ref(m.as_ref(), compilation.compiler_id()));
        Ok(parent_module)
      })
      .map(Option::flatten)
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<String> {
    Ok(self.dependency.dependency_type().as_str().to_string())
  }

  #[napi(getter)]
  pub fn category(&mut self) -> napi::Result<String> {
    Ok(self.dependency.category().as_str().to_string())
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<napi::Either<String, ()>> {
    Ok(match self.dependency.as_module_dependency() {
      Some(dep) => napi::Either::A(dep.request().to_string()),
      None => napi::Either::B(()),
    })
  }

  #[napi(getter, ts_return_type = "Record<string, string> | undefined")]
  pub fn attributes<'a>(&mut self, env: &'a Env) -> napi::Result<Either<Object<'a>, ()>> {
    Ok(match self.dependency.get_attributes() {
      Some(attributes) => {
        let mut object = Object::new(env)?;
        for (key, value) in attributes.iter() {
          object.set(key, value)?;
        }
        Either::A(object)
      }
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn critical(&mut self) -> napi::Result<bool> {
    Ok(match self.dependency.as_context_dependency() {
      Some(dep) => dep.critical().is_some(),
      None => false,
    })
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    if let Some(dep) = self.dependency.as_context_dependency()
      && !val
    {
      dep.set_critical(None);
    }
    Ok(())
  }

  #[napi(getter, ts_return_type = "Array<string> | undefined")]
  pub fn ids<'a>(&mut self, env: &'a Env) -> napi::Result<Either<Array<'a>, ()>> {
    Ok(
      self
        .with_graph(|_, module_graph, dependency| {
          if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>() {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Ok(Either::A(arr))
          } else if let Some(dependency) =
            dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
          {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Ok(Either::A(arr))
          } else if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>()
          {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Ok(Either::A(arr))
          } else {
            Ok(Either::B(()))
          }
        })?
        .unwrap_or(Either::B(())),
    )
  }

  #[napi(getter)]
  pub fn loc(&mut self) -> napi::Result<Option<crate::location::DependencyLocation>> {
    Ok(self.dependency.loc().map(|loc| loc.into()))
  }
}

type DependencyInstanceRefs = HashMap<DependencyId, OneShotInstanceRef<Dependency>>;

type DependencyInstanceRefsByCompilationId =
  RefCell<HashMap<CompilationId, DependencyInstanceRefs>>;

thread_local! {
  static DEPENDENCY_INSTANCE_REFS: DependencyInstanceRefsByCompilationId = Default::default();
}

pub struct DependencyWrapper {
  dependency_id: DependencyId,
  dependency: DependencyRef,
  cache_compilation_id: CompilationId,
  compilation_id: Option<CompilationId>,
}

impl DependencyWrapper {
  pub fn from_graph(dependency: DependencyRef, compilation_id: CompilationId) -> Self {
    let dependency_id = *dependency.id();
    Self {
      dependency_id,
      dependency,
      cache_compilation_id: compilation_id,
      compilation_id: Some(compilation_id),
    }
  }

  pub fn from_hook(dependency: DependencyRef, compilation_id: CompilationId) -> Self {
    let dependency_id = *dependency.id();
    Self {
      dependency_id,
      dependency,
      cache_compilation_id: compilation_id,
      compilation_id: None,
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for DependencyWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      DEPENDENCY_INSTANCE_REFS.with(|refs| {
        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(val.cache_compilation_id);
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = HashMap::default();
            entry.insert(refs)
          }
        };

        match refs.entry(val.dependency_id) {
          std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
            let r = occupied_entry.get_mut();
            let instance = &mut **r;
            instance.compilation_id = val.compilation_id;
            instance.dependency = val.dependency;

            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let js_dependency = Dependency {
              compilation_id: val.compilation_id,
              dependency_id: val.dependency_id,
              dependency: val.dependency,
            };
            let r = vacant_entry.insert(OneShotInstanceRef::new(env, js_dependency)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      })
    }
  }
}
