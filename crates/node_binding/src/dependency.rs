use std::{cell::RefCell, ptr::NonNull};

use napi::{bindgen_prelude::ToNapiValue, Either, Env, JsString};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, Dependency, DependencyId};
use rspack_napi::OneShotInstanceRef;
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::FxHashMap as HashMap;

// JsDependency allows JS-side access to a Dependency instance that has already
// been processed and stored in the Compilation.
#[napi]
pub struct JsDependency {
  pub(crate) compilation: Option<NonNull<Compilation>>,
  pub(crate) dependency_id: DependencyId,
  pub(crate) dependency: NonNull<dyn Dependency>,
}

impl JsDependency {
  fn as_ref(&mut self) -> napi::Result<(&dyn Dependency, Option<&Compilation>)> {
    if let Some(compilation) = self.compilation {
      let compilation = unsafe { compilation.as_ref() };
      let module_graph = compilation.get_module_graph();
      if let Some(dependency) = module_graph.dependency_by_id(&self.dependency_id) {
        self.dependency = {
          #[allow(clippy::unwrap_used)]
          NonNull::new(dependency.as_ref() as *const dyn Dependency as *mut dyn Dependency).unwrap()
        };
        Ok((unsafe { self.dependency.as_ref() }, Some(compilation)))
      } else {
        Err(napi::Error::from_reason(format!(
          "Unable to access dependency with id = {:?} now. The dependency have been removed on the Rust side.",
          self.dependency_id
        )))
      }
    } else {
      // SAFETY:
      // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
      // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
      Ok((unsafe { self.dependency.as_ref() }, None))
    }
  }

  fn as_mut(&mut self) -> napi::Result<&mut dyn Dependency> {
    // SAFETY:
    // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
    // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
    Ok(unsafe { self.dependency.as_mut() })
  }
}

#[napi]
impl JsDependency {
  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    let (dependency, _) = self.as_ref()?;

    Ok(dependency.dependency_type().as_str())
  }

  #[napi(getter)]
  pub fn category(&mut self) -> napi::Result<&str> {
    let (dependency, _) = self.as_ref()?;

    Ok(dependency.category().as_str())
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<napi::Either<&str, ()>> {
    let (dependency, _) = self.as_ref()?;

    Ok(match dependency.as_module_dependency() {
      Some(dep) => napi::Either::A(dep.request()),
      None => napi::Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn critical(&mut self) -> napi::Result<bool> {
    let (dependency, _) = self.as_ref()?;

    Ok(match dependency.as_context_dependency() {
      Some(dep) => dep.critical().is_some(),
      None => false,
    })
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    let dependency = self.as_mut()?;

    if let Some(dep) = dependency.as_context_dependency_mut() {
      let critical = dep.critical_mut();
      if !val {
        *critical = None;
      }
    }
    Ok(())
  }

  #[napi(getter)]
  pub fn ids(&mut self, env: Env) -> napi::Result<Either<Vec<JsString>, ()>> {
    let (dependency, compilation) = self.as_ref()?;

    Ok(match compilation {
      Some(compilation) => {
        let module_graph = compilation.get_module_graph();
        if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>() {
          let ids = dependency
            .get_ids(&module_graph)
            .iter()
            .map(|atom| env.create_string(atom.as_str()))
            .collect::<napi::Result<Vec<_>>>()?;
          Either::A(ids)
        } else if let Some(dependency) =
          dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
        {
          let ids = dependency
            .get_ids(&module_graph)
            .iter()
            .map(|atom| env.create_string(atom.as_str()))
            .collect::<napi::Result<Vec<_>>>()?;
          Either::A(ids)
        } else if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>() {
          let ids = dependency
            .get_ids(&module_graph)
            .iter()
            .map(|atom| env.create_string(atom.as_str()))
            .collect::<napi::Result<Vec<_>>>()?;
          Either::A(ids)
        } else {
          Either::B(())
        }
      }
      None => Either::B(()),
    })
  }
}

type DependencyInstanceRefs = HashMap<DependencyId, OneShotInstanceRef<JsDependency>>;

type DependencyInstanceRefsByCompilationId =
  RefCell<HashMap<CompilationId, DependencyInstanceRefs>>;

thread_local! {
  static DEPENDENCY_INSTANCE_REFS: DependencyInstanceRefsByCompilationId = Default::default();
}

pub struct JsDependencyWrapper {
  dependency_id: DependencyId,
  dependency: NonNull<dyn Dependency>,
  compilation_id: CompilationId,
  compilation: Option<NonNull<Compilation>>,
}

impl JsDependencyWrapper {
  pub fn new(
    dependency: &dyn Dependency,
    compilation_id: CompilationId,
    compilation: Option<&Compilation>,
  ) -> Self {
    let dependency_id = *dependency.id();

    #[allow(clippy::unwrap_used)]
    Self {
      dependency_id,
      dependency: NonNull::new(dependency as *const dyn Dependency as *mut dyn Dependency).unwrap(),
      compilation_id,
      compilation: compilation
        .map(|c| NonNull::new(c as *const Compilation as *mut Compilation).unwrap()),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }
}

impl ToNapiValue for JsDependencyWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
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
        std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
          let r = occupied_entry.get_mut();
          let instance = &mut **r;
          instance.compilation = val.compilation;
          instance.dependency = val.dependency;

          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
          let js_dependency = JsDependency {
            compilation: val.compilation,
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

#[napi(object)]
pub struct RawDependency {
  pub request: String,
}
