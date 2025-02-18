#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;

use napi::{
  bindgen_prelude::{
    ClassInstance, FromNapiMutRef, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue,
    WeakReference,
  },
  Either, Env, JsString,
};
use napi_derive::napi;
use rspack_collections::UkeyMap;
use rspack_core::{Compilation, CompilerId, Dependency, DependencyId};
use rspack_napi::OneShotInstanceRef;
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};

use crate::{JsCompiler, COMPILER_REFERENCES};

// JsDependency allows JS-side access to a Dependency instance that has already
// been processed and stored in the Compilation.
#[napi]
pub struct JsDependency {
  pub(crate) dependency_id: DependencyId,
  pub(crate) dependency: Option<Box<dyn Dependency>>,
  compiler_id: CompilerId,
  compiler_reference: WeakReference<JsCompiler>,
}

impl JsDependency {
  fn as_ref<T>(
    &mut self,
    f: impl FnOnce(&Compilation, &dyn Dependency) -> napi::Result<T>,
  ) -> napi::Result<T> {
    match self.compiler_reference.get() {
      Some(this) => {
        let compilation = &this.compiler.compilation;
        let module_graph = compilation.get_module_graph();
        if let Some(dependency) = module_graph.dependency_by_id(&self.dependency_id) {
          f(compilation, dependency.as_ref())
        } else if let Some(dependency) = &self.dependency {
          f(compilation, dependency.as_ref())
        } else {
          Err(napi::Error::from_reason(format!(
            "Unable to access dependency with id = {:?} now. The module have been removed on the Rust side.",
            self.dependency_id
          )))
        }
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to access dependency with id = {:?} now. The Compiler has been garbage collected by JavaScript.",
        self.dependency_id
      ))),
    }
  }

  fn as_mut(
    &mut self,
    mut f: impl FnMut(&mut dyn Dependency) -> napi::Result<()>,
  ) -> napi::Result<()> {
    match &mut self.dependency {
      Some(dependency) => {
        f(dependency.as_mut())
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to modify dependency with id = {:?}. Currently, you can only modify the dependency in the module factory hooks in Rspack.",
        self.dependency_id
      ))),
    }
  }
}

#[napi]
impl JsDependency {
  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    self.as_ref(|_, dependency| Ok(dependency.dependency_type().as_str()))
  }

  #[napi(getter)]
  pub fn category(&mut self) -> napi::Result<&str> {
    self.as_ref(|_, dependency| Ok(dependency.category().as_str()))
  }

  #[napi(getter)]
  pub fn request(&mut self, env: Env) -> napi::Result<napi::Either<JsString, ()>> {
    self.as_ref(|_, dependency| {
      Ok(match dependency.as_module_dependency() {
        Some(dep) => napi::Either::A(env.create_string(dep.request())?),
        None => napi::Either::B(()),
      })
    })
  }

  #[napi(getter)]
  pub fn critical(&mut self) -> napi::Result<bool> {
    self.as_ref(|_, dependency| {
      Ok(match dependency.as_context_dependency() {
        Some(dep) => dep.critical().is_some(),
        None => false,
      })
    })
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    self.as_mut(|dependency| {
      if let Some(dep) = dependency.as_context_dependency_mut() {
        let critical = dep.critical_mut();
        if !val {
          *critical = None;
        }
      }
      Ok(())
    })
  }

  #[napi(getter)]
  pub fn ids(&mut self, env: Env) -> napi::Result<Either<Vec<JsString>, ()>> {
    self.as_ref(|compilation, dependency| {
      let module_graph = compilation.get_module_graph();
      Ok(
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
        },
      )
    })
  }
}

type DependencyInstanceRefs = UkeyMap<DependencyId, OneShotInstanceRef<JsDependency>>;

type DependencyInstanceRefsByCompilerId = RefCell<UkeyMap<CompilerId, DependencyInstanceRefs>>;

thread_local! {
  static DEPENDENCY_INSTANCE_REFS: DependencyInstanceRefsByCompilerId = Default::default();
}

pub struct JsDependencyWrapper {
  dependency_id: DependencyId,
  dependency: Option<Box<dyn Dependency>>,
  compiler_id: CompilerId,
}

impl JsDependencyWrapper {
  pub fn from_id(dependency_id: DependencyId, compiler_id: CompilerId) -> Self {
    Self {
      dependency_id,
      dependency: None,
      compiler_id,
    }
  }

  pub fn from_owned(dependency: Box<dyn Dependency>, compiler_id: CompilerId) -> Self {
    Self {
      dependency_id: *dependency.id(),
      dependency: Some(dependency),
      compiler_id,
    }
  }

  pub fn take(&mut self) -> Option<Box<dyn Dependency>> {
    self.dependency.take()
  }

  pub fn into_instance<'scope>(
    self,
    env: napi::sys::napi_env,
  ) -> napi::Result<ClassInstance<'scope, JsDependency>> {
    let compiler_id = self.compiler_id;
    let dependency_id = self.dependency_id;

    let napi_value = unsafe { ToNapiValue::to_napi_value(env, self)? };
    let wrapped_value = DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let refs_by_compiler_id = refs.borrow();
      #[allow(clippy::unwrap_used)]
      let refs = refs_by_compiler_id.get(&compiler_id).unwrap();
      #[allow(clippy::unwrap_used)]
      let r = refs.get(&dependency_id).unwrap();
      &**r as *const _ as *mut JsDependency
    });

    Ok(unsafe { ClassInstance::new(napi_value, env, wrapped_value) })
  }

  pub fn cleanup_by_compiler_id(compiler_id: &CompilerId) {
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      refs_by_compiler_id.remove(compiler_id)
    });
  }

  pub fn cleanup_last_compilation(compilation: &Compilation) {
    let module_graph = compilation.get_module_graph();
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      if let Some(refs) = refs_by_compiler_id.get_mut(&compilation.compiler_id()) {
        refs.retain(|dependency_id, _| module_graph.dependency_by_id(dependency_id).is_some());
      }
    });
  }
}

impl ToNapiValue for JsDependencyWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    DEPENDENCY_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      let entry = refs_by_compiler_id.entry(val.compiler_id);
      let refs = match entry {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
          entry.insert(UkeyMap::default())
        }
      };

      match refs.entry(val.dependency_id) {
        std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
          let r = occupied_entry.get_mut();
          let instance = &mut **r;
          instance.dependency = val.dependency;
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
          match COMPILER_REFERENCES.with(|ref_cell| {
            let references = ref_cell.borrow();
            references.get(&val.compiler_id).cloned()
          }) {
            Some(compiler_reference) => {
              let js_module = JsDependency {
                dependency_id: val.dependency_id,
                dependency: val.dependency,
                compiler_id: val.compiler_id,
                compiler_reference,
            };
              let r = vacant_entry.insert(OneShotInstanceRef::new(env, js_module)?);
              ToNapiValue::to_napi_value(env, r)
            },
            None => {
              Err(napi::Error::from_reason(format!(
                "Unable to construct dependency with id = {:?} now. The Compiler has been garbage collected by JavaScript.",
                val.dependency_id
              )))
            },
          }
        }
      }
    })
  }
}

impl FromNapiValue for JsDependencyWrapper {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let instance = <JsDependency as FromNapiMutRef>::from_napi_mut_ref(env, napi_val)?;
    Ok(Self {
      dependency_id: instance.dependency_id,
      dependency: instance.dependency.take(),
      compiler_id: instance.compiler_id,
    })
  }
}

impl TypeName for JsDependencyWrapper {
  fn type_name() -> &'static str {
    "JsDependency"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsDependencyWrapper {}

#[napi(object)]
pub struct RawDependency {
  pub request: String,
}
