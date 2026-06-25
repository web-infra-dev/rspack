#[macro_export]
macro_rules! impl_module_methods {
  ($module:ident) => {
    impl $crate::module::DerivedModule for $module {
      fn as_module(&mut self) -> &mut $crate::module::Module {
        &mut self.module
      }
    }

    impl $module {
      fn new_inherited<'a>(
        self,
        env: &'a napi::Env,
        properties: &mut Vec<napi::Property>,
      ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'a, Self>> {
        use napi::bindgen_prelude::{JavaScriptClassExt, JsObjectValue};

        let mut instance = self.into_instance(env)?;
        let mut object = instance.as_object(env);
        $crate::module::define_module_properties(
          env,
          &mut *instance,
          &mut object,
          properties,
        )?;

        Ok(instance)
      }
    }

    #[napi]
    impl $module {
      #[napi]
      pub fn readable_identifier(&mut self) -> napi::Result<String> {
        self.module.readable_identifier()
      }

      #[napi(js_name = "_originalSource", ts_return_type = "JsSource", enumerable = false)]
      pub fn original_source(
        &mut self,
        env: &napi::Env,
      ) -> napi::Result<napi::Either<$crate::source::JsSourceToJs, ()>> {
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
      pub fn lib_ident<'a>(
        &mut self,
        env: &'a napi::Env,
        options: $crate::module::JsLibIdentOptions,
      ) -> napi::Result<Option<napi::JsString<'a>>> {
        self.module.lib_ident(env, options)
      }

      #[napi(
        js_name = "_emitFile",
        enumerable = false,
        ts_args_type = "filename: string, source: JsSource, assetInfo?: AssetInfo | undefined | null"
      )]
      pub fn emit_file(
        &mut self,
        env: &napi::Env,
        filename: String,
        source: $crate::source::JsSourceFromJs,
        asset_info: Option<napi::bindgen_prelude::Object>,
      ) -> napi::Result<()> {
        self
          .module
          .emit_file(env, filename, source, asset_info)
      }
    }
  };
}
