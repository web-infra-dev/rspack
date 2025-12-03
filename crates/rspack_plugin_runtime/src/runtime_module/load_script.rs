use std::ptr::NonNull;

use rspack_collections::Identifier;
use rspack_core::{ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

use crate::{
  CreateScriptData, RuntimeModuleChunkWrapper, RuntimePlugin, get_chunk_runtime_requirements,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct LoadScriptRuntimeModule {
  id: Identifier,
  unique_name: String,
  with_create_script_url: bool,
  chunk_ukey: ChunkUkey,
}

impl LoadScriptRuntimeModule {
  pub fn new(unique_name: String, with_create_script_url: bool, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/load_script"),
      unique_name,
      with_create_script_url,
      chunk_ukey,
    )
  }
}

enum TemplateId {
  Raw,
  CreateScript,
}

#[async_trait::async_trait]
impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("runtime/load_script.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::CreateScript),
        include_str!("runtime/load_script_create_script.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &self.chunk_ukey);
    let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

    let unique_prefix = if self.unique_name.is_empty() {
      None
    } else {
      Some(format!(
        r#"var uniqueName = {};"#,
        serde_json::to_string(&format!("{}:", self.unique_name))
          .expect("failed to serialize unique prefix")
      ))
    };

    let create_script_code = compilation.runtime_template.render(
      &self.template_id(TemplateId::CreateScript),
      Some(serde_json::json!({
        "_script_type": &compilation.options.output.script_type,
        "_charset": compilation.options.output.charset,
        "_unique_prefix": unique_prefix.is_some(),
        "_with_fetch_priority": with_fetch_priority,
        "_with_create_script_url": self.with_create_script_url,
        "_cross_origin": compilation.options.output.cross_origin_loading.to_string(),
        "_chunk_load_timeout": compilation.options.output.chunk_load_timeout.saturating_div(1000).to_string(),
      })),
    )?;

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());
    let chunk_ukey = self.chunk_ukey;
    let res = hooks
      .borrow()
      .create_script
      .call(CreateScriptData {
        code: create_script_code,
        chunk: RuntimeModuleChunkWrapper {
          chunk_ukey,
          compilation_id: compilation.id(),
          compilation: NonNull::from(compilation),
        },
      })
      .await?;

    let render_source = compilation.runtime_template.render(
      &self.template_id(TemplateId::Raw),
      Some(serde_json::json!({
        "_unique_prefix": unique_prefix.unwrap_or_default(),
        "_create_script": res.code,
        "_chunk_load_timeout": compilation.options.output.chunk_load_timeout.to_string(),
        "_fetch_priority": if with_fetch_priority { ", fetchPriority" } else { "" },
      })),
    )?;

    Ok(render_source)
  }
}

impl LoadScriptRuntimeModule {
  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.to_string();

    match id {
      TemplateId::Raw => base_id,
      TemplateId::CreateScript => format!("{base_id}_create_script"),
    }
  }
}
