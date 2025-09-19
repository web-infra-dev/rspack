use std::sync::Arc;

use rspack_core::{
  AfterResolveResult, BeforeResolveResult, ContextElementDependency,
  ContextModuleFactoryAfterResolve, ContextModuleFactoryBeforeResolve, DependencyId,
  DependencyType, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

pub struct ContextReplacementPluginOptions {
  pub resource_reg_exp: RspackRegex,
  pub new_content_resource: Option<String>,
  pub new_content_recursive: Option<bool>,
  pub new_content_reg_exp: Option<RspackRegex>,
  pub new_content_create_context_map: Option<HashMap<String, String>>,
}

#[plugin]
#[derive(Debug)]
pub struct ContextReplacementPlugin {
  resource_reg_exp: RspackRegex,
  new_content_resource: Option<String>,
  new_content_recursive: Option<bool>,
  new_content_reg_exp: Option<RspackRegex>,
  new_content_create_context_map: Option<HashMap<String, String>>,
}

impl ContextReplacementPlugin {
  pub fn new(options: ContextReplacementPluginOptions) -> Self {
    Self::new_inner(
      options.resource_reg_exp,
      options.new_content_resource,
      options.new_content_recursive,
      options.new_content_reg_exp,
      options.new_content_create_context_map,
    )
  }
}

#[plugin_hook(ContextModuleFactoryBeforeResolve for ContextReplacementPlugin)]
async fn cmf_before_resolve(&self, mut result: BeforeResolveResult) -> Result<BeforeResolveResult> {
  if let BeforeResolveResult::Data(data) = &mut result {
    if self.resource_reg_exp.test(&data.request)
      && let Some(new_content_resource) = &self.new_content_resource
    {
      data.request = new_content_resource.clone();
    }
    if let Some(new_content_recursive) = self.new_content_recursive {
      data.recursive = new_content_recursive;
    }
    if let Some(new_content_reg_exp) = &self.new_content_reg_exp {
      data.reg_exp = Some(new_content_reg_exp.clone());
    }
    // if let Some(new_content_callback) = &self.new_content_after_resolve_callback {
    //   new_content_callback(&mut result).await?;
    // } else {
    for d in &mut data.dependencies {
      if let Some(d) = d.as_context_dependency_mut() {
        *d.critical_mut() = None;
      }
    }
    // }
  }

  Ok(result)
}

#[plugin_hook(ContextModuleFactoryAfterResolve for ContextReplacementPlugin)]
async fn cmf_after_resolve(&self, mut result: AfterResolveResult) -> Result<AfterResolveResult> {
  if let AfterResolveResult::Data(data) = &mut result
    && self.resource_reg_exp.test(data.resource.as_str())
  {
    if let Some(new_content_resource) = &self.new_content_resource {
      if new_content_resource.starts_with('/') || new_content_resource.chars().nth(1) == Some(':') {
        data.resource = new_content_resource.clone().into();
      } else {
        data.resource = data.resource.join(Utf8PathBuf::from(new_content_resource));
      }
    }
    if let Some(new_content_recursive) = self.new_content_recursive {
      data.recursive = new_content_recursive;
    }
    if let Some(new_content_reg_exp) = &self.new_content_reg_exp {
      data.reg_exp = Some(new_content_reg_exp.clone());
    }
    if let Some(new_content_create_context_map) = &self.new_content_create_context_map {
      let new_content_create_context_map = new_content_create_context_map.clone();
      data.resolve_dependencies = Arc::new(move |options| {
        let deps = new_content_create_context_map
          .iter()
          .map(|(key, value)| {
            let request = format!(
              "{}{}{}",
              value,
              options.resource_query.clone(),
              options.resource_fragment.clone(),
            );

            let resource_identifier = ContextElementDependency::create_resource_identifier(
              options.resource.as_str(),
              &request,
              options.context_options.attributes.as_ref(),
            );
            ContextElementDependency {
              id: DependencyId::new(),
              request,
              user_request: key.to_string(),
              category: options.context_options.category,
              context: options.resource.clone().into(),
              layer: options.layer.clone(),
              options: options.context_options.clone(),
              resource_identifier,
              attributes: options.context_options.attributes.clone(),
              referenced_exports: options.context_options.referenced_exports.clone(),
              dependency_type: DependencyType::ContextElement(options.type_prefix),
              factorize_info: Default::default(),
            }
          })
          .collect::<Vec<_>>();
        Ok(deps)
      });
    }
    // if let Some(new_content_callback) = &self.new_content_callback {
    //   new_content_callback(&mut result).await?;
    // } else {
    for d in &mut data.dependencies {
      if let Some(d) = d.as_context_dependency_mut() {
        *d.critical_mut() = None;
      }
    }
    // }
  }
  Ok(result)
}

impl Plugin for ContextReplacementPlugin {
  fn name(&self) -> &'static str {
    "rspack.ContextReplacementPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .context_module_factory_hooks
      .before_resolve
      .tap(cmf_before_resolve::new(self));
    ctx
      .context_module_factory_hooks
      .after_resolve
      .tap(cmf_after_resolve::new(self));
    Ok(())
  }
}
