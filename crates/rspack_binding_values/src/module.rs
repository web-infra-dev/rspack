use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use napi_derive::napi;
use rspack_collections::IdentifierMap;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, CompilationId,
  DependenciesBlock, Module, ModuleGraph, ModuleIdentifier, RuntimeModuleStage, SourceType,
};
use rspack_napi::{napi::bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, OneShotRef};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap as HashMap;

use super::{JsCompatSource, ToJsCompatSource};
use crate::{JsChunk, JsCodegenerationResults, JsDependency};

#[derive(Default)]
#[napi(object)]
pub struct JsFactoryMeta {
  pub side_effect_free: Option<bool>,
}

#[napi]
pub struct JsDependenciesBlock {
  block_id: AsyncDependenciesBlockIdentifier,
  compilation: NonNull<Compilation>,
}

impl JsDependenciesBlock {
  pub fn new(block_id: AsyncDependenciesBlockIdentifier, compilation: *const Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      block_id,
      compilation: NonNull::new(compilation as *mut Compilation).unwrap(),
    }
  }

  fn block<'a>(&self, module_graph: &'a ModuleGraph) -> &'a AsyncDependenciesBlock {
    module_graph.block_by_id(&self.block_id).unwrap_or_else(|| {
      panic!(
        "Cannot find block with id = {:?}. It might have been removed on the Rust side.",
        self.block_id
      )
    })
  }
}

#[napi]
impl JsDependenciesBlock {
  #[napi(getter)]
  pub fn dependencies(&self) -> Vec<JsDependency> {
    let compilation = unsafe { self.compilation.as_ref() };

    let module_graph = compilation.get_module_graph();
    let block = self.block(&module_graph);
    block
      .get_dependencies()
      .iter()
      .filter_map(|dependency_id| {
        module_graph
          .dependency_by_id(dependency_id)
          .map(JsDependency::new)
      })
      .collect::<Vec<_>>()
  }

  #[napi(getter)]
  pub fn blocks(&self) -> Vec<JsDependenciesBlock> {
    let compilation = unsafe { self.compilation.as_ref() };

    let module_graph = compilation.get_module_graph();
    let block = self.block(&module_graph);
    let blocks = block.get_blocks();
    blocks
      .iter()
      .cloned()
      .map(|block_id| JsDependenciesBlock::new(block_id, self.compilation.as_ptr()))
      .collect::<Vec<_>>()
  }
}

#[napi]
pub struct JsModule {
  identifier: ModuleIdentifier,
  module: NonNull<dyn Module>,
  compilation: Option<NonNull<Compilation>>,
}

impl JsModule {
  fn attach(&mut self, compilation: *const Compilation) {
    if self.compilation.is_none() {
      self.compilation = Some(
        #[allow(clippy::unwrap_used)]
        NonNull::new(compilation as *mut Compilation).unwrap(),
      );
    }
  }

  fn as_ref(&mut self) -> napi::Result<&'static dyn Module> {
    let module = unsafe { self.module.as_ref() };
    if module.identifier() == self.identifier {
      return Ok(module);
    }

    if let Some(compilation) = self.compilation {
      let compilation = unsafe { compilation.as_ref() };
      if let Some(module) = compilation.module_by_identifier(&self.identifier) {
        let module = module.as_ref();
        self.module = {
          #[allow(clippy::unwrap_used)]
          NonNull::new(module as *const dyn Module as *mut dyn Module).unwrap()
        };
        return Ok(module);
      }
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access module with id = {} now. The module have been removed on the Rust side.",
      self.identifier
    )))
  }

  fn as_mut(&mut self) -> napi::Result<&'static mut dyn Module> {
    let module = unsafe { self.module.as_mut() };
    if module.identifier() == self.identifier {
      return Ok(module);
    }

    Err(napi::Error::from_reason(format!(
      "Unable to access module with id = {} now. The module have been removed on the Rust side.",
      self.identifier
    )))
  }
}

#[napi]
impl JsModule {
  #[napi(getter)]
  pub fn context(&mut self) -> napi::Result<Either<String, ()>> {
    let module = self.as_ref()?;

    Ok(match module.get_context() {
      Some(ctx) => Either::A(ctx.to_string()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn original_source(&mut self) -> napi::Result<Either<JsCompatSource, ()>> {
    let module = self.as_ref()?;

    Ok(match module.original_source() {
      Some(source) => match source.to_js_compat_source().ok() {
        Some(s) => Either::A(s),
        None => Either::B(()),
      },
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn resource(&mut self) -> napi::Result<Either<&String, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(&normal_module.resource_resolved_data().resource),
      Err(_) => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn module_identifier(&mut self) -> napi::Result<&str> {
    let module = self.as_ref()?;

    Ok(module.identifier().as_str())
  }

  #[napi(getter)]
  pub fn name_for_condition(&mut self) -> napi::Result<Either<String, ()>> {
    let module = self.as_ref()?;

    Ok(match module.name_for_condition() {
      Some(s) => Either::A(s.to_string()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn request(&mut self) -> napi::Result<Either<&str, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.request()),
      Err(_) => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn user_request(&mut self) -> napi::Result<Either<&str, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.user_request()),
      Err(_) => Either::B(()),
    })
  }

  #[napi(setter)]
  pub fn set_user_request(&mut self, val: String) -> napi::Result<()> {
    let module: &mut dyn Module = self.as_mut()?;

    if let Ok(normal_module) = module.try_as_normal_module_mut() {
      *normal_module.user_request_mut() = val;
    }
    Ok(())
  }

  #[napi(getter)]
  pub fn raw_request(&mut self) -> napi::Result<Either<&str, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.raw_request()),
      Err(_) => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn factory_meta(&mut self) -> napi::Result<Either<JsFactoryMeta, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => match normal_module.factory_meta() {
        Some(meta) => Either::A(JsFactoryMeta {
          side_effect_free: meta.side_effect_free,
        }),
        None => Either::B(()),
      },
      Err(_) => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    let module = self.as_ref()?;

    Ok(module.module_type().as_str())
  }

  #[napi(getter)]
  pub fn layer(&mut self) -> napi::Result<Either<&String, ()>> {
    let module = self.as_ref()?;

    Ok(match module.get_layer() {
      Some(layer) => Either::A(layer),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn blocks(&mut self) -> napi::Result<Vec<JsDependenciesBlock>> {
    Ok(match self.compilation {
      Some(compilation) => {
        let module = self.as_ref()?;

        let blocks = module.get_blocks();
        blocks
          .iter()
          .cloned()
          .map(|block_id| JsDependenciesBlock::new(block_id, compilation.as_ptr()))
          .collect::<Vec<_>>()
      }
      None => {
        vec![]
      }
    })
  }

  #[napi(getter)]
  pub fn dependencies(&mut self) -> napi::Result<Vec<JsDependency>> {
    Ok(match self.compilation {
      Some(compilation) => {
        let compilation = unsafe { compilation.as_ref() };
        let module_graph = compilation.get_module_graph();
        let module = self.as_ref()?;
        let dependencies = module.get_dependencies();
        dependencies
          .iter()
          .filter_map(|dependency_id| {
            module_graph
              .dependency_by_id(dependency_id)
              .map(JsDependency::new)
          })
          .collect::<Vec<_>>()
      }
      None => {
        vec![]
      }
    })
  }

  #[napi]
  pub fn size(&mut self, ty: Option<String>) -> napi::Result<f64> {
    let module = self.as_ref()?;
    let compilation = self.compilation.map(|c| unsafe { c.as_ref() });

    let ty = ty.map(|s| SourceType::from(s.as_str()));
    Ok(module.size(ty.as_ref(), compilation))
  }

  #[napi(getter, ts_return_type = "JsModule[] | undefined")]
  pub fn modules(&mut self) -> napi::Result<Either<Vec<JsModuleWrapper>, ()>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_concatenated_module() {
      Ok(concatenated_module) => match self.compilation {
        Some(compilation) => {
          let compilation = unsafe { compilation.as_ref() };

          let inner_modules = concatenated_module
            .get_modules()
            .iter()
            .filter_map(|inner_module_info| {
              compilation
                .module_by_identifier(&inner_module_info.id)
                .map(|module| JsModuleWrapper::new(module.as_ref(), Some(compilation)))
            })
            .collect::<Vec<_>>();
          Either::A(inner_modules)
        }
        None => Either::A(vec![]),
      },
      Err(_) => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn use_source_map(&mut self) -> napi::Result<bool> {
    let module = self.as_ref()?;
    Ok(module.get_source_map_kind().source_map())
  }
}

type ModuleInstanceRefs = IdentifierMap<OneShotRef<ClassInstance<JsModule>>>;

type ModuleInstanceRefsByCompilationId = RefCell<HashMap<CompilationId, ModuleInstanceRefs>>;

thread_local! {
  static MODULE_INSTANCE_REFS: ModuleInstanceRefsByCompilationId = Default::default();

  static UNASSOCIATED_MODULE_INSTANCE_REFS: RefCell<ModuleInstanceRefs> = Default::default();
}

// The difference between JsModuleWrapper and JsModule is:
// JsModuleWrapper maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a JsModule from Rust to JS, you must use JsModuleWrapper instead.
pub struct JsModuleWrapper {
  identifier: ModuleIdentifier,
  module: NonNull<dyn Module>,
  compilation: Option<NonNull<Compilation>>,
}

unsafe impl Send for JsModuleWrapper {}

impl JsModuleWrapper {
  pub fn new(module: &dyn Module, compilation: Option<&Compilation>) -> Self {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    let identifier = module.identifier();

    #[allow(clippy::unwrap_used)]
    Self {
      identifier,
      module: NonNull::new(module as *const dyn Module as *mut dyn Module).unwrap(),
      compilation: compilation
        .map(|c| NonNull::new(c as *const Compilation as *mut Compilation).unwrap()),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
  }

  pub fn attach(&mut self, compilation: *const Compilation) {
    if self.compilation.is_none() {
      self.compilation = Some(
        #[allow(clippy::unwrap_used)]
        NonNull::new(compilation as *mut Compilation).unwrap(),
      );
    }
  }
}

impl ToNapiValue for JsModuleWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let module = unsafe { val.module.as_ref() };

    match val.compilation {
      Some(compilation_ptr) => MODULE_INSTANCE_REFS.with(|refs| {
        let compilation = unsafe { compilation_ptr.as_ref() };

        let mut refs_by_compilation_id = refs.borrow_mut();
        let entry = refs_by_compilation_id.entry(compilation.id());
        let refs = match entry {
          std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
          std::collections::hash_map::Entry::Vacant(entry) => {
            let refs = IdentifierMap::default();
            entry.insert(refs)
          }
        };

        UNASSOCIATED_MODULE_INSTANCE_REFS.with(|ref_cell| {
          let mut unassociated_refs = ref_cell.borrow_mut();
          if let Some(unassociated_ref) = unassociated_refs.remove(&module.identifier()) {
            let mut instance = unassociated_ref.from_napi_value()?;
            instance.as_mut().attach(compilation_ptr.as_ptr());

            let napi_value = ToNapiValue::to_napi_value(env, &unassociated_ref);
            refs.insert(module.identifier(), unassociated_ref);
            napi_value
          } else {
            match refs.entry(module.identifier()) {
              std::collections::hash_map::Entry::Occupied(entry) => {
                let r = entry.get();
                ToNapiValue::to_napi_value(env, r)
              }
              std::collections::hash_map::Entry::Vacant(entry) => {
                let instance: ClassInstance<JsModule> = JsModule {
                  identifier: val.identifier,
                  module: val.module,
                  compilation: Some(compilation_ptr),
                }
                .into_instance(Env::from_raw(env))?;
                let r = entry.insert(OneShotRef::new(env, instance)?);
                ToNapiValue::to_napi_value(env, r)
              }
            }
          }
        })
      }),
      None => UNASSOCIATED_MODULE_INSTANCE_REFS.with(|ref_cell| {
        let mut refs = ref_cell.borrow_mut();
        match refs.entry(module.identifier()) {
          std::collections::hash_map::Entry::Occupied(entry) => {
            let r = entry.get();

            let mut instance: ClassInstance<JsModule> = r.from_napi_value()?;
            if !std::ptr::addr_eq(instance.module.as_ptr(), val.module.as_ptr()) {
              instance.module = val.module;
            }
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(entry) => {
            let instance = JsModule {
              identifier: val.identifier,
              module: val.module,
              compilation: None,
            }
            .into_instance(Env::from_raw(env))?;
            let r = entry.insert(OneShotRef::new(env, instance)?);
            ToNapiValue::to_napi_value(env, r)
          }
        }
      }),
    }
  }
}

impl FromNapiValue for JsModuleWrapper {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let instance: ClassInstance<JsModule> = FromNapiValue::from_napi_value(env, napi_val)?;
    let module = instance.module;

    Ok(JsModuleWrapper {
      identifier: instance.identifier,
      #[allow(clippy::unwrap_used)]
      module: NonNull::new(module.as_ptr()).unwrap(),
      compilation: instance.compilation,
    })
  }
}

#[napi(object)]
pub struct JsExecuteModuleArg {
  pub entry: String,
  pub runtime_modules: Vec<String>,
  pub codegen_results: JsCodegenerationResults,
  pub id: u32,
}

#[derive(Default)]
#[napi(object)]
pub struct JsRuntimeModule {
  pub source: Option<JsCompatSource>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
}

#[napi(object)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  pub chunk: JsChunk,
}

type GenerateFn = ThreadsafeFunction<(), String>;

#[napi(object, object_to_js = false)]
pub struct JsAddingRuntimeModule {
  pub name: String,
  #[napi(ts_type = "() => String")]
  pub generator: GenerateFn,
  pub dependent_hash: bool,
  pub full_hash: bool,
  pub isolate: bool,
  pub stage: u32,
}

impl From<JsAddingRuntimeModule> for RuntimeModuleFromJs {
  fn from(value: JsAddingRuntimeModule) -> Self {
    Self {
      name: value.name,
      full_hash: value.full_hash,
      dependent_hash: value.dependent_hash,
      isolate: value.isolate,
      stage: RuntimeModuleStage::from(value.stage),
      generator: Arc::new(move || value.generator.blocking_call_with_sync(())),
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
      cached_generated_code: Default::default(),
    }
  }
}
