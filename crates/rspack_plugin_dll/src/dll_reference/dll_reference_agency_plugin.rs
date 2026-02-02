use rspack_core::{Context, ExternalItem, ExternalItemValue, LibraryType, Plugin};
use rspack_error::Result;
use rspack_hook::plugin;
use rspack_plugin_externals::ExternalsPlugin;
use rustc_hash::FxHashMap as HashMap;

use super::delegated_plugin::{DelegatedPlugin, DelegatedPluginOptions};
use crate::{DllManifest, DllManifestContent};

#[derive(Debug, Clone)]
pub struct DllReferenceAgencyPluginOptions {
  pub context: Option<Context>,
  pub name: Option<String>,
  pub content: Option<DllManifestContent>,
  pub manifest: Option<DllManifest>,
  pub extensions: Vec<String>,
  pub scope: Option<String>,
  pub source_type: Option<LibraryType>,
  pub r#type: String,
}

#[plugin]
#[derive(Debug)]
pub struct DllReferenceAgencyPlugin {
  options: DllReferenceAgencyPluginOptions,
}

impl DllReferenceAgencyPlugin {
  pub fn new(options: DllReferenceAgencyPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

impl Plugin for DllReferenceAgencyPlugin {
  fn name(&self) -> &'static str {
    "rspack.DllReferenceAgencyPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    let mut name = self.options.name.clone();
    let mut source_type = self.options.source_type.clone();
    let mut resolved_content = self.options.content.clone();

    if let Some(manifest) = &self.options.manifest {
      if name.is_none() {
        name = manifest.name.clone();
      }

      if source_type.is_none() {
        source_type = manifest.r#type.clone();
      }

      if resolved_content.is_none() {
        resolved_content = Some(manifest.content.clone());
      }
    }

    let resolved_content = resolved_content.expect("Manifest should have content.");

    let name = name.expect("Should pass name or manifest should have the name attribute");

    let source = format!("dll-reference {name}");

    let mut external_item_object = HashMap::default();

    external_item_object.insert(source.clone(), ExternalItemValue::String(name));

    let external = ExternalItem::Object(external_item_object);

    ExternalsPlugin::new(source_type.unwrap_or("var".into()), vec![external], false).apply(ctx)?;

    DelegatedPlugin::new(DelegatedPluginOptions {
      source,
      r#type: self.options.r#type.clone(),
      scope: self.options.scope.clone(),
      content: resolved_content,
      extensions: self.options.extensions.clone(),
      context: self.options.context.clone(),
      compilation_context: ctx.compiler_options.context.clone(),
    })
    .apply(ctx)?;

    Ok(())
  }
}
