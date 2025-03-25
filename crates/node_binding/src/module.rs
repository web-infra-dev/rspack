use std::sync::Arc;

use napi::{CallContext, JsString, NapiRaw};
use napi_derive::napi;
use rspack_core::{
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, CompilationAsset, CompilationId,
  LibIdentOptions, Module as _, ModuleIdentifier, Reflector, RuntimeModuleStage, SourceType,
};
use rspack_napi::{napi::bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use rspack_plugin_runtime::RuntimeModuleFromJs;
use rspack_util::source_map::SourceMapKind;

use super::JsCompatSourceOwned;
use crate::{
  allocator::COMPILATION_INSTANCE_REFS, AssetInfo, AsyncDependenciesBlockWrapper,
  ConcatenatedModule, ContextModule, DependencyWrapper, ExternalModule, JsChunkWrapper,
  JsCodegenerationResults, JsCompatSource, JsCompilation, NormalModule, ToJsCompatSource,
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
pub struct Module(pub(crate) Box<dyn rspack_core::Module>);

impl Module {
  pub(crate) fn custom_into_instance(self, env: &Env) -> napi::Result<ClassInstance<Self>> {
    let instance = self.into_instance(env)?;
    let mut object = instance.as_object(env);
    let module = &*instance.0;

    #[js_function]
    fn context_getter(ctx: CallContext) -> napi::Result<Either<String, ()>> {
      let this = ctx.this_unchecked::<Object>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = &*wrapped_value.0;
      Ok(match module.get_context() {
        Some(ctx) => Either::A(ctx.to_string()),
        None => Either::B(()),
      })
    }

    #[js_function]
    fn layer_getter(ctx: CallContext) -> napi::Result<Either<&String, ()>> {
      let this = ctx.this_unchecked::<Object>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = &*wrapped_value.0;
      Ok(match module.get_layer() {
        Some(layer) => Either::A(layer),
        None => Either::B(()),
      })
    }

    #[js_function]
    fn use_source_map_getter(ctx: CallContext) -> napi::Result<bool> {
      let this = ctx.this_unchecked::<Object>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = &*wrapped_value.0;
      Ok(module.get_source_map_kind().source_map())
    }

    #[js_function]
    fn use_simple_source_map_getter(ctx: CallContext) -> napi::Result<bool> {
      let this = ctx.this_unchecked::<Object>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = &*wrapped_value.0;
      Ok(module.get_source_map_kind().source_map())
    }

    #[js_function]
    fn factory_meta_getter(ctx: CallContext) -> napi::Result<Either<JsFactoryMeta, ()>> {
      let this = ctx.this_unchecked::<Object>();
      let wrapped_value = unsafe { Module::from_napi_mut_ref(ctx.env.raw(), this.raw())? };
      let module = &*wrapped_value.0;
      Ok(match module.as_normal_module() {
        Some(normal_module) => match normal_module.factory_meta() {
          Some(meta) => Either::A(JsFactoryMeta {
            side_effect_free: meta.side_effect_free,
          }),
          None => Either::B(()),
        },
        None => Either::B(()),
      })
    }

    object.define_properties(&[
      Property::new("type")?
        .with_value(&env.create_string(module.module_type().as_str())?)
        .with_property_attributes(PropertyAttributes::Enumerable),
      Property::new("context")?.with_getter(context_getter),
      Property::new("layer")?.with_getter(layer_getter),
      Property::new("useSourceMap")?.with_getter(use_source_map_getter),
      Property::new("useSimpleSourceMap")?.with_getter(use_simple_source_map_getter),
      Property::new("factoryMeta")?.with_getter(factory_meta_getter),
      Property::new("buildInfo")?.with_value(&env.create_object()?),
      Property::new("buildMeta")?.with_value(&env.create_object()?),
    ])?;

    Ok(instance)
  }

  fn get_compilation_ref(
    &self,
    env: &Env,
    this: This,
  ) -> napi::Result<Option<&rspack_core::Compilation>> {
    Ok(match this.get::<Object>("_compilation")? {
      Some(compilation_object) => {
        let js_compilation =
          unsafe { JsCompilation::from_napi_mut_ref(env.raw(), compilation_object.raw())? };
        Some(&js_compilation.0)
      }
      None => None,
    })
  }
}

#[napi]
impl Module {
  #[napi(js_name = "_originalSource", enumerable = false)]
  pub fn original_source(&self, env: &Env) -> napi::Result<Either<JsCompatSource, ()>> {
    let module = &*self.0;

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
    let module = &*self.0;

    Ok(module.identifier().as_str())
  }

  #[napi]
  pub fn name_for_condition(&mut self) -> napi::Result<Either<String, ()>> {
    let module = &*self.0;

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
  pub fn blocks(
    &mut self,
    env: &Env,
    this: This,
  ) -> napi::Result<Vec<AsyncDependenciesBlockWrapper>> {
    let Some(compilation) = self.get_compilation_ref(env, this)? else {
      return Ok(vec![]);
    };

    let module = &*self.0;

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
  pub fn dependencies(&mut self, env: &Env, this: This) -> napi::Result<Vec<DependencyWrapper>> {
    let Some(compilation) = self.get_compilation_ref(env, this)? else {
      return Ok(vec![]);
    };

    let module = &*self.0;
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
  pub fn size(&mut self, env: &Env, this: This, ty: Option<String>) -> napi::Result<f64> {
    let module = &*self.0;
    let ty = ty.map(|s| SourceType::from(s.as_str()));

    let Some(compilation) = self.get_compilation_ref(env, this)? else {
      return Ok(module.size(ty.as_ref(), None));
    };
    Ok(module.size(ty.as_ref(), Some(compilation)))
  }

  #[napi]
  pub fn lib_ident(
    &mut self,
    env: &Env,
    options: JsLibIdentOptions,
  ) -> napi::Result<Option<JsString>> {
    let module = &*self.0;
    Ok(
      match module.lib_ident(LibIdentOptions {
        context: &options.context,
      }) {
        Some(lib_ident) => Some(env.create_string(lib_ident.as_ref())?),
        None => None,
      },
    )
  }

  #[napi(js_name = "_emitFile", enumerable = false)]
  pub fn emit_file(
    &mut self,
    filename: String,
    source: JsCompatSource,
    js_asset_info: Option<AssetInfo>,
  ) -> napi::Result<()> {
    let module = &mut *self.0;

    let asset_info = js_asset_info.map(Into::into).unwrap_or_default();

    module.build_info_mut().assets.insert(
      filename,
      CompilationAsset::new(Some(source.into()), asset_info),
    );
    Ok(())
  }
}

type ModuleInstanceRef<'a> = Either5<
  &'a NormalModule,
  &'a ConcatenatedModule,
  &'a ContextModule,
  &'a ExternalModule,
  &'a Module,
>;

pub struct ModuleObject {
  module: Reflector,
  compilation_id: CompilationId,
}

impl ModuleObject {
  pub fn new(module: &dyn rspack_core::Module, compilation_id: CompilationId) -> Self {
    Self {
      module: module.reflector().clone(),
      compilation_id,
    }
  }
}

impl ToNapiValue for ModuleObject {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let napi_val = ToNapiValue::to_napi_value(env, val.module)?;
    let mut module_object = Object::from_napi_value(env, napi_val)?;

    let compilation_object = COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let references = ref_cell.borrow();
      let Some(weak_reference) = references.get(&val.compilation_id) else {
        return Err(napi::Error::from_reason(
          "Unable to create module object. The relative Compilation has been garbage collected by JavaScript."
        ));
      };
      let napi_val = ToNapiValue::to_napi_value(env, weak_reference.clone())?;
      Object::from_napi_value(env, napi_val)
    })?;

    module_object.set_named_property("_compilation", compilation_object)?;

    Ok(napi_val)
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
        identifier: normal_module.module.0.identifier(),
      },
      Either5::B(concatenated_module) => Self {
        identifier: concatenated_module.module.0.identifier(),
      },
      Either5::C(context_module) => Self {
        identifier: context_module.module.0.identifier(),
      },
      Either5::D(external_module) => Self {
        identifier: external_module.module.0.identifier(),
      },
      Either5::E(module) => Self {
        identifier: module.0.identifier(),
      },
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
    let mut module = Self::new(
      value.name,
      Arc::new(move || value.generator.blocking_call_with_sync(())),
      value.full_hash,
      value.dependent_hash,
      value.isolate,
      RuntimeModuleStage::from(value.stage),
    );
    module.source_map_kind = SourceMapKind::empty();
    module
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
