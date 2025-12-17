use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, PathData, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  RuntimeTemplate, get_filename_without_hash_length, impl_runtime_module,
};
use rspack_util::itoa;

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncWasmLoadingRuntimeModule {
  id: Identifier,
  generate_load_binary_code: String,
  generate_before_load_binary_code: String,
  generate_before_instantiate_streaming: String,
  supports_streaming: bool,
  chunk: ChunkUkey,
}

impl AsyncWasmLoadingRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    supports_streaming: bool,
    chunk: ChunkUkey,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}async_wasm_loading",
        runtime_template.runtime_module_prefix()
      )),
      generate_load_binary_code,
      Default::default(),
      Default::default(),
      supports_streaming,
      chunk,
    )
  }

  pub fn new_with_before_streaming(
    runtime_template: &RuntimeTemplate,
    generate_load_binary_code: String,
    generate_before_load_binary_code: String,
    generate_before_instantiate_streaming: String,
    supports_streaming: bool,
    chunk: ChunkUkey,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}async_wasm_loading",
        runtime_template.runtime_module_prefix()
      )),
      generate_load_binary_code,
      generate_before_load_binary_code,
      generate_before_instantiate_streaming,
      supports_streaming,
      chunk,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncWasmLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let (fake_filename, hash_len_map) =
      get_filename_without_hash_length(&compilation.options.output.webassembly_module_filename);

    // Even use content hash when [hash] in webpack
    let hash = match hash_len_map
      .get("[contenthash]")
      .or(hash_len_map.get("[hash]"))
    {
      Some(hash_len) => {
        let mut hash_len_buffer = itoa::Buffer::new();
        let hash_len_str = hash_len_buffer.format(*hash_len);
        format!("\" + wasmModuleHash.slice(0, {}) + \"", hash_len_str)
      }
      None => "\" + wasmModuleHash + \"".to_string(),
    };

    let chunk = compilation.chunk_by_ukey.expect_get(&self.chunk);
    let path = compilation
      .get_path(
        &fake_filename,
        PathData::default()
          .hash(&hash)
          .content_hash(&hash)
          .id(&PathData::prepare_id("\" + wasmModuleId + \""))
          .runtime(chunk.runtime().as_str()),
      )
      .await?;

    Ok(get_async_wasm_loading(
      &self
        .generate_load_binary_code
        .cow_replace(
          "$IMPORT_META_NAME",
          compilation.options.output.import_meta_name.as_str(),
        )
        .cow_replace("$PATH", &format!("\"{path}\"")),
      &self
        .generate_before_load_binary_code
        .cow_replace("$PATH", &format!("\"{path}\"")),
      &self.generate_before_instantiate_streaming,
      self.supports_streaming,
      &compilation.runtime_template,
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
  runtime_template: &RuntimeTemplate,
) -> String {
  let fallback_code = r#"
          .then(function(x) { return x.arrayBuffer();})
          .then(function(bytes) { return WebAssembly.instantiate(bytes, importsObj);})
          .then(function(res) { return Object.assign(exports, res.instance.exports);});
"#;

  let streaming_code = format!(
    r#"
      return req.then(function(res) {{
        if (typeof WebAssembly.instantiateStreaming === "function") {{
{generate_before_instantiate_streaming}          return WebAssembly.instantiateStreaming(res, importsObj)
            .then(
              function(res) {{ return Object.assign(exports, res.instance.exports);}},
              function(e) {{
                if(res.headers.get("Content-Type") !== "application/wasm") {{
                  console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                  return fallback();
                }}
                throw e;
              }}
            );
        }}
        return fallback();
      }});
"#
  );
  let instantiate_wasm = runtime_template.render_runtime_globals(&RuntimeGlobals::INSTANTIATE_WASM);

  if supports_streaming {
    format!(
      r#"
    {instantiate_wasm} = function(exports, wasmModuleId, wasmModuleHash, importsObj) {{
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
    {instantiate_wasm} = function(exports, wasmModuleId, wasmModuleHash, importsObj) {{
      return {req}{fallback_code}
    }};
      "#
    )
  }
}
