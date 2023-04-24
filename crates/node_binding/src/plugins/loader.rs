use std::{fmt::Debug, path::Path, sync::Arc};

use rspack_binding_options::{JsLoaderAdapter, JsLoaderRunner};
use rspack_core::{BoxLoader, CompilerOptions, NormalModule, Plugin, ResolveResult, Resolver};
use rspack_error::{internal_error, Result};

pub struct JsLoaderResolver {
  pub js_loader_runner: JsLoaderRunner,
}

impl Debug for JsLoaderResolver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JsLoaderResolver")
      .field("js_loader_runner", &"..")
      .finish()
  }
}

#[async_trait::async_trait]
impl Plugin for JsLoaderResolver {
  async fn before_loaders(&self, module: &mut NormalModule) -> Result<()> {
    let old_loaders = module.loaders_mut_vec();
    if old_loaders.is_empty() || old_loaders.len() == 1 {
      return Ok(());
    }

    let mut loaders: Vec<BoxLoader> = vec![];

    let mut starting_point = 0;
    for (idx, l) in old_loaders.iter().enumerate() {
      if l.identifier().starts_with("builtin:") {
        // Compose JS loaders
        let composed = old_loaders[starting_point..idx]
          .iter()
          .map(|l| l.identifier().as_str())
          .collect::<Vec<_>>()
          .join("$");
        loaders.push(Arc::new(JsLoaderAdapter {
          runner: self.js_loader_runner.clone(),
          identifier: composed.into(),
        }));
        loaders.push(l.clone());
        starting_point = idx + 1;
      }
    }
    // Compose the rest of JS loaders
    if starting_point < old_loaders.len() {
      let composed = old_loaders[starting_point..]
        .iter()
        .map(|l| l.identifier().as_str())
        .collect::<Vec<_>>()
        .join("$");
      loaders.push(Arc::new(JsLoaderAdapter {
        runner: self.js_loader_runner.clone(),
        identifier: composed.into(),
      }));
    }

    *module.loaders_mut_vec() = loaders;

    Ok(())
  }

  async fn resolve_loader(
    &self,
    _compiler_options: &CompilerOptions,
    context: &Path,
    resolver: &Resolver,
    loader_request: &str,
  ) -> Result<Option<BoxLoader>> {
    if loader_request.starts_with("builtin:") {
      // builtin loaders are not supported.
      // TODO: Options have to be serializable.
      return Ok(None);
    }

    let mut rest = None;
    let loader_request = if let Some(index) = loader_request.find('?') {
      rest = Some(&loader_request[index..]);
      Path::new(&loader_request[0..index])
    } else {
      Path::new(loader_request)
    };
    let resolve_result = resolver
      .resolve(context, &loader_request.to_string_lossy())
      .map_err(|err| {
        let loader_request = loader_request.display();
        let context = context.display();
        internal_error!("Failed to resolve loader: {loader_request} in {context} {err:?}")
      })?;

    match resolve_result {
      ResolveResult::Resource(resource) => {
        let resource = resource.path.to_string_lossy().to_string() + rest.unwrap_or_default();
        Ok(Some(Arc::new(JsLoaderAdapter {
          identifier: resource.into(),
          runner: self.js_loader_runner.clone(),
        })))
      }
      ResolveResult::Ignored => {
        let loader_request = loader_request.display();
        Err(internal_error!(
          "Failed to resolve loader: {loader_request}"
        ))
      }
    }
  }
}
