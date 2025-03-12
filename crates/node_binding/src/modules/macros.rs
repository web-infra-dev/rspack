#[macro_export]
macro_rules! impl_module_methods {
  ($module:ident) => {
    impl $module {
      pub(crate) fn custom_into_instance(
        self,
        env: &napi::Env,
      ) -> napi::Result<napi::bindgen_prelude::ClassInstance<Self>> {
        use napi::bindgen_prelude::JavaScriptClassExt;

        let mut instance = self.into_instance(env)?;
        let mut object = instance.as_object(env);
        let (_, module) = instance.module.as_ref()?;

        object.set_named_property("type", env.create_string(module.module_type().as_str())?)?;

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
        fn factory_meta_getter(
          ctx: napi::CallContext,
        ) -> napi::Result<napi::Either<$crate::JsFactoryMeta, ()>> {
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
              Some(meta) => napi::Either::A($crate::JsFactoryMeta {
                side_effect_free: meta.side_effect_free,
              }),
              None => napi::Either::B(()),
            },
            None => napi::Either::B(()),
          })
        }

        object.define_properties(&[
          napi::Property::new("context")?.with_getter(context_getter),
          napi::Property::new("layer")?.with_getter(layer_getter),
          napi::Property::new("useSourceMap")?.with_getter(use_source_map_getter),
          napi::Property::new("useSimpleSourceMap")?.with_getter(use_simple_source_map_getter),
          napi::Property::new("factoryMeta")?.with_getter(factory_meta_getter),
        ])?;

        object.set_named_property("buildInfo", env.create_object()?)?;
        object.set_named_property("buildMeta", env.create_object()?)?;

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
      pub fn identifier(&mut self) -> napi::Result<&str> {
        self.module.identifier()
      }

      #[napi]
      pub fn name_for_condition(&mut self) -> napi::Result<napi::Either<String, ()>> {
        self.module.name_for_condition()
      }

      #[napi(
        getter,
        js_name = "_blocks",
        ts_return_type = "JsDependenciesBlock[]",
        enumerable = false
      )]
      pub fn blocks(&mut self) -> napi::Result<Vec<$crate::JsDependenciesBlockWrapper>> {
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

      #[napi(js_name = "_emitFile", enumerable = false)]
      pub fn emit_file(
        &mut self,
        filename: String,
        source: $crate::JsCompatSource,
        js_asset_info: Option<$crate::AssetInfo>,
      ) -> napi::Result<()> {
        self.module.emit_file(filename, source, js_asset_info)
      }
    }
  };
}
