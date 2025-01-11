use std::{cell::RefCell, ptr::NonNull, sync::Arc};

use napi::JsString;
use napi_derive::napi;
use rspack_collections::IdentifierMap;
use rspack_core::{
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, Compilation, CompilationId, CompilerId,
  ExportsArgument, LibIdentOptions, Module, ModuleArgument, ModuleIdentifier, RuntimeModuleStage,
  SourceType,
};
use rspack_napi::{
  napi::bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, OneShotInstanceRef,
};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap as HashMap;

use super::JsCompatSourceOwned;
use crate::{
  JsChunkWrapper, JsCodegenerationResults, JsCompatSource, JsDependenciesBlockWrapper,
  JsDependencyWrapper, JsResourceData, ToJsCompatSource,
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
pub struct JsModule {
  pub(crate) identifier: ModuleIdentifier,
  module: NonNull<dyn Module>,
  compiler_id: CompilerId,
  compilation_id: CompilationId,
  compilation: Option<NonNull<Compilation>>,
}

impl JsModule {
  fn as_ref(&mut self) -> napi::Result<&'static dyn Module> {
    if let Some(compilation) = self.compilation {
      let compilation = unsafe { compilation.as_ref() };
      if let Some(module) = compilation.module_by_identifier(&self.identifier) {
        Ok(module.as_ref())
      } else {
        Err(napi::Error::from_reason(format!(
          "Unable to access module with id = {} now. The module have been removed on the Rust side.",
          self.identifier
        )))
      }
    } else {
      // SAFETY:
      // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
      // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
      Ok(unsafe { self.module.as_ref() })
    }
  }

  fn as_mut(&mut self) -> napi::Result<&'static mut dyn Module> {
    // SAFETY:
    // We need to make users aware in the documentation that values obtained within the JS hook callback should not be used outside the scope of the callback.
    // We do not guarantee that the memory pointed to by the pointer remains valid when used outside the scope.
    Ok(unsafe { self.module.as_mut() })
  }
}

#[napi]
impl JsModule {
  #[napi(getter)]
  pub fn constructor_name(&mut self) -> napi::Result<String> {
    let module = self.as_ref()?;
    Ok(module.constructor_name().to_string())
  }

  #[napi(getter)]
  pub fn context(&mut self) -> napi::Result<Option<String>> {
    let module = self.as_ref()?;

    Ok(match module.get_context() {
      Some(ctx) => Some(ctx.to_string()),
      None => None,
    })
  }

  #[napi(getter)]
  pub fn original_source<'a>(
    &mut self,
    env: &'a Env,
  ) -> napi::Result<Either<JsCompatSource<'a>, ()>> {
    let module = self.as_ref()?;

    Ok(match module.original_source() {
      Some(source) => match source.to_js_compat_source(env).ok() {
        Some(s) => Either::A(s),
        None => Either::B(()),
      },
      None => Either::B(()),
    })
  }

  #[napi(getter)]
  pub fn resource(&mut self) -> napi::Result<Option<&String>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Some(&normal_module.resource_resolved_data().resource),
      Err(_) => None,
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
  pub fn request(&mut self) -> napi::Result<Option<&str>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Some(normal_module.request()),
      Err(_) => None,
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
  pub fn raw_request(&mut self) -> napi::Result<Option<&str>> {
    let module = self.as_ref()?;

    Ok(match module.try_as_normal_module() {
      Ok(normal_module) => Some(normal_module.raw_request()),
      Err(_) => None,
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
  pub fn layer(&mut self) -> napi::Result<Option<&String>> {
    let module = self.as_ref()?;

    Ok(match module.get_layer() {
      Some(layer) => Some(layer),
      None => None,
    })
  }

  #[napi(getter, ts_return_type = "JsDependenciesBlock[]")]
  pub fn blocks(&mut self) -> napi::Result<Vec<JsDependenciesBlockWrapper>> {
    Ok(match self.compilation {
      Some(compilation) => {
        let compilation = unsafe { compilation.as_ref() };
        let module_graph = compilation.get_module_graph();
        let module = self.as_ref()?;

        let blocks = module.get_blocks();
        blocks
          .iter()
          .filter_map(|block_id| {
            module_graph
              .block_by_id(block_id)
              .map(|block| JsDependenciesBlockWrapper::new(block, compilation))
          })
          .collect::<Vec<_>>()
      }
      None => {
        vec![]
      }
    })
  }

  #[napi(getter, ts_return_type = "JsDependency[]")]
  pub fn dependencies(&mut self) -> napi::Result<Vec<JsDependencyWrapper>> {
    Ok(match self.compilation {
      Some(compilation) => {
        let compilation = unsafe { compilation.as_ref() };
        let module_graph = compilation.get_module_graph();
        let module = self.as_ref()?;
        let dependencies = module.get_dependencies();
        dependencies
          .iter()
          .filter_map(|dependency_id| {
            module_graph.dependency_by_id(dependency_id).map(|dep| {
              let compilation = unsafe { self.compilation.map(|c| c.as_ref()) };
              JsDependencyWrapper::new(dep.as_ref(), self.compilation_id, compilation)
            })
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
                .map(|module| {
                  JsModuleWrapper::new(
                    module.as_ref(),
                    compilation.compiler_id(),
                    compilation.id(),
                    Some(compilation),
                  )
                })
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

  #[napi]
  pub fn lib_ident(
    &mut self,
    env: Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString>> {
    let module = self.as_ref()?;
    Ok(
      match module.lib_ident(LibIdentOptions {
        context: &options.context,
      }) {
        Some(lib_ident) => Some(env.create_string(lib_ident.as_ref())?),
        None => None,
      },
    )
  }

  #[napi(getter)]
  pub fn resource_resolve_data(&mut self) -> napi::Result<Option<JsResourceData>> {
    let module = self.as_ref()?;
    Ok(match module.as_normal_module() {
      Some(module) => Some(module.resource_resolved_data().into()),
      None => None,
    })
  }

  #[napi(getter)]
  pub fn match_resource(&mut self) -> napi::Result<Option<&String>> {
    let module = self.as_ref()?;
    Ok(match module.as_normal_module() {
      Some(module) => match &module.match_resource() {
        Some(match_resource) => Some(&match_resource.resource),
        None => None,
      },
      None => None,
    })
  }

  #[napi(getter)]
  pub fn loaders(&mut self) -> napi::Result<Either<Vec<&str>, ()>> {
    let module = self.as_ref()?;
    Ok(match module.as_normal_module() {
      Some(module) => {
        let ids = module
          .loaders()
          .iter()
          .map(|loader| loader.identifier().as_str())
          .collect::<Vec<_>>();
        Either::A(ids)
      }
      None => Either::B(()),
    })
  }
}

type ModuleInstanceRefs = IdentifierMap<OneShotInstanceRef<JsModule>>;

type ModuleInstanceRefsByCompilerId = RefCell<HashMap<CompilerId, ModuleInstanceRefs>>;

thread_local! {
  static MODULE_INSTANCE_REFS: ModuleInstanceRefsByCompilerId = Default::default();
}

// The difference between JsModuleWrapper and JsModule is:
// JsModuleWrapper maintains a cache to ensure that the corresponding instance of the same Module is unique on the JS side.
//
// This means that when transferring a JsModule from Rust to JS, you must use JsModuleWrapper instead.
pub struct JsModuleWrapper {
  identifier: ModuleIdentifier,
  module: NonNull<dyn Module>,
  compiler_id: CompilerId,
  compilation_id: CompilationId,
  compilation: Option<NonNull<Compilation>>,
}

unsafe impl Send for JsModuleWrapper {}

impl JsModuleWrapper {
  pub fn new(
    module: &dyn Module,
    compiler_id: CompilerId,
    compilation_id: CompilationId,
    compilation: Option<&Compilation>,
  ) -> Self {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    let identifier = module.identifier();

    #[allow(clippy::unwrap_used)]
    Self {
      identifier,
      module: NonNull::new(module as *const dyn Module as *mut dyn Module).unwrap(),
      compiler_id,
      compilation_id,
      compilation: compilation
        .map(|c| NonNull::new(c as *const Compilation as *mut Compilation).unwrap()),
    }
  }

  pub fn cleanup_last_compilation(compilation_id: CompilationId) {
    // MODULE_INSTANCE_REFS.with(|refs| {
    //   let mut refs_by_compilation_id = refs.borrow_mut();
    //   refs_by_compilation_id.remove(&compilation_id)
    // });
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

      match refs.entry(module.identifier()) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
          let r = entry.get_mut();
          let instance = &mut **r;
          instance.compilation = val.compilation;
          instance.module = val.module;
          ToNapiValue::to_napi_value(env, r)
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let js_module = JsModule {
            identifier: val.identifier,
            module: val.module,
            compiler_id: val.compiler_id,
            compilation_id: val.compilation_id,
            compilation: val.compilation,
          };
          let r = entry.insert(OneShotInstanceRef::new(env, js_module)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}

impl FromNapiValue for JsModuleWrapper {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let instance: ClassInstance<JsModule> = FromNapiValue::from_napi_value(env, napi_val)?;

    Ok(JsModuleWrapper {
      identifier: instance.identifier,
      #[allow(clippy::unwrap_used)]
      module: instance.module,
      compiler_id: instance.compiler_id,
      compilation_id: instance.compilation_id,
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
  #[napi(ts_type = "'module' | 'webpackModule'")]
  pub module_argument: String,
  #[napi(ts_type = "'exports' | 'webpackExports'")]
  pub exports_argument: String,
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
      exports_argument: raw_exports_argument,
      default_object: raw_default_object,
      module_argument: raw_module_argument,
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

    let module_argument = match raw_module_argument.as_str() {
      "module" => ModuleArgument::Module,
      "webpackModule" => ModuleArgument::WebpackModule,
      _ => unreachable!(),
    };

    let exports_argument = match raw_exports_argument.as_str() {
      "exports" => ExportsArgument::Exports,
      "webpackExports" => ExportsArgument::WebpackExports,
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
      module_argument,
      exports_argument,
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
