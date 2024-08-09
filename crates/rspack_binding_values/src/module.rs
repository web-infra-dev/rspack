use std::cell::RefCell;

use napi_derive::napi;
use rspack_collections::Identifier;
use rspack_core::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Compilation, CompilationId,
  CompilerModuleContext, DependenciesBlock, Module, ModuleGraph, ModuleIdentifier, SourceType,
};
use rspack_napi::{napi::bindgen_prelude::*, Ref};
use rustc_hash::FxHashMap as HashMap;
use sys::napi_env;

use super::{JsCompatSource, ToJsCompatSource};
use crate::{DependencyDTO, JsChunk, JsCodegenerationResults};

#[derive(Default)]
#[napi(object)]
pub struct JsFactoryMeta {
  pub side_effect_free: Option<bool>,
}

#[napi]
pub struct DependenciesBlockDTO {
  block_id: AsyncDependenciesBlockIdentifier,
  compilation: &'static Compilation,
}

impl DependenciesBlockDTO {
  pub fn new(
    block_id: AsyncDependenciesBlockIdentifier,
    compilation: &'static Compilation,
  ) -> Self {
    Self {
      block_id,
      compilation,
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
impl DependenciesBlockDTO {
  #[napi(getter)]
  pub fn dependencies(&self) -> Vec<DependencyDTO> {
    let module_graph = self.compilation.get_module_graph();
    let block = self.block(&module_graph);
    block
      .get_dependencies()
      .iter()
      .cloned()
      .map(|dep_id| DependencyDTO::new(dep_id, self.compilation))
      .collect::<Vec<_>>()
  }

  #[napi(getter)]
  pub fn blocks(&self) -> Vec<DependenciesBlockDTO> {
    let module_graph = self.compilation.get_module_graph();
    let block = self.block(&module_graph);
    let blocks = block.get_blocks();
    blocks
      .iter()
      .cloned()
      .map(|block_id| DependenciesBlockDTO::new(block_id, self.compilation))
      .collect::<Vec<_>>()
  }
}

#[napi]
pub struct ModuleDTO {
  pub(crate) module_id: ModuleIdentifier,
  pub(crate) compilation: &'static Compilation,
}

impl ModuleDTO {
  pub fn new(module_id: ModuleIdentifier, compilation: &'static Compilation) -> Self {
    Self {
      module_id,
      compilation,
    }
  }

  fn module(&self) -> &dyn Module {
    self
      .compilation
      .module_by_identifier(&self.module_id)
      .unwrap_or_else(|| {
        panic!(
          "Cannot find module with id = {}. It might have been removed on the Rust side.",
          self.module_id
        )
      })
      .as_ref()
  }
}

#[napi]
impl ModuleDTO {
  #[napi(getter)]
  pub fn context(&self) -> Either<String, ()> {
    let module = self.module();
    match module.get_context() {
      Some(ctx) => Either::A(ctx.to_string()),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn original_source(&self) -> Either<JsCompatSource, ()> {
    let module = self.module();
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
    let module = self.module();
    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.resource_resolved_data().resource.to_string()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn module_identifier(&self) -> &str {
    let module = self.module();
    module.identifier().as_str()
  }

  #[napi(getter)]
  pub fn name_for_condition(&self) -> Either<String, ()> {
    let module = self.module();
    match module.name_for_condition() {
      Some(s) => Either::A(s.to_string()),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn request(&self) -> Either<&str, ()> {
    let module = self.module();
    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn user_request(&self) -> Either<&str, ()> {
    let module = self.module();
    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.user_request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn raw_request(&self) -> Either<&str, ()> {
    let module = self.module();
    match module.try_as_normal_module() {
      Ok(normal_module) => Either::A(normal_module.raw_request()),
      Err(_) => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn factory_meta(&self) -> Either<JsFactoryMeta, ()> {
    let module = self.module();
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
    let module = self.module();
    module.module_type().as_str()
  }

  #[napi(getter)]
  pub fn layer(&self) -> Either<&String, ()> {
    let module = self.module();
    match module.get_layer() {
      Some(layer) => Either::A(layer),
      None => Either::B(()),
    }
  }

  #[napi(getter)]
  pub fn blocks(&self) -> Vec<DependenciesBlockDTO> {
    let module = self.module();
    let blocks = module.get_blocks();
    blocks
      .iter()
      .cloned()
      .map(|block_id| DependenciesBlockDTO::new(block_id, self.compilation))
      .collect::<Vec<_>>()
  }

  #[napi]
  pub fn size(&self, ty: Option<String>) -> f64 {
    let module = self.module();
    let ty = ty.map(|s| SourceType::from(s.as_str()));
    module.size(ty.as_ref(), self.compilation)
  }
}

type ModuleInstanceRefs = HashMap<Identifier, (Ref, napi_env)>;

#[derive(Default)]
struct ModuleInstanceRefsByCompilationId(RefCell<HashMap<CompilationId, ModuleInstanceRefs>>);

impl Drop for ModuleInstanceRefsByCompilationId {
  fn drop(&mut self) {
    let mut refs_by_compilation_id = self.0.borrow_mut();
    for (_, mut refs) in refs_by_compilation_id.drain() {
      for (_, (mut r, env)) in refs.drain() {
        let _ = r.unref(env);
      }
    }
  }
}

thread_local! {
  static MODULE_INSTANCE_REFS: ModuleInstanceRefsByCompilationId = Default::default();
}

// The difference between ModuleDTOWrapper and ModuleDTO is:
// ModuleDTOWrapper maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a ModuleDTO from Rust to JS, you must use ModuleDTOWrapper instead.
pub struct ModuleDTOWrapper {
  pub module_id: ModuleIdentifier,
  pub compilation: &'static Compilation,
}

impl ModuleDTOWrapper {
  pub fn new(module_id: ModuleIdentifier, compilation: &Compilation) -> Self {
    // SAFETY:
    // 1. `Compiler` is stored on the heap and pinned in binding crate.
    // 2. `Compilation` outlives `JsCompilation` and `Compiler` outlives `Compilation`.
    // 3. `JsCompilation` was replaced everytime a new `Compilation` was created before getting accessed.
    let compilation = unsafe {
      std::mem::transmute::<&rspack_core::Compilation, &'static rspack_core::Compilation>(
        compilation,
      )
    };
    Self {
      module_id,
      compilation,
    }
  }

  pub fn cleanup(compilation_id: CompilationId) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.0.borrow_mut();
      if let Some(mut refs) = refs_by_compilation_id.remove(&compilation_id) {
        for (_, (mut r, env)) in refs.drain() {
          let _ = r.unref(env);
        }
      }
    });
  }
}

impl ToNapiValue for ModuleDTOWrapper {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compilation_id = refs.0.borrow_mut();
      let entry = refs_by_compilation_id.entry(val.compilation.id());
      let refs = match entry {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
          let refs = HashMap::default();
          entry.insert(refs)
        }
      };
      match refs.entry(val.module_id) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          let r = entry.get();
          ToNapiValue::to_napi_value(env, &r.0)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let instance =
            ModuleDTO::new(val.module_id, val.compilation).into_instance(Env::from_raw(env))?;
          let napi_value = ToNapiValue::to_napi_value(env, instance)?;
          let r = Ref::new(env, napi_value, 1)?;
          let r = entry.insert((r, env));
          ToNapiValue::to_napi_value(env, &r.0)
        }
      }
    })
  }
}

#[derive(Default)]
#[napi(object)]
pub struct JsModule {
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
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = || {
      self
        .original_source()
        .and_then(|source| source.to_js_compat_source().ok())
    };
    let name_for_condition = || self.name_for_condition().map(|s| s.to_string());
    let module_identifier = || self.identifier().to_string();
    let context = || self.get_context().map(|c| c.to_string());
    let module_type = || self.module_type().to_string();
    let module_layer = || self.get_layer().cloned();

    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        context: context(),
        original_source: original_source(),
        resource: Some(normal_module.resource_resolved_data().resource.to_string()),
        r#type: module_type(),
        layer: module_layer(),
        module_identifier: module_identifier(),
        name_for_condition: name_for_condition(),
        request: Some(normal_module.request().to_string()),
        user_request: Some(normal_module.user_request().to_string()),
        raw_request: Some(normal_module.raw_request().to_string()),
        factory_meta: normal_module
          .factory_meta()
          .map(|factory_meta| JsFactoryMeta {
            side_effect_free: factory_meta.side_effect_free,
          }),
      })
      .or_else(|_| {
        self.try_as_raw_module().map(|_| JsModule {
          context: context(),
          r#type: module_type(),
          layer: module_layer(),
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_context_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          r#type: module_type(),
          layer: module_layer(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_external_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          r#type: module_type(),
          layer: module_layer(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        Ok(JsModule {
          context: context(),
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          layer: module_layer(),
          r#type: module_type(),
          ..Default::default()
        })
      })
  }
}

impl ToJsModule for CompilerModuleContext {
  fn to_js_module(&self) -> Result<JsModule> {
    let module = JsModule {
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
