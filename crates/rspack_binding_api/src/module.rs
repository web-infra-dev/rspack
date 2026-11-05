#![allow(deprecated)]
use std::{
  any::TypeId,
  cell::{Cell, RefCell},
  ptr::NonNull,
  sync::{Arc, Weak},
};

use napi::JsString;
use napi_derive::napi;
use rspack_collections::Identifier;
use rspack_core::{
  BindingCell, BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, Compilation, CompilerId,
  FactoryMeta, LibIdentOptions, Module as _, ModuleIdentifier, RuntimeModuleCommon,
  RuntimeModuleStage, SourceType, internal, rspack_sources::Source,
};
use rspack_napi::{OneShotRef, WeakRef, napi::bindgen_prelude::*, string::JsStringExt};
use rspack_plugin_runtime::RuntimeModuleFromJs;

use crate::{
  COMPILER_REFERENCES,
  asset::AssetInfo,
  async_dependency_block::AsyncDependenciesBlockWrapper,
  build_info::BuildInfo,
  chunk::ChunkWrapper,
  codegen_result::JsCodegenerationResults,
  compiler_scoped_tsfn::CompilerScopedTsFnHandle,
  dependency::DependencyWrapper,
  module_bridge,
  source::{JsSourceFromJs, JsSourceToJs, JsSourceWithLazyMap},
  symbols,
};

#[napi(object)]
pub struct JsLibIdentOptions {
  pub context: String,
}

#[derive(Default)]
#[napi(object)]
pub struct JsFactoryMeta {
  pub side_effect_free: Option<bool>,
}

impl From<JsFactoryMeta> for FactoryMeta {
  fn from(value: JsFactoryMeta) -> Self {
    Self {
      side_effect_free: value.side_effect_free,
    }
  }
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_identifier<'env>(env: &'env Env, module: Object<'env>) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  env.create_string(module.identifier.as_str())
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_type<'env>(env: &'env Env, module: Object<'env>) -> Result<JsString<'env>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| env.create_string(module.module_type().as_str()))
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_context(module: Object<'_>) -> napi::Result<Either<String, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| {
    Ok(match module.get_context() {
      Some(ctx) => Either::A(ctx.to_string()),
      None => Either::B(()),
    })
  })
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_layer(module: Object<'_>) -> napi::Result<Either<String, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| {
    Ok(match module.get_layer() {
      Some(layer) => Either::A(layer.to_owned()),
      None => Either::B(()),
    })
  })
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_use_source_map(module: Object<'_>) -> napi::Result<bool> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| Ok(module.get_source_map_kind().source_map()))
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_use_simple_source_map(module: Object<'_>) -> napi::Result<bool> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| Ok(module.get_source_map_kind().simple_source_map()))
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_factory_meta(module: Object<'_>) -> napi::Result<JsFactoryMeta> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.with_ref(|_, module| {
    Ok(match module.as_normal_module() {
      Some(normal_module) => match normal_module.factory_meta() {
        Some(meta) => JsFactoryMeta {
          side_effect_free: meta.side_effect_free,
        },
        None => JsFactoryMeta {
          side_effect_free: None,
        },
      },
      None => JsFactoryMeta {
        side_effect_free: None,
      },
    })
  })
}

#[napi(ts_args_type = "module: Module, factoryMeta: JsFactoryMeta")]
pub fn module_set_factory_meta(
  module: Object<'_>,
  factory_meta: JsFactoryMeta,
) -> napi::Result<()> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  // napi-rs converts all arguments before we acquire mutable core access.
  module.as_mut()?.set_factory_meta(factory_meta.into());
  Ok(())
}

#[napi(ts_args_type = "module: Module", ts_return_type = "BuildInfo")]
pub fn module_get_build_info<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> napi::Result<Object<'env>> {
  let mut this = module;
  let raw_env = env.raw();
  let reference = unsafe { ModuleHandle::from_napi_ref(raw_env, this.raw())? };
  if let Some(r) = &*reference.build_info_ref.borrow() {
    return r.as_object(env);
  }
  let mut build_info =
    BuildInfo::new(reference.compiler_id, reference.identifier).get_jsobject(env)?;
  this.set_property(
    env.symbol_for(symbols::MODULE_BUILD_INFO_SYMBOL)?,
    build_info,
  )?;
  let r = WeakRef::new(raw_env, &mut build_info)?;
  let result = r.as_object(env);
  *reference.build_info_ref.borrow_mut() = Some(r);
  result
}

#[napi(ts_args_type = "module: Module, value: BuildInfo")]
pub fn module_set_build_info(
  env: &Env,
  module: Object<'_>,
  input_object: Object<'_>,
) -> napi::Result<()> {
  let mut this = module;
  let raw_env = env.raw();
  let reference = unsafe { ModuleHandle::from_napi_ref(raw_env, this.raw())? };
  let new_build_info = BuildInfo::new(reference.compiler_id, reference.identifier);
  let mut new_instance = new_build_info.get_jsobject(env)?;

  let names = input_object.get_all_property_names(
    napi::KeyCollectionMode::OwnOnly,
    napi::KeyFilter::AllProperties,
    napi::KeyConversion::KeepNumbers,
  )?;
  let names = Array::from_unknown(names.to_unknown())?;
  for index in 0..names.len() {
    if let Some(name) = names.get::<Unknown>(index)? {
      let name_clone = Object::from_raw(env.raw(), name.raw());
      let name_str = name_clone.coerce_to_string()?.into_string();
      // known build info properties
      if name_str != "assets" {
        let value = input_object.get_property::<Unknown, Unknown>(name)?;
        new_instance.set_property::<Unknown, Unknown>(name, value)?;
      }
    }
  }

  this.set_property(
    env.symbol_for(symbols::MODULE_BUILD_INFO_SYMBOL)?,
    new_instance,
  )?;
  *reference.build_info_ref.borrow_mut() = Some(WeakRef::new(raw_env, &mut new_instance)?);
  Ok(())
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_readable_identifier(module: Object<'_>) -> Result<String> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.readable_identifier()
}

#[napi(
  ts_args_type = "module: Module",
  ts_return_type = "JsSourceWithLazyMap | undefined"
)]
pub fn module_get_original_source<'env>(
  env: &'env Env,
  module: Object<'env>,
) -> Result<Either<Unknown<'env>, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.original_source(env)
}

#[napi(ts_args_type = "module: Module")]
pub fn module_get_name_for_condition(module: Object<'_>) -> Result<Either<String, ()>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.name_for_condition()
}

#[napi(
  ts_args_type = "module: Module",
  ts_return_type = "AsyncDependenciesBlock[]"
)]
pub fn module_get_blocks(module: Object<'_>) -> Result<Vec<AsyncDependenciesBlockWrapper>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.blocks()
}

#[napi(ts_args_type = "module: Module", ts_return_type = "Dependency[]")]
pub fn module_get_dependencies(module: Object<'_>) -> Result<Vec<DependencyWrapper>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.dependencies()
}

#[napi(ts_args_type = "module: Module, ty?: string | null")]
pub fn module_size(module: Object<'_>, ty: Option<String>) -> Result<f64> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.size(ty)
}

#[napi(ts_args_type = "module: Module, options: JsLibIdentOptions")]
pub fn module_lib_ident<'env>(
  env: &'env Env,
  module: Object<'env>,
  options: JsLibIdentOptions,
) -> Result<Option<JsString<'env>>> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.lib_ident(env, options)
}

#[napi(
  ts_args_type = "module: Module, filename: string, source: JsSourceFromJs, assetInfo?: AssetInfo | null"
)]
pub fn module_emit_file(
  env: &Env,
  module: Object<'_>,
  filename: String,
  source: JsSourceFromJs,
  info: Option<Object<'_>>,
) -> Result<()> {
  let module = unsafe { ModuleHandle::from_napi_ref(module.value().env, module.raw())? };
  module.emit_file(env, filename, source, info)
}

// ## Clarify Access Methods for napi Module to Rust Module
// Primary access: Query compilation.module_graph using module_identifier
// Fallback for unregistered modules: Access via raw pointer when modules aren't yet stored in compilation.module_graph (e.g., during loader execution phase)
//
// ## Clarify napi Module Lifecycle
// Created when accessed via JavaScript API
// Lifecycle ends upon triggering revoked_modules hook
//
// ## Behavior When napi Module Lifecycle Ends
// When JavaScript API accesses napi module properties after lifecycle termination:
// module_identifier query in compilation.module_graph returns undefined
// Raw pointer stored in napi module becomes None
// Throw an Error to the JavaScript side
struct OriginalSourceNapiRef {
  // Only retain a weak pointer for identity comparison. Holding another BoxSource here would
  // increment its Arc strong count and keep the native source alive after the Module replaces it.
  // The Weak pointer lets the Source and its owned buffers drop with the Module while also keeping
  // the old Arc allocation unavailable for pointer reuse, so identity comparisons remain reliable.
  related_source: Weak<dyn Source>,
  // Keep the converted JavaScript object alive and return that exact object on cache hits. The
  // reference is replaced on the next call after `module.source()` points to a different Source.
  napi_ref: OneShotRef,
}

pub struct ModuleHandle {
  pub(crate) identifier: ModuleIdentifier,
  pub(crate) ptr: Cell<Option<NonNull<dyn rspack_core::Module>>>,
  pub(crate) compiler_id: CompilerId,
  pub(crate) type_id: TypeId,
  original_source_ref: RefCell<Option<OriginalSourceNapiRef>>,
  pub(crate) build_info_ref: RefCell<Option<WeakRef>>,
}

impl ModuleHandle {
  pub(crate) fn new(value: &ModuleObject) -> Self {
    Self {
      identifier: value.identifier,
      compiler_id: value.compiler_id,
      type_id: value.type_id,
      ptr: Cell::new(value.ptr),
      original_source_ref: RefCell::new(None),
      build_info_ref: RefCell::new(None),
    }
  }

  // The receiver is rooted by the active N-API callback's handle scope.
  pub(crate) unsafe fn from_napi_ref<'a>(
    env: sys::napi_env,
    value: sys::napi_value,
  ) -> Result<&'a Self> {
    unsafe { module_bridge::unwrap(env, value) }
  }

  fn compiler_garbage_collected_error(&self) -> napi::Error {
    napi::Error::from_reason(format!(
      "Unable to access module with id = {} now. The Compiler has been garbage collected by JavaScript.",
      self.identifier
    ))
  }

  fn module_removed_error(&self) -> napi::Error {
    napi::Error::from_reason(format!(
      "Unable to access module with id = {} now. The module have been removed on the Rust side.",
      self.identifier
    ))
  }

  pub(crate) fn with_compilation<R>(
    &self,
    f: impl FnOnce(&Compilation) -> napi::Result<R>,
  ) -> napi::Result<R> {
    let compiler_reference = COMPILER_REFERENCES.with(|ref_cell| {
      let references = ref_cell.borrow();
      references.get(&self.compiler_id).cloned()
    });

    let Some(this) = compiler_reference
      .as_ref()
      .and_then(|compiler_reference| compiler_reference.get())
    else {
      return Err(self.compiler_garbage_collected_error());
    };

    f(&this.compiler.compilation)
  }

  pub(crate) fn with_ref<R>(
    &self,
    f: impl FnOnce(&Compilation, &dyn rspack_core::Module) -> napi::Result<R>,
  ) -> napi::Result<R> {
    let module_identifier = self.identifier;
    let module_ptr = self.ptr.get();
    self.with_compilation(|compilation| {
      if let Some(module) = compilation.module_by_identifier(&module_identifier) {
        f(compilation, module.as_ref())
      } else if let Some(ptr) = module_ptr {
        // SAFETY:
        // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
        // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
        f(compilation, unsafe { ptr.as_ref() })
      } else {
        Err(self.module_removed_error())
      }
    })
  }

  pub(crate) fn as_mut(&self) -> napi::Result<&'static mut dyn rspack_core::Module> {
    match self.ptr.get() {
      Some(mut ptr) => {
        // SAFETY:
        // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
        // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
        Ok(unsafe { ptr.as_mut() })
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to modify module with id = {}. Currently, you can only modify the module in the loader in Rspack.",
        self.identifier
      ))),
    }
  }
}

impl ModuleHandle {
  pub fn readable_identifier(&self) -> napi::Result<String> {
    self.with_ref(|compilation, module| {
      Ok(
        module
          .readable_identifier(&compilation.options.context)
          .to_string(),
      )
    })
  }

  pub fn original_source<'a>(&self, env: &'a Env) -> napi::Result<Either<Unknown<'a>, ()>> {
    let compiler_reference = COMPILER_REFERENCES.with(|ref_cell| {
      let references = ref_cell.borrow();
      references.get(&self.compiler_id).cloned()
    });

    let compilation = {
      let Some(this) = compiler_reference
        .as_ref()
        .and_then(|compiler_reference| compiler_reference.get())
      else {
        return Err(self.compiler_garbage_collected_error());
      };
      &this.compiler.compilation
    };

    let module = {
      if let Some(module) = compilation.module_by_identifier(&self.identifier) {
        module.as_ref()
      } else if let Some(ptr) = self.ptr.get() {
        // SAFETY:
        // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
        // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
        unsafe { ptr.as_ref() }
      } else {
        return Ok(Either::B(()));
      }
    };

    let Some(original_source) = module.source() else {
      *self.original_source_ref.borrow_mut() = None;
      return Ok(Either::B(()));
    };

    if let Some(OriginalSourceNapiRef {
      related_source,
      napi_ref,
    }) = &*self.original_source_ref.borrow()
      && (related_source.ptr_eq(&Arc::downgrade(original_source)))
    {
      return Ok(Either::A(ToNapiValue::into_unknown(napi_ref, env)?));
    }

    let binding = JsSourceWithLazyMap::to_js(env, original_source)?;
    let mut one_shot_ref = OneShotRef::new(env.raw(), binding)?;
    let result = ToNapiValue::into_unknown(&mut one_shot_ref, env)?;
    *self.original_source_ref.borrow_mut() = Some(OriginalSourceNapiRef {
      related_source: Arc::downgrade(original_source),
      napi_ref: one_shot_ref,
    });

    Ok(Either::A(result))
  }

  pub fn name_for_condition(&self) -> napi::Result<Either<String, ()>> {
    self.with_ref(|_, module| {
      Ok(match module.name_for_condition() {
        Some(s) => Either::A(s.to_string()),
        None => Either::B(()),
      })
    })
  }

  pub fn blocks(&self) -> napi::Result<Vec<AsyncDependenciesBlockWrapper>> {
    self.with_ref(|compilation, module| {
      let Some(module_graph) = compilation.try_get_module_graph() else {
        return Err(napi::Error::from_reason(
          "Module.blocks is unavailable while the module graph is under construction (e.g. inside a loader during compilation.rebuildModule)",
        ));
      };
      let blocks = module.get_blocks();
      Ok(
        blocks
          .iter()
          .filter_map(|block_id| {
            module_graph
              .block_by_id(block_id)
              .map(|block| AsyncDependenciesBlockWrapper::new(block, compilation))
          })
          .collect::<Vec<_>>(),
      )
    })
  }

  pub fn dependencies(&self) -> napi::Result<Vec<DependencyWrapper>> {
    self.with_ref(|compilation, module| {
      let Some(module_graph) = compilation.try_get_module_graph() else {
        return Err(napi::Error::from_reason(
          "Module.dependencies is unavailable while the module graph is under construction (e.g. inside a loader during compilation.rebuildModule)",
        ));
      };
      // Only expose dependencies that are still present in the current graph.
      let dependencies = module.get_dependency_ids();
      Ok(
        dependencies
          .filter_map(|dependency_id| {
            internal::try_dependency_ref_by_id(module_graph, dependency_id)
              .map(|dep| DependencyWrapper::from_graph(dep, compilation.id()))
          })
          .collect::<Vec<_>>(),
      )
    })
  }

  pub fn size(&self, ty: Option<String>) -> napi::Result<f64> {
    self.with_ref(|_, module| {
      let ty = ty.map(|s| SourceType::from(s.as_str()));
      Ok(module.size(ty.as_ref()))
    })
  }

  pub fn lib_ident<'a>(
    &self,
    env: &'a Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString<'a>>> {
    self.with_ref(|_, module| {
      Ok(
        match module.lib_ident(LibIdentOptions {
          context: &options.context,
        }) {
          Some(lib_ident) => Some(env.create_string(lib_ident.as_ref())?),
          None => None,
        },
      )
    })
  }

  pub fn emit_file(
    &self,
    env: &Env,
    filename: String,
    source: JsSourceFromJs,
    object: Option<Object>,
  ) -> napi::Result<()> {
    let asset_info = match object {
      Some(object) => {
        let js_info: AssetInfo =
          unsafe { FromNapiValue::from_napi_value(env.raw(), object.raw())? };
        let info: rspack_core::AssetInfo = js_info.into();
        let info = BindingCell::from(info);
        info.reflector().set_jsobject(env, object)?;
        info
      }
      None => Default::default(),
    };

    let source = source.try_into()?;
    let module = self.as_mut()?;
    module.build_info_mut().assets.insert(
      filename,
      rspack_core::CompilationAsset {
        source: Some(source),
        info: asset_info,
      },
    );
    Ok(())
  }
}

pub(crate) use module_bridge::with_module_instance;

// The difference between ModuleObject and ModuleHandle is:
// ModuleObject maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a Module from Rust to JS, you must use ModuleObject instead.
#[derive(Debug)]
pub struct ModuleObject {
  pub(crate) type_id: TypeId,
  pub(crate) identifier: ModuleIdentifier,
  pub(crate) ptr: Option<NonNull<dyn rspack_core::Module>>,
  pub(crate) compiler_id: CompilerId,
}

unsafe impl Send for ModuleObject {}
unsafe impl Sync for ModuleObject {}

impl ModuleObject {
  pub fn with_ref(module: &dyn rspack_core::Module, compiler_id: CompilerId) -> Self {
    Self {
      type_id: module.as_any().type_id(),
      identifier: module.identifier(),
      ptr: None,
      compiler_id,
    }
  }

  pub fn with_ptr(module_ptr: NonNull<dyn rspack_core::Module>, compiler_id: CompilerId) -> Self {
    let module = unsafe { module_ptr.as_ref() };

    Self {
      type_id: module.as_any().type_id(),
      identifier: module.identifier(),
      ptr: Some(module_ptr),
      compiler_id,
    }
  }

  pub fn cleanup_by_compiler_id(compiler_id: &CompilerId) {
    module_bridge::cleanup_compiler(compiler_id);
  }

  pub fn cleanup_by_module_identifiers(
    compiler_id: &CompilerId,
    revoked_modules: &[ModuleIdentifier],
  ) {
    module_bridge::cleanup_modules(compiler_id, revoked_modules);
  }

  pub fn identifier(&self) -> &ModuleIdentifier {
    &self.identifier
  }
}

impl ToNapiValue for ModuleObject {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let env = unsafe { Env::from_raw(env) };
    module_bridge::to_jsobject(&env, val)
  }
}

impl FromNapiValue for ModuleObject {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    let handle = unsafe { ModuleHandle::from_napi_ref(env, value)? };
    Ok(Self {
      type_id: handle.type_id,
      identifier: handle.identifier,
      ptr: handle.ptr.get(),
      compiler_id: handle.compiler_id,
    })
  }
}

pub struct ModuleObjectRef {
  pub(crate) identifier: ModuleIdentifier,
  pub(crate) compiler_id: CompilerId,
}

impl FromNapiValue for ModuleObjectRef {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    let handle = unsafe { ModuleHandle::from_napi_ref(env, value)? };
    Ok(Self {
      identifier: handle.identifier,
      compiler_id: handle.compiler_id,
    })
  }
}

#[napi(object, object_from_js = false)]
pub struct JsExecuteModuleArg {
  pub entry: String,
  pub runtime_modules: Vec<String>,
  pub codegen_results: JsCodegenerationResults,
  pub id: u32,
}

#[derive(Default)]
#[napi(object)]
pub struct JsRuntimeModule {
  #[napi(ts_type = "JsSource")]
  pub source: Option<JsSourceToJs>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
  pub stage: u32,
  pub isolate: bool,
}

#[napi(object, object_from_js = false)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
}

type GenerateFn = CompilerScopedTsFnHandle<(), String>;

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
      common: RuntimeModuleCommon {
        id: Identifier::from(value.name),
        ..Default::default()
      },
      full_hash: value.full_hash,
      dependent_hash: value.dependent_hash,
      isolate: value.isolate,
      stage: RuntimeModuleStage::from(value.stage),
      generator: Arc::new(move || {
        let generator = value.generator.clone();
        Box::pin(async move { generator.call_with_sync(()).await })
      }),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct JsBuildMeta {
  pub strict_esm_module: Option<bool>,
  pub has_top_level_await: Option<bool>,
  pub esm: Option<bool>,
  #[napi(ts_type = "undefined | 'unset' | 'default' | 'namespace' | 'flagged' | 'dynamic'")]
  pub exports_type: Option<String>,
  #[napi(ts_type = "undefined | 'false' | 'redirect' | 'redirect-warn'")]
  pub default_object: Option<String>,
  pub side_effect_free: Option<bool>,
}

impl From<JsBuildMeta> for BuildMeta {
  fn from(value: JsBuildMeta) -> Self {
    let JsBuildMeta {
      strict_esm_module,
      has_top_level_await,
      esm,
      default_object: raw_default_object,
      side_effect_free,
      exports_type: raw_exports_type,
    } = value;

    let default_object =
      raw_default_object.map(|raw_default_object| match raw_default_object.as_str() {
        "false" => BuildMetaDefaultObject::False,
        "redirect" => BuildMetaDefaultObject::Redirect,
        // Accept webpack/kebab form and the camelCase form previously written by
        // `#[serde(rename_all = "camelCase")]` on BuildMetaDefaultObject (#15010).
        "redirect-warn" | "redirectWarn" => BuildMetaDefaultObject::RedirectWarn,
        other => panic!("Unexpected buildMeta.defaultObject value: {other}"),
      });

    let exports_type = raw_exports_type
      .as_deref()
      .map(BuildMetaExportsType::from)
      .unwrap_or_default();

    BuildMeta {
      strict_esm_module,
      has_top_level_await,
      esm,
      is_css_module: None,
      need_id_in_concatenation: None,
      exports_type,
      default_object,
      side_effect_free,
    }
  }
}
