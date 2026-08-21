use std::{cell::RefCell, ptr::NonNull};

use napi::{
  Either, Env,
  bindgen_prelude::{Array, Object, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, DependencyId, internal};
use rspack_napi::OneShotInstanceRef;
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::FxHashMap as HashMap;

use crate::{module::ModuleObject, with_compilation};

// allows JS-side access to a Dependency instance that has already
// been processed and stored in the Compilation.
#[napi]
pub struct Dependency {
  pub(crate) compilation_id: Option<CompilationId>,
  pub(crate) dependency_id: DependencyId,
  pub(crate) dependency: NonNull<dyn rspack_core::Dependency>,
}

impl Dependency {
  fn with_ref<R>(
    &mut self,
    f: impl FnOnce(&dyn rspack_core::Dependency, Option<&Compilation>) -> napi::Result<R>,
  ) -> napi::Result<R> {
    if let Some(compilation_id) = self.compilation_id {
      with_compilation(compilation_id, |compilation| {
        let module_graph = compilation.get_module_graph();
        if let Some(dependency) = internal::try_dependency_by_id(module_graph, &self.dependency_id)
        {
          f(dependency.as_ref(), Some(compilation))
        } else {
          Err(napi::Error::from_reason(format!(
            "Unable to access dependency with id = {:?} now. The dependency have been removed on the Rust side.",
            self.dependency_id
          )))
        }
      })
    } else {
      // SAFETY:
      // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
      // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
      f(unsafe { self.dependency.as_ref() }, None)
    }
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
    self.with_ref(|dependency, compilation| {
      let Some(compilation) = compilation else {
        return Ok(None);
      };
      let module_graph = compilation.get_module_graph();
      let parent_module = module_graph
        .get_parent_module(dependency.id())
        .and_then(|m| compilation.module_by_identifier(m))
        .map(|m| ModuleObject::with_ref(m.as_ref(), compilation.compiler_id()));
      Ok(parent_module)
    })
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<String> {
    self.with_ref(|dependency, _| Ok(dependency.dependency_type().as_str().to_string()))
  }

  #[napi(getter)]
  pub fn category(&mut self) -> napi::Result<String> {
    self.with_ref(|dependency, _| Ok(dependency.category().as_str().to_string()))
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<napi::Either<String, ()>> {
    self.with_ref(|dependency, _| {
      Ok(match dependency.as_module_dependency() {
        Some(dep) => napi::Either::A(dep.request().to_string()),
        None => napi::Either::B(()),
      })
    })
  }

  #[napi(getter, ts_return_type = "Record<string, string> | undefined")]
  pub fn attributes<'a>(&mut self, env: &'a Env) -> napi::Result<Either<Object<'a>, ()>> {
    self.with_ref(|dependency, _| {
      Ok(match dependency.get_attributes() {
        Some(attributes) => {
          let mut object = Object::new(env)?;
          for (key, value) in attributes.iter() {
            object.set(key, value)?;
          }
          Either::A(object)
        }
        None => Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn critical(&mut self) -> napi::Result<bool> {
    self.with_ref(|dependency, _| {
      Ok(match dependency.as_context_dependency() {
        Some(dep) => dep.critical().is_some(),
        None => false,
      })
    })
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    self.with_ref(|dependency, _| {
      if let Some(dep) = dependency.as_context_dependency()
        && !val
      {
        dep.set_critical(None);
      }
      Ok(())
    })
  }

  #[napi(getter, ts_return_type = "Array<string> | undefined")]
  pub fn ids<'a>(&mut self, env: &'a Env) -> napi::Result<Either<Array<'a>, ()>> {
    self.with_ref(|dependency, compilation| {
      Ok(match compilation {
        Some(compilation) => {
          let module_graph = compilation.get_module_graph();
          if let Some(dependency) = dependency.downcast_ref::<CommonJsExportRequireDependency>() {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Either::A(arr)
          } else if let Some(dependency) =
            dependency.downcast_ref::<ESMExportImportedSpecifierDependency>()
          {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Either::A(arr)
          } else if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>()
          {
            let ids = dependency.get_ids(module_graph);
            let mut arr = env.create_array(ids.len() as u32)?;
            for (i, v) in ids.iter().enumerate() {
              arr.set(i as u32, v.as_str())?;
            }
            Either::A(arr)
          } else {
            Either::B(())
          }
        }
        None => Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn loc(&mut self) -> napi::Result<Option<crate::location::DependencyLocation>> {
    self.with_ref(|dependency, _| Ok(dependency.loc().map(|loc| loc.into())))
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
  dependency: NonNull<dyn rspack_core::Dependency>,
  compilation_id: CompilationId,
  registered_compilation_id: Option<CompilationId>,
}

impl DependencyWrapper {
  pub fn new<'a>(
    dependency: &'a dyn rspack_core::Dependency,
    compilation_id: CompilationId,
    compilation: Option<&Compilation>,
  ) -> Self {
    let dependency_id = *dependency.id();

    // SAFETY:
    // We extend `'a` to `'static` to satisfy the `NonNull<dyn Dependency>` field type,
    // which has an implied `'static` bound on the trait object. All actual accesses are
    // still mediated by the JS wrapper lifecycle plus runtime dependency lookups, so the
    // references derived from this pointer must not outlive the original allocation.
    let dependency_ptr = unsafe {
      std::mem::transmute::<
        *const (dyn rspack_core::Dependency + 'a),
        *mut (dyn rspack_core::Dependency + 'static),
      >(dependency)
    };

    // SAFETY:
    // `dependency` is a valid reference, so the erased raw pointer is non-null.
    let dependency = unsafe { NonNull::new_unchecked(dependency_ptr) };

    Self {
      dependency_id,
      dependency,
      compilation_id,
      registered_compilation_id: compilation.map(Compilation::id),
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
            instance.compilation_id = val.registered_compilation_id;
            instance.dependency = val.dependency;

            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(vacant_entry) => {
            let js_dependency = Dependency {
              compilation_id: val.registered_compilation_id,
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
