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
        own_property_names: &[&str],
      ) -> napi::Result<napi::bindgen_prelude::ClassInstance<'a, Self>> {
        use napi::bindgen_prelude::{JavaScriptClassExt, JsObjectValue};

        let mut instance = self.into_instance(env)?;
        let mut object = instance.as_object(env);
        $crate::module::define_module_instance_properties(env, &mut *instance, &mut object)?;
        $crate::module::define_own_properties_from_prototype(env, &mut object, own_property_names)?;

        Ok(instance)
      }
    }

    #[napi]
    impl $module {
      #[napi(skip_typescript, getter, js_name = "type")]
      pub fn module_type(&mut self) -> napi::Result<String> {
        self.module.get_module_type()
      }

      #[napi(skip_typescript, getter)]
      pub fn context(&mut self) -> napi::Result<napi::Either<String, ()>> {
        self.module.get_context()
      }

      #[napi(skip_typescript, getter)]
      pub fn layer(&mut self) -> napi::Result<napi::Either<String, ()>> {
        self.module.get_layer()
      }

      #[napi(skip_typescript, getter, js_name = "useSourceMap")]
      pub fn use_source_map(&mut self) -> napi::Result<bool> {
        self.module.get_use_source_map()
      }

      #[napi(skip_typescript, getter, js_name = "useSimpleSourceMap")]
      pub fn use_simple_source_map(&mut self) -> napi::Result<bool> {
        self.module.get_use_simple_source_map()
      }

      #[napi(skip_typescript, getter, js_name = "factoryMeta")]
      pub fn factory_meta(&mut self) -> napi::Result<$crate::module::JsFactoryMeta> {
        self.module.get_factory_meta()
      }

      #[napi(skip_typescript, setter)]
      pub fn set_factory_meta(
        &mut self,
        factory_meta: $crate::module::JsFactoryMeta,
      ) -> napi::Result<()> {
        self.module.set_factory_meta_value(factory_meta)
      }

      #[napi(skip_typescript, getter, js_name = "buildInfo")]
      pub fn build_info<'a>(
        &mut self,
        env: &'a napi::Env,
      ) -> napi::Result<napi::bindgen_prelude::Object<'a>> {
        self.module.get_build_info(env)
      }

      #[napi(skip_typescript, setter)]
      pub fn set_build_info(
        &mut self,
        env: &napi::Env,
        build_info: napi::bindgen_prelude::Object,
      ) -> napi::Result<()> {
        self.module.set_build_info_object(env, build_info)
      }

      #[napi(skip_typescript, getter, js_name = "buildMeta")]
      pub fn build_meta<'a>(
        &mut self,
        env: &'a napi::Env,
      ) -> napi::Result<napi::bindgen_prelude::Object<'a>> {
        self.module.get_build_meta(env)
      }

      #[napi(skip_typescript, setter)]
      pub fn set_build_meta(
        &mut self,
        env: &napi::Env,
        build_meta: napi::bindgen_prelude::Object,
      ) -> napi::Result<()> {
        self.module.set_build_meta_object(env, build_meta)
      }

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
