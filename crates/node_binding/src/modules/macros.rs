#[macro_export]
macro_rules! impl_module_methods {
  ($module:ident) => {
    impl $module {
      fn new_inherited<'a>(
        self,
        env: &'a napi::Env,
        mut properties: Vec<napi::Property>,
      ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'a, Self>> {
        use napi::bindgen_prelude::JavaScriptClassExt;

        let mut instance = self.into_instance(env)?;
        let mut object = instance.as_object(env);
        let (_, module) = instance.module.as_ref()?;

        #[js_function]
        fn context_getter(ctx: napi::CallContext) -> napi::Result<napi::Either<String, ()>> {
          let this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          Ok(match module.get_context() {
            Some(ctx) => napi::Either::A(ctx.to_string()),
            None => napi::Either::B(()),
          })
        }

        #[js_function]
        fn layer_getter(ctx: napi::CallContext) -> napi::Result<napi::Either<&String, ()>> {
          let this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          Ok(match module.get_layer() {
            Some(layer) => napi::Either::A(layer),
            None => napi::Either::B(()),
          })
        }

        #[js_function]
        fn use_source_map_getter(ctx: napi::CallContext) -> napi::Result<bool> {
          let this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          Ok(module.get_source_map_kind().source_map())
        }

        #[js_function]
        fn use_simple_source_map_getter(ctx: napi::CallContext) -> napi::Result<bool> {
          let this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          Ok(module.get_source_map_kind().source_map())
        }

        #[js_function]
        fn factory_meta_getter(ctx: napi::CallContext) -> napi::Result<$crate::JsFactoryMeta> {
          use rspack_core::Module;

          let this = ctx.this_unchecked::<napi::bindgen_prelude::Object>();
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          Ok(match module.as_normal_module() {
            Some(normal_module) => match normal_module.factory_meta() {
              Some(meta) => $crate::JsFactoryMeta {
                side_effect_free: meta.side_effect_free,
              },
              None => $crate::JsFactoryMeta {
                side_effect_free: None,
              },
            },
            None => $crate::JsFactoryMeta {
              side_effect_free: None,
            },
          })
        }

        #[js_function(1)]
        fn factory_meta_setter(ctx: napi::CallContext) -> napi::Result<napi::JsUndefined> {
          let this = ctx.this_unchecked::<napi::bindgen_prelude::Object>();
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let module = wrapped_value.module.as_mut()?;
          let factory_meta = ctx.get::<$crate::JsFactoryMeta>(0)?;
          module.set_factory_meta(factory_meta.into());
          ctx.env.get_undefined()
        }

        #[js_function]
        fn readable_identifier_getter(ctx: napi::CallContext) -> napi::Result<String> {
          let this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let wrapped_value: &mut $module = unsafe {
            napi::bindgen_prelude::FromNapiMutRef::from_napi_mut_ref(
              ctx.env.raw(),
              napi::NapiRaw::raw(&this),
            )?
          };
          let (_, module) = wrapped_value.module.as_ref()?;
          let res = module
            .get_context()
            .map(|ctx| module.readable_identifier(ctx.as_ref()).to_string())
            .unwrap_or_default();
          Ok(res)
        }

        #[js_function]
        fn build_info_getter(ctx: napi::CallContext) -> napi::Result<napi::bindgen_prelude::Object> {
          use napi::{bindgen_prelude::FromNapiValue, NapiRaw, NapiValue};
          let mut this = ctx.this::<napi::bindgen_prelude::Object>()?;
          let env = ctx.env;
          let raw_env = env.raw();
          let mut reference: napi::bindgen_prelude::Reference<$crate::Module> =
            unsafe { napi::bindgen_prelude::Reference::from_napi_value(raw_env, this.raw())? };
          if let Some(r) = &reference.build_info_ref {
            return r.as_object(env);
          }
          let mut build_info = $crate::BuildInfo::new(reference.downgrade()).get_jsobject(env)?;
          $crate::MODULE_BUILD_INFO_SYMBOL.with(|once_cell| {
            let sym = unsafe {
              #[allow(clippy::unwrap_used)]
              let napi_val = napi::bindgen_prelude::ToNapiValue::to_napi_value(env.raw(), once_cell.get().unwrap())?;
              napi::JsSymbol::from_raw_unchecked(env.raw(), napi_val)
            };
            this.set_property(sym, &build_info)
          })?;
          let r = rspack_napi::WeakRef::new(raw_env, &mut build_info)?;
          let result = r.as_object(env);
          reference.build_info_ref = Some(r);
          result
        }

        #[js_function(1)]
        fn build_info_setter(ctx: napi::CallContext) -> napi::Result<()> {
          use napi::{bindgen_prelude::FromNapiValue, NapiRaw, NapiValue};
          use rspack_napi::string::JsStringExt;
          let mut this = ctx.this_unchecked::<napi::bindgen_prelude::Object>();
          let input_object = ctx.get::<napi::bindgen_prelude::Object>(0)?;
          let env = ctx.env;
          let raw_env = env.raw();
          let mut reference: napi::bindgen_prelude::Reference<Module> =
            unsafe { napi::bindgen_prelude::Reference::from_napi_value(raw_env, this.raw())? };
          let new_build_info = $crate::BuildInfo::new(reference.downgrade());
          let mut new_instrance = new_build_info.get_jsobject(env)?;

          let names = input_object.get_all_property_names(
            napi::KeyCollectionMode::OwnOnly,
            napi::KeyFilter::AllProperties,
            napi::KeyConversion::KeepNumbers,
          )?;
          let names = napi::bindgen_prelude::Array::from_unknown(names.into_unknown())?;
          for index in 0..names.len() {
            if let Some(name) = names.get::<napi::bindgen_prelude::Unknown>(index)? {
              let name_clone = unsafe { napi::bindgen_prelude::Object::from_raw_unchecked(env.raw(), name.raw()) };
              let name_str = name_clone.coerce_to_string()?.into_string();
              // known build info properties
              if name_str == "assets" {
                // TODO: Currently, setting assets is not supported.
                continue;
              } else {
                let value = input_object.get_property::<&napi::bindgen_prelude::Unknown, napi::bindgen_prelude::Unknown>(&name)?;
                new_instrance.set_property::<napi::bindgen_prelude::Unknown, napi::bindgen_prelude::Unknown>(name, value)?;
              }
            }
          }

          $crate::MODULE_BUILD_INFO_SYMBOL.with(|once_cell| {
            let sym = unsafe {
              #[allow(clippy::unwrap_used)]
              let napi_val = napi::bindgen_prelude::ToNapiValue::to_napi_value(env.raw(), once_cell.get().unwrap())?;
              napi::JsSymbol::from_raw_unchecked(env.raw(), napi_val)
            };
            this.set_property(sym, &new_instrance)
          })?;
          reference.build_info_ref = Some(rspack_napi::WeakRef::new(raw_env, &mut new_instrance)?);
          Ok(())
        }

        properties.push(
          napi::Property::new("type")?
            .with_value(&env.create_string(module.module_type().as_str())?),
        );
        properties.push(napi::Property::new("context")?.with_getter(context_getter));
        properties.push(napi::Property::new("layer")?.with_getter(layer_getter));
        properties.push(napi::Property::new("useSourceMap")?.with_getter(use_source_map_getter));
        properties.push(
          napi::Property::new("useSimpleSourceMap")?.with_getter(use_simple_source_map_getter),
        );
        properties.push(
          napi::Property::new("factoryMeta")?
            .with_getter(factory_meta_getter)
            .with_setter(factory_meta_setter),
        );
        properties.push(napi::Property::new("buildInfo")?.with_getter(build_info_getter).with_setter(build_info_setter));
        properties.push(napi::Property::new("buildMeta")?.with_value(&env.create_object()?));
        properties.push(
          napi::Property::new("_readableIdentifier")?.with_getter(readable_identifier_getter),
        );
        object.define_properties(&properties)?;

        $crate::MODULE_IDENTIFIER_SYMBOL.with(|once_cell| {
          let identifier = env.create_string(module.identifier().as_str())?;
          let symbol = unsafe {
            #[allow(clippy::unwrap_used)]
            let napi_val = napi::bindgen_prelude::ToNapiValue::to_napi_value(
              env.raw(),
              once_cell.get().unwrap(),
            )?;
            <napi::JsSymbol as napi::NapiValue>::from_raw_unchecked(env.raw(), napi_val)
          };
          object.set_property(symbol, identifier)
        })?;

        Ok(instance)
      }
    }

    #[napi]
    impl $module {
      #[napi(js_name = "_originalSource", enumerable = false)]
      pub fn original_source(
        &mut self,
        env: &napi::Env,
      ) -> napi::Result<napi::Either<$crate::JsCompatSource, ()>> {
        self.module.original_source(env)
      }

      #[napi]
      pub fn name_for_condition(&mut self) -> napi::Result<napi::Either<String, ()>> {
        self.module.name_for_condition()
      }

      #[napi(
        getter,
        ts_return_type = "AsyncDependenciesBlock[]",
        enumerable = false
      )]
      pub fn blocks(&mut self) -> napi::Result<Vec<$crate::AsyncDependenciesBlockWrapper>> {
        self.module.blocks()
      }

      #[napi(getter, ts_return_type = "Dependency[]")]
      pub fn dependencies(&mut self) -> napi::Result<Vec<$crate::DependencyWrapper>> {
        self.module.dependencies()
      }

      #[napi]
      pub fn size(&mut self, ty: Option<String>) -> napi::Result<f64> {
        self.module.size(ty)
      }

      #[napi]
      pub fn lib_ident(
        &mut self,
        env: &napi::Env,
        options: $crate::JsLibIdentOptions,
      ) -> napi::Result<Option<napi::JsString>> {
        self.module.lib_ident(env, options)
      }

      #[napi(
        js_name = "_emitFile",
        enumerable = false,
        ts_args_type = "filename: string, source: JsCompatSource, assetInfo?: AssetInfo | undefined | null"
      )]
      pub fn emit_file(
        &mut self,
        env: &napi::Env,
        filename: String,
        source: $crate::JsCompatSource,
        asset_info: Option<napi::bindgen_prelude::Object>,
      ) -> napi::Result<()> {
        self
          .module
          .emit_file(env, filename, source, asset_info)
      }
    }
  };
}
