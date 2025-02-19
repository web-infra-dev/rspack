#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;

use napi::{
  bindgen_prelude::{ClassInstance, ToNapiValue, TypeName, ValidateNapiValue, WeakReference},
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

pub(crate) struct JsDependencyBinding {
  pub(crate) dependency_id: DependencyId,
  pub(crate) dependency: Option<Box<dyn Dependency>>,
  pub(crate) compiler_id: CompilerId,
  pub(crate) compiler_reference: WeakReference<JsCompiler>,
}

#[napi]
pub struct JsDependency(pub(crate) Option<JsDependencyBinding>);

impl JsDependency {
  fn with_ref<T>(
    &self,
    with_dependency: impl FnOnce(&Compilation, &dyn Dependency) -> napi::Result<T>,
    fallback: impl FnOnce() -> napi::Result<T>,
  ) -> napi::Result<T> {
    let Some(binding) = &self.0 else {
      return fallback();
    };

    match binding.compiler_reference.get() {
      Some(this) => {
        let compilation = &this.compiler.compilation;
        let module_graph = compilation.get_module_graph();
        if let Some(dependency) = module_graph.dependency_by_id(&binding.dependency_id) {
          with_dependency(compilation, dependency.as_ref())
        } else if let Some(dependency) = &binding.dependency {
          with_dependency(compilation, dependency.as_ref())
        } else {
          Err(napi::Error::from_reason(format!(
            "Unable to access dependency with id = {:?} now. The module have been removed on the Rust side.",
            binding.dependency_id
          )))
        }
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to access dependency with id = {:?} now. The Compiler has been garbage collected by JavaScript.",
        binding.dependency_id
      ))),
    }
  }

  fn with_mut(
    &mut self,
    mut with_dependency: impl FnMut(&mut dyn Dependency) -> napi::Result<()>,
  ) -> napi::Result<()> {
    let Some(binding) = &mut self.0 else {
      return Ok(());
    };

    match &mut binding.dependency {
      Some(dependency) => {
        with_dependency(dependency.as_mut())
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to modify dependency with id = {:?}. Currently, you can only modify the dependency in the module factory hooks in Rspack.",
        binding.dependency_id
      ))),
    }
  }
}

#[napi]
impl JsDependency {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    Err(napi::Error::from_reason(
      "Rspack currently does not support constructing a Dependency directly from JavaScript.",
    ))
  }

  #[napi(getter)]
  pub fn get_type(&self) -> napi::Result<&str> {
    self.with_ref(
      |_, dependency| Ok(dependency.dependency_type().as_str()),
      || Ok("unknown"),
    )
  }

  #[napi(getter)]
  pub fn category(&self) -> napi::Result<&str> {
    self.with_ref(
      |_, dependency| Ok(dependency.category().as_str()),
      || Ok("unknown"),
    )
  }

  #[napi(getter)]
  pub fn request(&self, env: Env) -> napi::Result<napi::Either<JsString, ()>> {
    self.with_ref(
      |_, dependency| {
        Ok(match dependency.as_module_dependency() {
          Some(dep) => napi::Either::A(env.create_string(dep.request())?),
          None => napi::Either::B(()),
        })
      },
      || Ok(napi::Either::B(())),
    )
  }

  #[napi(getter)]
  pub fn critical(&self) -> napi::Result<bool> {
    self.with_ref(
      |_, dependency| {
        Ok(match dependency.as_context_dependency() {
          Some(dep) => dep.critical().is_some(),
          None => false,
        })
      },
      || Ok(false),
    )
  }

  #[napi(setter)]
  pub fn set_critical(&mut self, val: bool) -> napi::Result<()> {
    self.with_mut(|dependency| {
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
  pub fn ids(&self, env: Env) -> napi::Result<Either<Vec<JsString>, ()>> {
    self.with_ref(
      |compilation, dependency| {
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
          } else if let Some(dependency) = dependency.downcast_ref::<ESMImportSpecifierDependency>()
          {
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
      },
      || Ok(Either::B(())),
    )
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
          if let Some(binding) = &mut instance.0 {
            binding.dependency = val.dependency;
          }
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
          match COMPILER_REFERENCES.with(|ref_cell| {
            let references = ref_cell.borrow();
            references.get(&val.compiler_id).cloned()
          }) {
            Some(compiler_reference) => {
              let js_module = JsDependency(Some(JsDependencyBinding {
                dependency_id: val.dependency_id,
                dependency: val.dependency,
                compiler_id: val.compiler_id,
                compiler_reference,
            }));
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

impl TypeName for JsDependencyWrapper {
  fn type_name() -> &'static str {
    "JsDependency"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsDependencyWrapper {}
