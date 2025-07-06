#![allow(deprecated)]
use std::{any::TypeId, cell::RefCell, ptr::NonNull, sync::Arc};

use napi::{CallContext, JsObject, JsString, JsSymbol, NapiRaw};
use napi_derive::napi;
use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_core::{
  BindingCell, BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, Compilation, CompilerId,
  FactoryMeta, LibIdentOptions, Module as _, ModuleIdentifier, RuntimeModuleStage, SourceType,
};
use rspack_napi::{
  napi::bindgen_prelude::*, string::JsStringExt, threadsafe_function::ThreadsafeFunction,
  OneShotInstanceRef, WeakRef,
};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;

use super::JsCompatSourceOwned;
use crate::{
  define_symbols, AssetInfo, AsyncDependenciesBlockWrapper, BuildInfo, ChunkWrapper,
  ConcatenatedModule, ContextModule, DependencyWrapper, ExternalModule, JsCodegenerationResults,
  JsCompatSource, JsCompiler, NormalModule, ToJsCompatSource, COMPILER_REFERENCES,
};

define_symbols! {
  MODULE_IDENTIFIER_SYMBOL => "MODULE_IDENTIFIER_SYMBOL",
  MODULE_BUILD_INFO_SYMBOL => "MODULE_BUILD_INFO_SYMBOL",
  COMPILATION_HOOKS_MAP_SYMBOL => "COMPILATION_HOOKS_MAP_SYMBOL",
}

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
#[napi]
pub struct Module {
  pub(crate) identifier: ModuleIdentifier,
  ptr: Option<NonNull<dyn rspack_core::Module>>,
  compiler_id: CompilerId,
  compiler_reference: WeakReference<JsCompiler>,
  pub(crate) build_info_ref: Option<WeakRef>,
}

impl Module {
  pub(crate) fn custom_into_instance(self, env: &Env) -> napi::Result<ClassInstance<Self>> {
    let mut instance = self.into_instance(env)?;
    let mut object = instance.as_object(env);
    let (_, module) = (*instance).as_ref()?;

    #[js_function]
    fn context_getter(ctx: CallContext) -> napi::Result<Either<String, ()>> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let (_, module) = wrapped_value.as_ref()?;
      Ok(match module.get_context() {
        Some(ctx) => Either::A(ctx.to_string()),
        None => Either::B(()),
      })
    }

    #[js_function]
    fn layer_getter(ctx: CallContext) -> napi::Result<Either<&String, ()>> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let (_, module) = wrapped_value.as_ref()?;
      Ok(match module.get_layer() {
        Some(layer) => Either::A(layer),
        None => Either::B(()),
      })
    }

    #[js_function]
    fn use_source_map_getter(ctx: CallContext) -> napi::Result<bool> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let (_, module) = wrapped_value.as_ref()?;
      Ok(module.get_source_map_kind().source_map())
    }

    #[js_function]
    fn use_simple_source_map_getter(ctx: CallContext) -> napi::Result<bool> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let (_, module) = wrapped_value.as_ref()?;
      Ok(module.get_source_map_kind().source_map())
    }

    #[js_function]
    fn factory_meta_getter(ctx: CallContext) -> napi::Result<JsFactoryMeta> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let (_, module) = wrapped_value.as_ref()?;
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
    }

    #[js_function(1)]
    fn factory_meta_setter(ctx: CallContext) -> napi::Result<()> {
      let this = ctx.this_unchecked::<JsObject>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = wrapped_value.as_mut()?;
      let factory_meta = ctx.get::<JsFactoryMeta>(0)?;
      module.set_factory_meta(factory_meta.into());
      Ok(())
    }

    #[js_function]
    fn build_info_getter(ctx: CallContext) -> napi::Result<Object> {
      let mut this = ctx.this_unchecked::<JsObject>();
      let env = ctx.env;
      let raw_env = env.raw();
      let mut reference: Reference<Module> =
        unsafe { Reference::from_napi_value(raw_env, this.raw())? };
      if let Some(r) = &reference.build_info_ref {
        return r.as_object(env);
      }
      let mut build_info = BuildInfo::new(reference.downgrade()).get_jsobject(env)?;
      MODULE_BUILD_INFO_SYMBOL.with(|once_cell| {
        let sym = unsafe {
          #[allow(clippy::unwrap_used)]
          let napi_val = ToNapiValue::to_napi_value(env.raw(), once_cell.get().unwrap())?;
          JsSymbol::from_napi_value(env.raw(), napi_val)
        };
        this.set_property(sym, build_info)
      })?;
      let r = WeakRef::new(raw_env, &mut build_info)?;
      let result = r.as_object(env);
      reference.build_info_ref = Some(r);
      result
    }

    #[js_function(1)]
    fn build_info_setter(ctx: CallContext) -> napi::Result<()> {
      let mut this = ctx.this_unchecked::<JsObject>();
      let input_object = ctx.get::<Object>(0)?;
      let env = ctx.env;
      let raw_env = env.raw();
      let mut reference: Reference<Module> =
        unsafe { Reference::from_napi_value(raw_env, this.raw())? };
      let new_build_info = BuildInfo::new(reference.downgrade());
      let mut new_instrance = new_build_info.get_jsobject(env)?;

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
          if name_str == "assets" {
            // TODO: Currently, setting assets is not supported.
            continue;
          } else {
            let value = input_object.get_property::<Unknown, Unknown>(name)?;
            new_instrance.set_property::<Unknown, Unknown>(name, value)?;
          }
        }
      }

      MODULE_BUILD_INFO_SYMBOL.with(|once_cell| {
        let sym = unsafe {
          #[allow(clippy::unwrap_used)]
          let napi_val = ToNapiValue::to_napi_value(env.raw(), once_cell.get().unwrap())?;
          JsSymbol::from_napi_value(env.raw(), napi_val)
        };
        this.set_property(sym, new_instrance)
      })?;
      reference.build_info_ref = Some(WeakRef::new(raw_env, &mut new_instrance)?);
      Ok(())
    }

    let mut properties = vec![
      Property::new()
        .with_utf8_name("type")?
        .with_value(&env.create_string(module.module_type().as_str())?),
      Property::new()
        .with_utf8_name("context")?
        .with_getter(context_getter),
      Property::new()
        .with_utf8_name("layer")?
        .with_getter(layer_getter),
      Property::new()
        .with_utf8_name("useSourceMap")?
        .with_getter(use_source_map_getter),
      Property::new()
        .with_utf8_name("useSimpleSourceMap")?
        .with_getter(use_simple_source_map_getter),
      Property::new()
        .with_utf8_name("factoryMeta")?
        .with_getter(factory_meta_getter)
        .with_setter(factory_meta_setter),
      Property::new()
        .with_utf8_name("buildInfo")?
        .with_getter(build_info_getter)
        .with_setter(build_info_setter),
      Property::new()
        .with_utf8_name("buildMeta")?
        .with_value(&Object::new(env)?),
    ];

    MODULE_IDENTIFIER_SYMBOL.with(|once_cell| {
      let identifier = env.create_string(module.identifier().as_str())?;
      #[allow(clippy::unwrap_used)]
      let symbol = once_cell.get().unwrap();
      properties.push(
        Property::new()
          .with_name(env, symbol)?
          .with_value(&identifier)
          .with_property_attributes(PropertyAttributes::Configurable),
      );
      Ok::<(), napi::Error>(())
    })?;

    object.define_properties(&properties)?;

    Ok(instance)
  }

  pub(crate) fn as_ref(&mut self) -> napi::Result<(&Compilation, &dyn rspack_core::Module)> {
    match self.compiler_reference.get() {
      Some(this) => {
        let compilation = &this.compiler.compilation;
        if let Some(module) = compilation.module_by_identifier(&self.identifier) {
          Ok((compilation, module.as_ref()))
        } else if let Some(ptr) = self.ptr {
          // SAFETY:
          // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
          // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
          Ok((compilation, unsafe { ptr.as_ref() }))
        } else {
          Err(napi::Error::from_reason(format!(
            "Unable to access module with id = {} now. The module have been removed on the Rust side.",
            self.identifier
          )))
        }
      }
      None => Err(napi::Error::from_reason(format!(
        "Unable to access module with id = {} now. The Compiler has been garbage collected by JavaScript.",
        self.identifier
      ))),
    }
  }

  pub(crate) fn as_mut(&mut self) -> napi::Result<&'static mut dyn rspack_core::Module> {
    match self.ptr.as_mut() {
      Some(ptr) => {
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

#[napi]
impl Module {
  #[napi]
  pub fn readable_identifier(&mut self) -> napi::Result<String> {
    let (_, module) = self.as_ref()?;
    Ok(
      module
        .get_context()
        .map(|ctx| module.readable_identifier(ctx.as_ref()).to_string())
        .unwrap_or_default(),
    )
  }

  #[napi(js_name = "_originalSource", enumerable = false)]
  pub fn original_source(&mut self, env: &Env) -> napi::Result<Either<JsCompatSource, ()>> {
    let (_, module) = self.as_ref()?;

    Ok(match module.source() {
      Some(source) => match source.to_js_compat_source(env).ok() {
        Some(s) => Either::A(s),
        None => Either::B(()),
      },
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn name_for_condition(&mut self) -> napi::Result<Either<String, ()>> {
    let (_, module) = self.as_ref()?;

    Ok(match module.name_for_condition() {
      Some(s) => Either::A(s.to_string()),
      None => Either::B(()),
    })
  }

  #[napi(
    getter,
    ts_return_type = "AsyncDependenciesBlock[]",
    enumerable = false
  )]
  pub fn blocks(&mut self) -> napi::Result<Vec<AsyncDependenciesBlockWrapper>> {
    let (compilation, module) = self.as_ref()?;

    let module_graph = compilation.get_module_graph();
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
  }

  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&mut self) -> napi::Result<Vec<DependencyWrapper>> {
    let (compilation, module) = self.as_ref()?;

    let module_graph = compilation.get_module_graph();
    let dependencies = module.get_dependencies();
    Ok(
      dependencies
        .iter()
        .filter_map(|dependency_id| {
          module_graph
            .dependency_by_id(dependency_id)
            .map(|dep| DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation)))
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi]
  pub fn size(&mut self, ty: Option<String>) -> napi::Result<f64> {
    let (compilation, module) = self.as_ref()?;

    let ty = ty.map(|s| SourceType::from(s.as_str()));
    Ok(module.size(ty.as_ref(), Some(compilation)))
  }

  #[napi]
  pub fn lib_ident<'a>(
    &mut self,
    env: &'a Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString<'a>>> {
    let (_, module) = self.as_ref()?;
    Ok(
      match module.lib_ident(LibIdentOptions {
        context: &options.context,
      }) {
        Some(lib_ident) => Some(env.create_string(lib_ident.as_ref())?),
        None => None,
      },
    )
  }

  #[napi(
    js_name = "_emitFile",
    enumerable = false,
    ts_args_type = "filename: string, source: JsCompatSource, assetInfo?: AssetInfo | undefined | null"
  )]
  pub fn emit_file(
    &mut self,
    env: &Env,
    filename: String,
    source: JsCompatSource,
    object: Option<Object>,
  ) -> napi::Result<()> {
    let module = self.as_mut()?;

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

    module.build_info_mut().assets.insert(
      filename,
      rspack_core::CompilationAsset {
        source: Some(source.into()),
        info: asset_info,
      },
    );
    Ok(())
  }
}

type ModuleInstanceNapiRef = Either5<
  OneShotInstanceRef<NormalModule>,
  OneShotInstanceRef<ConcatenatedModule>,
  OneShotInstanceRef<ContextModule>,
  OneShotInstanceRef<ExternalModule>,
  OneShotInstanceRef<Module>,
>;

type ModuleInstanceRef<'a> = Either5<
  &'a NormalModule,
  &'a ConcatenatedModule,
  &'a ContextModule,
  &'a ExternalModule,
  &'a Module,
>;

type ModuleInstanceMutRef<'a> = Either5<
  &'a mut NormalModule,
  &'a mut ConcatenatedModule,
  &'a mut ContextModule,
  &'a mut ExternalModule,
  &'a mut Module,
>;

type ModuleInstanceNapiRefs = IdentifierMap<ModuleInstanceNapiRef>;

type ModuleInstanceNapiRefsByCompilerId = RefCell<UkeyMap<CompilerId, ModuleInstanceNapiRefs>>;

thread_local! {
  static MODULE_INSTANCE_REFS: ModuleInstanceNapiRefsByCompilerId = Default::default();
}

// The difference between ModuleObject and Module is:
// ModuleObject maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a Module from Rust to JS, you must use ModuleObject instead.
#[derive(Debug)]
pub struct ModuleObject {
  type_id: TypeId,
  identifier: ModuleIdentifier,
  ptr: Option<NonNull<dyn rspack_core::Module>>,
  compiler_id: CompilerId,
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
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      refs_by_compiler_id.remove(compiler_id)
    });
  }

  pub fn cleanup_by_module_identifiers(
    compiler_id: &CompilerId,
    revoked_modules: &[ModuleIdentifier],
  ) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      for module_identifier in revoked_modules {
        if let Some(refs) = refs_by_compiler_id.get_mut(compiler_id) {
          if let Some(r) = refs.remove(module_identifier) {
            match r {
              Either5::A(mut normal_module) => normal_module.module.ptr = None,
              Either5::B(mut concatenated_module) => concatenated_module.module.ptr = None,
              Either5::C(mut context_module) => context_module.module.ptr = None,
              Either5::D(mut external_module) => external_module.module.ptr = None,
              Either5::E(mut module) => module.ptr = None,
            }
          }
        }
      }
    });
  }

  pub fn identifier(&self) -> &ModuleIdentifier {
    &self.identifier
  }
}

impl ToNapiValue for ModuleObject {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      let entry = refs_by_compiler_id.entry(val.compiler_id);
      let refs = match entry {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => {
          let refs = IdentifierMap::default();
          entry.insert(refs)
        }
      };

      match refs.entry(val.identifier) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
          let instance_ref = entry.get_mut();
          let instance = match instance_ref {
            Either5::A(normal_module) => &mut normal_module.module,
            Either5::B(concatenated_module) => &mut concatenated_module.module,
            Either5::C(context_module) => &mut context_module.module,
            Either5::D(external_module) => &mut external_module.module,
            Either5::E(module) => &mut **module,
          };
          instance.ptr = val.ptr;
          match instance_ref {
            Either5::A(r) => ToNapiValue::to_napi_value(env, r),
            Either5::B(r) => ToNapiValue::to_napi_value(env, r),
            Either5::C(r) => ToNapiValue::to_napi_value(env, r),
            Either5::D(r) =>ToNapiValue::to_napi_value(env, r),
            Either5::E(r) => ToNapiValue::to_napi_value(env, r),
          }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          match COMPILER_REFERENCES.with(|ref_cell| {
            let references = ref_cell.borrow();
            references.get(&val.compiler_id).cloned()
          }) {
            Some(compiler_reference) => {
              let js_module = Module {
                identifier: val.identifier,
                compiler_id: val.compiler_id,
                ptr: val.ptr,
                compiler_reference,
                build_info_ref: Default::default(),
              };
              let env_wrapper = Env::from_raw(env);

              let instance_ref = if val.type_id == TypeId::of::<rspack_core::NormalModule>() {
                let instance = NormalModule::new(js_module).custom_into_instance(&env_wrapper)?;
                entry.insert(Either5::A(OneShotInstanceRef::from_instance(env, instance)?))
              } else if val.type_id == TypeId::of::<rspack_core::ConcatenatedModule>() {
                let instance = ConcatenatedModule { module: js_module }.custom_into_instance(&env_wrapper)?;
                entry.insert(Either5::B(OneShotInstanceRef::from_instance(env, instance)?))
              } else if val.type_id == TypeId::of::<rspack_core::ContextModule>() {
                let instance = ContextModule { module: js_module }.custom_into_instance(&env_wrapper)?;
                entry.insert(Either5::C(OneShotInstanceRef::from_instance(env, instance)?))
              } else if val.type_id == TypeId::of::<rspack_core::ExternalModule>() {
                let instance = ExternalModule { module: js_module }.custom_into_instance(&env_wrapper)?;
                entry.insert(Either5::D(OneShotInstanceRef::from_instance(env, instance)?))
              } else {
                let instance = js_module.custom_into_instance(&env_wrapper)?;
                entry.insert(Either5::E(OneShotInstanceRef::from_instance(env, instance)?))
              };
              match instance_ref {
                Either5::A(r) => ToNapiValue::to_napi_value(env, r),
                Either5::B(r) => ToNapiValue::to_napi_value(env, r),
                Either5::C(r) => ToNapiValue::to_napi_value(env, r),
                Either5::D(r) => ToNapiValue::to_napi_value(env, r),
                Either5::E(r) => ToNapiValue::to_napi_value(env, r),
              }
            },
            None => {
              Err(napi::Error::from_reason(format!(
                "Unable to construct module with id = {} now. The Compiler has been garbage collected by JavaScript.",
                val.identifier
              )))
            },
          }
        }
      }
    })
  }
}

impl FromNapiValue for ModuleObject {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let instance: ModuleInstanceMutRef = FromNapiValue::from_napi_value(env, napi_val)?;

    Ok(match instance {
      Either5::A(normal_module) => Self {
        type_id: TypeId::of::<rspack_core::NormalModule>(),
        identifier: normal_module.module.identifier,
        ptr: normal_module.module.ptr,
        compiler_id: normal_module.module.compiler_id,
      },
      Either5::B(concatenated_module) => Self {
        type_id: TypeId::of::<rspack_core::ConcatenatedModule>(),
        identifier: concatenated_module.module.identifier,
        ptr: concatenated_module.module.ptr,
        compiler_id: concatenated_module.module.compiler_id,
      },
      Either5::C(context_module) => Self {
        type_id: TypeId::of::<rspack_core::ContextModule>(),
        identifier: context_module.module.identifier,
        ptr: context_module.module.ptr,
        compiler_id: context_module.module.compiler_id,
      },
      Either5::D(external_module) => Self {
        type_id: TypeId::of::<rspack_core::ExternalModule>(),
        identifier: external_module.module.identifier,
        ptr: external_module.module.ptr,
        compiler_id: external_module.module.compiler_id,
      },
      Either5::E(module) => Self {
        type_id: TypeId::of::<dyn rspack_core::Module>(),
        identifier: module.identifier,
        ptr: module.ptr,
        compiler_id: module.compiler_id,
      },
    })
  }
}

pub struct ModuleObjectRef {
  pub(crate) identifier: ModuleIdentifier,
}

impl FromNapiValue for ModuleObjectRef {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let instance: ModuleInstanceRef = FromNapiValue::from_napi_value(env, napi_val)?;

    Ok(match instance {
      Either5::A(normal_module) => Self {
        identifier: normal_module.module.identifier,
      },
      Either5::B(concatenated_module) => Self {
        identifier: concatenated_module.module.identifier,
      },
      Either5::C(context_module) => Self {
        identifier: context_module.module.identifier,
      },
      Either5::D(external_module) => Self {
        identifier: external_module.module.identifier,
      },
      Either5::E(module) => Self {
        identifier: module.identifier,
      },
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
  pub source: Option<JsCompatSourceOwned>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
}

#[napi(object, object_from_js = false)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  #[napi(ts_type = "Chunk")]
  pub chunk: ChunkWrapper,
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
      generator: Arc::new(move || {
        let generator = value.generator.clone();
        Box::pin(async move { generator.call_with_sync(()).await })
      }),
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
      cached_generated_code: Default::default(),
    }
  }
}

#[napi(object, object_to_js = false)]
pub struct JsBuildMeta {
  pub strict_esm_module: bool,
  pub has_top_level_await: bool,
  pub esm: bool,
  #[napi(ts_type = "'unset' | 'default' | 'namespace' | 'flagged' | 'dynamic'")]
  pub exports_type: String,
  #[napi(ts_type = "'false' | 'redirect' | JsBuildMetaDefaultObjectRedirectWarn")]
  pub default_object: JsBuildMetaDefaultObject,
  pub side_effect_free: Option<bool>,
  #[napi(ts_type = "Array<[string, string]> | undefined")]
  pub exports_final_name: Option<Vec<Vec<String>>>,
}

impl From<JsBuildMeta> for BuildMeta {
  fn from(value: JsBuildMeta) -> Self {
    let JsBuildMeta {
      strict_esm_module,
      has_top_level_await,
      esm,
      default_object: raw_default_object,
      exports_final_name: raw_exports_final_name,
      side_effect_free,
      exports_type: raw_exports_type,
    } = value;

    let default_object = match raw_default_object {
      Either::A(s) => match s.as_str() {
        "false" => BuildMetaDefaultObject::False,
        "redirect" => BuildMetaDefaultObject::Redirect,
        _ => unreachable!(),
      },
      Either::B(default_object) => BuildMetaDefaultObject::RedirectWarn {
        ignore: default_object.redirect_warn.ignore,
      },
    };

    let exports_type = match raw_exports_type.as_str() {
      "unset" => BuildMetaExportsType::Unset,
      "default" => BuildMetaExportsType::Default,
      "namespace" => BuildMetaExportsType::Namespace,
      "flagged" => BuildMetaExportsType::Flagged,
      "dynamic" => BuildMetaExportsType::Dynamic,
      _ => unreachable!(),
    };

    let exports_final_name = raw_exports_final_name.map(|exports_name| {
      exports_name
        .into_iter()
        .map(|export_name| {
          let first = export_name
            .first()
            .expect("The buildMeta exportsFinalName item should have first value")
            .clone();
          let second = export_name
            .get(1)
            .expect("The buildMeta exportsFinalName item should have second value")
            .clone();
          (first, second)
        })
        .collect::<Vec<_>>()
    });

    Self {
      strict_esm_module,
      has_top_level_await,
      esm,
      exports_type,
      default_object,
      side_effect_free,
      exports_final_name,
      consume_shared_key: None,
      shared_key: None,
    }
  }
}

#[napi(object)]
pub struct JsBuildMetaDefaultObjectRedirectWarn {
  pub redirect_warn: JsDefaultObjectRedirectWarnObject,
}

impl From<JsBuildMetaDefaultObjectRedirectWarn> for BuildMetaDefaultObject {
  fn from(value: JsBuildMetaDefaultObjectRedirectWarn) -> Self {
    Self::RedirectWarn {
      ignore: value.redirect_warn.ignore,
    }
  }
}

#[napi(object)]
pub struct JsDefaultObjectRedirectWarnObject {
  pub ignore: bool,
}

pub type JsBuildMetaDefaultObject = Either<String, JsBuildMetaDefaultObjectRedirectWarn>;
