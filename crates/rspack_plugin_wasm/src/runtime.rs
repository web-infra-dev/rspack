use cow_utils::CowUtils;
use rspack_core::{
  Compilation, RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleStage, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncWasmLoadingRuntimeModule {
  generate_load_binary_code: String,
  generate_before_load_binary_code: String,
  generate_before_instantiate_streaming: String,
  supports_streaming: bool,
}

impl AsyncWasmLoadingRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    supports_streaming: bool,
  ) -> Self {
    Self::with_default(
      runtime_template,
      generate_load_binary_code,
      Default::default(),
      Default::default(),
      supports_streaming,
    )
  }

  pub fn new_with_before_streaming(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    generate_before_load_binary_code: String,
    generate_before_instantiate_streaming: String,
    supports_streaming: bool,
  ) -> Self {
    Self::with_default(
      runtime_template,
      generate_load_binary_code,
      generate_before_load_binary_code,
      generate_before_instantiate_streaming,
      supports_streaming,
    )
  }
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncWasmCompileRuntimeModule {
  generate_load_binary_code: String,
  generate_before_load_binary_code: String,
  generate_before_compile_streaming: String,
  supports_streaming: bool,
}

impl AsyncWasmCompileRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    supports_streaming: bool,
  ) -> Self {
    Self::with_default(
      runtime_template,
      generate_load_binary_code,
      Default::default(),
      Default::default(),
      supports_streaming,
    )
  }

  pub fn new_with_before_streaming(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    generate_before_load_binary_code: String,
    generate_before_compile_streaming: String,
    supports_streaming: bool,
  ) -> Self {
    Self::with_default(
      runtime_template,
      generate_load_binary_code,
      generate_before_load_binary_code,
      generate_before_compile_streaming,
      supports_streaming,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncWasmCompileRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::COMPILE_WASM },
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;

    Ok(get_async_wasm_compile(
      &self
        .generate_load_binary_code
        .cow_replace(
          "$IMPORT_META_NAME",
          compilation.options.output.import_meta_name.as_str(),
        )
        .cow_replace("$PATH", "wasmModuleFilename"),
      &self
        .generate_before_load_binary_code
        .cow_replace("$PATH", "wasmModuleFilename"),
      &self.generate_before_compile_streaming,
      self.supports_streaming,
      compilation.options.output.wasm_streaming_fallback,
      runtime_template,
    ))
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: RuntimeGlobals::INSTANTIATE_WASM,
      force_context: RuntimeGlobals::INSTANTIATE_WASM,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;

    Ok(get_async_wasm_loading(
      &self
        .generate_load_binary_code
        .cow_replace(
          "$IMPORT_META_NAME",
          compilation.options.output.import_meta_name.as_str(),
        )
        .cow_replace("$PATH", "wasmModuleFilename"),
      &self
        .generate_before_load_binary_code
        .cow_replace("$PATH", "wasmModuleFilename"),
      &self.generate_before_instantiate_streaming,
      self.supports_streaming,
      compilation.options.output.wasm_streaming_fallback,
      runtime_template,
    ))
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

fn get_async_wasm_loading(
  req: &str,
  generate_before_load_binary_code: &str,
  generate_before_instantiate_streaming: &str,
  supports_streaming: bool,
  streaming_fallback: bool,
  runtime_template: &RuntimeCodeTemplate,
) -> String {
  let fallback_code = r#"
          .then(function(x) { return x.arrayBuffer();})
          .then(function(bytes) { return WebAssembly.instantiate(bytes, importsObj);})
          .then(function(res) { return Object.assign(exports, res.instance.exports);});
"#;

  let streaming_instantiation = if streaming_fallback {
    r#"return WebAssembly.instantiateStreaming(res, importsObj)
            .then(
              function(res) { return Object.assign(exports, res.instance.exports);},
              function(e) {
                if(res.headers.get("Content-Type") !== "application/wasm") {
                  console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                  return fallback();
                }
                throw e;
              }
            );"#
  } else {
    r#"return WebAssembly.instantiateStreaming(res, importsObj)
            .then(function(res) { return Object.assign(exports, res.instance.exports);});"#
  };

  let streaming_code = format!(
    r#"
      return req.then(function(res) {{
        if (typeof WebAssembly.instantiateStreaming === "function") {{
{generate_before_instantiate_streaming}          {streaming_instantiation}
        }}
        return fallback();
      }});
"#
  );
  let instantiate_wasm =
    runtime_template.render_runtime_global_definition(&RuntimeGlobals::INSTANTIATE_WASM);

  if supports_streaming {
    format!(
      r#"
    {instantiate_wasm} = function(exports, wasmModuleFilename, importsObj) {{
      {generate_before_load_binary_code}
      var req = {req};
      var fallback = function() {{
        return req{fallback_code}
      }}
      {streaming_code}
    }};
"#
    )
  } else {
    let req = req.trim_end_matches(';');
    format!(
      r#"
    {instantiate_wasm} = function(exports, wasmModuleFilename, importsObj) {{
      return {req}{fallback_code}
    }};
      "#
    )
  }
}

fn get_async_wasm_compile(
  req: &str,
  generate_before_load_binary_code: &str,
  generate_before_compile_streaming: &str,
  supports_streaming: bool,
  streaming_fallback: bool,
  runtime_template: &RuntimeCodeTemplate,
) -> String {
  let fallback_code = format!(
    r#"
          .then({})
          .then({});
"#,
    runtime_template.basic_function("x", "return x.arrayBuffer();"),
    runtime_template.basic_function("bytes", "return WebAssembly.compile(bytes);")
  );

  let streaming_compilation = if streaming_fallback {
    r#"return WebAssembly.compileStreaming(res)
            .catch(function(e) {
              if(res.headers.get("Content-Type") !== "application/wasm") {
                console.warn("`WebAssembly.compileStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.compile` which is slower. Original error:\n", e);
                return fallback();
              }
              throw e;
            });"#
  } else {
    "return WebAssembly.compileStreaming(res);"
  };

  let streaming_code = format!(
    r#"
      return req.then(function(res) {{
        if (typeof WebAssembly.compileStreaming === "function") {{
{generate_before_compile_streaming}          {streaming_compilation}
        }}
        return fallback();
      }});
"#
  );
  let compile_wasm =
    runtime_template.render_runtime_global_definition(&RuntimeGlobals::COMPILE_WASM);

  if supports_streaming {
    format!(
      r#"
    {compile_wasm} = function(wasmModuleFilename) {{
      {generate_before_load_binary_code}
      var req = {req};
      var fallback = function() {{
        return req{fallback_code}
      }}
      {streaming_code}
    }};
"#
    )
  } else {
    let req = req.trim_end_matches(';');
    format!(
      r#"
    {compile_wasm} = function(wasmModuleFilename) {{
      return {req}{fallback_code}
    }};
      "#
    )
  }
}
