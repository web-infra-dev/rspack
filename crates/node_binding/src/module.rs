use std::{any::TypeId, cell::RefCell, ptr::NonNull, sync::Arc};

use napi::JsString;
use napi_derive::napi;
use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_core::{
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, Compilation, CompilationAsset,
  CompilerId, LibIdentOptions, ModuleIdentifier, RuntimeModuleStage, SourceType,
};
use rspack_napi::{
  napi::bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, OneShotInstanceRef,
};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;

use super::JsCompatSourceOwned;
use crate::{
  AssetInfo, ContextModule, DependencyWrapper, JsChunkWrapper, JsCodegenerationResults,
  JsCompatSource, JsCompiler, JsDependenciesBlockWrapper, NormalModule, ToJsCompatSource,
  COMPILER_REFERENCES,
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

#[napi]
pub struct Module {
  pub(crate) identifier: ModuleIdentifier,
  // TODO: Replace with Option<Box<dyn Module>> in the future for better ownership and safety
  module: Option<NonNull<dyn rspack_core::Module>>,
  compiler_id: CompilerId,
  compiler_reference: WeakReference<JsCompiler>,
}

impl Module {
  pub(crate) fn as_ref(&mut self) -> napi::Result<(&Compilation, &dyn rspack_core::Module)> {
    match self.compiler_reference.get() {
      Some(this) => {
        let compilation = &this.compiler.compilation;
        if let Some(module) = compilation.module_by_identifier(&self.identifier) {
          Ok((compilation, module.as_ref()))
        } else if let Some(module) = self.module {
          // SAFETY:
          // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
          // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
          Ok((compilation, unsafe { module.as_ref() }))
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
    match self.module.as_mut() {
      Some(module) => {
        // SAFETY:
        // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
        // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
        Ok(unsafe { module.as_mut() })
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
  #[napi(getter)]
  pub fn context(&mut self) -> napi::Result<Either<String, ()>> {
    let (_, module) = self.as_ref()?;

    Ok(match module.get_context() {
      Some(ctx) => Either::A(ctx.to_string()),
      None => Either::B(()),
    })
  }

  #[napi(js_name = "_originalSource")]
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
  pub fn identifier(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.identifier().as_str())
  }

  #[napi]
  pub fn name_for_condition(&mut self) -> napi::Result<Either<String, ()>> {
    let (_, module) = self.as_ref()?;

    Ok(match module.name_for_condition() {
      Some(s) => Either::A(s.to_string()),
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn get_type(&mut self) -> napi::Result<&str> {
    let (_, module) = self.as_ref()?;

    Ok(module.module_type().as_str())
  }

  #[napi(getter)]
  pub fn layer(&mut self) -> napi::Result<Either<&String, ()>> {
    let (_, module) = self.as_ref()?;

    Ok(match module.get_layer() {
      Some(layer) => Either::A(layer),
      None => Either::B(()),
    })
  }

  #[napi(getter, js_name = "_blocks", ts_return_type = "JsDependenciesBlock[]")]
  pub fn blocks(&mut self) -> napi::Result<Vec<JsDependenciesBlockWrapper>> {
    let (compilation, module) = self.as_ref()?;

    let module_graph = compilation.get_module_graph();
    let blocks = module.get_blocks();
    Ok(
      blocks
        .iter()
        .filter_map(|block_id| {
          module_graph
            .block_by_id(block_id)
            .map(|block| JsDependenciesBlockWrapper::new(block, compilation))
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

  #[napi(getter)]
  pub fn use_source_map(&mut self) -> napi::Result<bool> {
    let (_, module) = self.as_ref()?;
    Ok(module.get_source_map_kind().source_map())
  }

  #[napi]
  pub fn lib_ident(
    &mut self,
    env: Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString>> {
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

  #[napi(js_name = "_emitFile")]
  pub fn emit_file(
    &mut self,
    filename: String,
    source: JsCompatSource,
    js_asset_info: Option<AssetInfo>,
  ) -> napi::Result<()> {
    let module = self.as_mut()?;

    let asset_info = js_asset_info.map(Into::into).unwrap_or_default();

    module.build_info_mut().assets.insert(
      filename,
      CompilationAsset::new(Some(source.into()), asset_info),
    );
    Ok(())
  }
}

type ModuleInstanceRef = Either3<
  OneShotInstanceRef<NormalModule>,
  OneShotInstanceRef<ContextModule>,
  OneShotInstanceRef<Module>,
>;

type ModuleInstanceRefs = IdentifierMap<ModuleInstanceRef>;

type ModuleInstanceRefsByCompilerId = RefCell<UkeyMap<CompilerId, ModuleInstanceRefs>>;

thread_local! {
  static MODULE_INSTANCE_REFS: ModuleInstanceRefsByCompilerId = Default::default();
}

// The difference between ModuleWrapper and Module is:
// ModuleWrapper maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a Module from Rust to JS, you must use ModuleWrapper instead.
pub struct ModuleWrapper {
  type_id: TypeId,
  identifier: ModuleIdentifier,
  module: Option<NonNull<dyn rspack_core::Module>>,
  compiler_id: CompilerId,
}

unsafe impl Send for ModuleWrapper {}

impl ModuleWrapper {
  pub fn with_ref(module: &dyn rspack_core::Module, compiler_id: CompilerId) -> Self {
    let type_id = if module.as_normal_module().is_some() {
      TypeId::of::<rspack_core::NormalModule>()
    } else if module.as_context_module().is_some() {
      TypeId::of::<rspack_core::ContextModule>()
    } else {
      TypeId::of::<dyn rspack_core::Module>()
    };
    Self {
      type_id,
      identifier: module.identifier(),
      module: None,
      compiler_id,
    }
  }

  pub fn with_ptr(module_ptr: NonNull<dyn rspack_core::Module>, compiler_id: CompilerId) -> Self {
    let module = unsafe { module_ptr.as_ref() };

    let type_id = if module.as_context_module().is_some() {
      TypeId::of::<rspack_core::ContextModule>()
    } else {
      TypeId::of::<dyn rspack_core::Module>()
    };

    Self {
      type_id,
      identifier: module.identifier(),
      module: Some(module_ptr),
      compiler_id,
    }
  }

  pub fn cleanup_by_compiler_id(compiler_id: &CompilerId) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      refs_by_compiler_id.remove(compiler_id)
    });
  }

  pub fn cleanup_by_module_identifiers(revoked_modules: &[ModuleIdentifier]) {
    MODULE_INSTANCE_REFS.with(|refs| {
      let mut refs_by_compiler_id = refs.borrow_mut();
      for module_identifier in revoked_modules {
        for (_, refs) in refs_by_compiler_id.iter_mut() {
          refs.remove(module_identifier);
        }
      }
    });
  }

  pub fn take(&mut self) -> Option<NonNull<dyn rspack_core::Module>> {
    self.module
  }
}

impl ToNapiValue for ModuleWrapper {
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
            Either3::A(normal_module) => &mut normal_module.module,
            Either3::B(context_module) => &mut context_module.module,
            Either3::C(module) => &mut **module,
          };
          instance.module = val.module;
          match instance_ref {
            Either3::A(r) => ToNapiValue::to_napi_value(env, r),
            Either3::B(r) => ToNapiValue::to_napi_value(env, r),
            Either3::C(r) => ToNapiValue::to_napi_value(env, r),
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
                module: val.module,
                compiler_reference,
              };
              let env_wrapper = Env::from_raw(env);
              let env_wrapper_ref = &env_wrapper;

              let set_named_properties = &mut |mut object: Object| -> napi::Result<()> {
                object.set_named_property("buildInfo", env_wrapper_ref.create_object()?)?;
                object.set_named_property("buildMeta", env_wrapper_ref.create_object()?)?;
                Ok(())
              };

              let instance_ref = if val.type_id == TypeId::of::<rspack_core::NormalModule>() {
                let instance = NormalModule { module: js_module }.into_instance(&env_wrapper)?;
                set_named_properties(instance.as_object(&env_wrapper))?;
                entry.insert(Either3::A(OneShotInstanceRef::from_instance(env, instance)?))
              } else if val.type_id == TypeId::of::<rspack_core::ContextModule>() {
                let instance = ContextModule { module: js_module }.into_instance(&env_wrapper)?;
                set_named_properties(instance.as_object(&env_wrapper))?;
                entry.insert(Either3::B(OneShotInstanceRef::from_instance(env, instance)?))
              } else {
                let instance = js_module.into_instance(&env_wrapper)?;
                set_named_properties(instance.as_object(&env_wrapper))?;
                entry.insert(Either3::C(OneShotInstanceRef::from_instance(env, instance)?))
              };
              match instance_ref {
                Either3::A(r) => ToNapiValue::to_napi_value(env, r),
                Either3::B(r) => ToNapiValue::to_napi_value(env, r),
                Either3::C(r) => ToNapiValue::to_napi_value(env, r),
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

impl FromNapiValue for ModuleWrapper {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let instance: Either<&mut ContextModule, &mut Module> =
      FromNapiValue::from_napi_value(env, napi_val)?;

    let (js_module, type_id) = match instance {
      Either::A(context_module) => (
        &mut context_module.module,
        TypeId::of::<rspack_core::ContextModule>(),
      ),
      Either::B(module) => (module, TypeId::of::<dyn rspack_core::Module>()),
    };

    Ok(ModuleWrapper {
      type_id,
      identifier: js_module.identifier,
      module: js_module.module.take(),
      compiler_id: js_module.compiler_id,
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
  pub source: Option<JsCompatSourceOwned>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
}

#[napi(object, object_from_js = false)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  #[napi(ts_type = "JsChunk")]
  pub chunk: JsChunkWrapper,
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
