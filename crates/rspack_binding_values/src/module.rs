use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use napi_derive::napi;
use rspack_collections::IdentifierMap;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, CompilationId,
  CompilerModuleContext, DependenciesBlock, Module, ModuleGraph, RuntimeModuleStage, SourceType,
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
}

#[napi]
impl JsModule {
  #[napi(getter)]
  pub fn context(&self) -> Either<String, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.get_context() {
      Some(ctx) => Either::A(ctx.to_string()),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn original_source(&self) -> Either<JsCompatSource, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.original_source() {
      Some(source) => match source.to_js_compat_source().ok() {
        Some(s) => Either::A(s),
        None => Either::B(()),
      },
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn resource(&self) -> Either<String, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.resource_resolved_data().resource.to_string()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn module_identifier(&self) -> &str {
    let module = unsafe { self.module.as_ref() };

    module.identifier().as_str()
  }

  #[napi(getter)]
  pub fn name_for_condition(&self) -> Either<String, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.name_for_condition() {
      Some(s) => Either::A(s.to_string()),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn request(&self) -> Either<&str, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn user_request(&self) -> Either<&str, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.user_request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn raw_request(&self) -> Either<&str, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.raw_request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn factory_meta(&self) -> Either<JsFactoryMeta, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_normal_module() {
      Ok(normal_module) => match normal_module.factory_meta() {
        Some(meta) => Either::A(JsFactoryMeta {
          side_effect_free: meta.side_effect_free,
        }),
        None => Either::B(()),
      },
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn get_type(&self) -> &str {
    let module = unsafe { self.module.as_ref() };

    module.module_type().as_str()
  }

  #[napi(getter)]
  pub fn layer(&self) -> Either<&String, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.get_layer() {
      Some(layer) => Either::A(layer),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn blocks(&self) -> Vec<JsDependenciesBlock> {
    let module = unsafe { self.module.as_ref() };

    match self.compilation {
      Some(compilation) => {
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
    }
  }

  #[napi]
  pub fn size(&self, ty: Option<String>) -> f64 {
    let module = unsafe { self.module.as_ref() };

    match self.compilation {
      Some(compilation) => {
        let compilation = unsafe { compilation.as_ref() };

        let ty = ty.map(|s| SourceType::from(s.as_str()));
        module.size(ty.as_ref(), compilation)
      }
      None => 0f64, // TODO fix
    }
  }

  #[napi(getter, ts_return_type = "JsModule[] | undefined")]
  pub fn modules(&self) -> Either<Vec<JsModuleWrapper>, ()> {
    let module = unsafe { self.module.as_ref() };

    match module.try_as_concatenated_module() {
      Ok(concatenated_module) => match self.compilation {
        Some(compilation_ptr) => {
          let compilation = unsafe { compilation_ptr.as_ref() };

          let inner_modules = concatenated_module
            .get_modules()
            .iter()
            .filter_map(|inner_module_info| {
              compilation
                .module_by_identifier(&inner_module_info.id)
                .map(|module| JsModuleWrapper::new(module.as_ref(), Some(compilation_ptr.as_ptr())))
            })
            .collect::<Vec<_>>();
          Either::A(inner_modules)
        }
        None => Either::A(vec![]),
      },
      Err(_) => Either::B(()),
    }
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
  module: NonNull<dyn Module>,
  compilation: Option<NonNull<Compilation>>,
}

unsafe impl Send for JsModuleWrapper {}

impl JsModuleWrapper {
  pub fn new(module: *const dyn Module, compilation: Option<*const Compilation>) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      module: NonNull::new(module as *mut dyn Module).unwrap(),
      compilation: compilation.map(|c| NonNull::new(c as *mut Compilation).unwrap()),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.borrow_mut();
      refs_by_compilation_id.remove(&compilation_id)
    });
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

            if !std::ptr::addr_eq(instance.module.as_ptr(), val.module.as_ptr()) {
              instance.module = val.module
            }
            let napi_value = ToNapiValue::to_napi_value(env, &unassociated_ref);
            refs.insert(module.identifier(), unassociated_ref);
            napi_value
          } else {
            match refs.entry(module.identifier()) {
              std::collections::hash_map::Entry::Occupied(entry) => {
                let r = entry.get();

                let mut instance: ClassInstance<JsModule> = r.from_napi_value()?;
                if !std::ptr::addr_eq(instance.module.as_ptr(), val.module.as_ptr()) {
                  instance.module = val.module
                }
                ToNapiValue::to_napi_value(env, r)
              }
              std::collections::hash_map::Entry::Vacant(entry) => {
                let instance: ClassInstance<JsModule> = JsModule {
                  module: val.module,
                  compilation: Some(compilation_ptr),
                }
                .into_instance(Env::from_raw(env))?;
                let r = entry.insert(OneShotRef::new(env, instance)?);

                let mut instance: ClassInstance<JsModule> = r.from_napi_value()?;
                if !std::ptr::addr_eq(instance.module.as_ptr(), val.module.as_ptr()) {
                  instance.module = val.module
                }
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
              instance.module = val.module
            }
            ToNapiValue::to_napi_value(env, r)
          }
          std::collections::hash_map::Entry::Vacant(entry) => {
            let instance = JsModule {
              module: val.module,
              compilation: None,
            }
            .into_instance(Env::from_raw(env))?;
            let r = entry.insert(OneShotRef::new(env, instance)?);

            let mut instance: ClassInstance<JsModule> = r.from_napi_value()?;
            if !std::ptr::addr_eq(instance.module.as_ptr(), val.module.as_ptr()) {
              instance.module = val.module
            }
            ToNapiValue::to_napi_value(env, r)
          }
        }
      }),
    }
  }
}

#[derive(Default)]
#[napi(object)]
pub struct JsCompilerModuleContext {
  pub context: Option<String>,
  pub original_source: Option<JsCompatSource>,
  pub resource: Option<String>,
  pub module_identifier: String,
  pub name_for_condition: Option<String>,
  pub request: Option<String>,
  pub user_request: Option<String>,
  pub raw_request: Option<String>,
  pub factory_meta: Option<JsFactoryMeta>,
  pub r#type: String,
  pub layer: Option<String>,
  pub use_source_map: Option<bool>,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsCompilerModuleContext>;
}

impl ToJsModule for CompilerModuleContext {
  fn to_js_module(&self) -> Result<JsCompilerModuleContext> {
    let module = JsCompilerModuleContext {
      context: self.context.as_ref().map(|c| c.to_string()),
      module_identifier: self.module_identifier.to_string(),
      name_for_condition: self.name_for_condition.clone(),
      r#type: self.r#type.to_string(),
      layer: self.layer.clone(),
      resource: self.resource_data.as_ref().map(|r| r.resource.to_string()),
      original_source: None,
      request: self.request.clone(),
      user_request: self.user_request.clone(),
      raw_request: self.raw_request.clone(),
      factory_meta: self.factory_meta.as_ref().map(|fm| JsFactoryMeta {
        side_effect_free: fm.side_effect_free,
      }),
      use_source_map: Some(self.use_source_map),
    };
    Ok(module)
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
  pub cacheable: bool,
  pub isolate: bool,
  pub stage: u32,
}

impl From<JsAddingRuntimeModule> for RuntimeModuleFromJs {
  fn from(value: JsAddingRuntimeModule) -> Self {
    Self {
      name: value.name,
      cacheable: value.cacheable,
      isolate: value.isolate,
      stage: RuntimeModuleStage::from(value.stage),
      generator: Arc::new(move || value.generator.blocking_call_with_sync(())),
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
      cached_generated_code: Default::default(),
    }
  }
}
