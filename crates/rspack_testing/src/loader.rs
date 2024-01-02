use std::{fmt::Debug, path::Path, sync::Arc};

use rspack_core::{
  BoxLoader, CompilerOptions, NormalModule, Plugin, Resolver, BUILTIN_LOADER_PREFIX,
};
use rspack_error::{internal_error, Result};
use rspack_loader_swc::SWC_LOADER_IDENTIFIER;

#[derive(Debug)]
pub struct BuiltinLoaderResolver;

fn get_builtin_loader(builtin: &str, options: Option<&str>) -> BoxLoader {
  if builtin.starts_with(SWC_LOADER_IDENTIFIER) {
    return Arc::new(
      rspack_loader_swc::SwcLoader::new(
        serde_json::from_str(options.unwrap_or("{}")).unwrap_or_else(|e| {
          panic!("Could not parse builtin:swc-loader options:{options:?},error: {e:?}")
        }),
      )
      .with_identifier(builtin.into()),
    );
  }

  unreachable!("Unexpected builtin loader: {builtin}")
}

#[async_trait::async_trait]
impl Plugin for BuiltinLoaderResolver {
  async fn before_loaders(&self, module: &mut NormalModule) -> Result<()> {
    let contains_inline = module.contains_inline_loader();

    if contains_inline {
      return Err(internal_error!(
        "Inline loaders are not supported for builtin loaders"
      ));
    }

    Ok(())
  }

  async fn resolve_loader(
    &self,
    _compiler_options: &CompilerOptions,
    _context: &Path,
    _resolver: &Resolver,
    loader_request: &str,
    loader_options: Option<&str>,
  ) -> Result<Option<BoxLoader>> {
    if loader_request.starts_with(BUILTIN_LOADER_PREFIX) {
      return Ok(Some(get_builtin_loader(loader_request, loader_options)));
    }

    Err(internal_error!(
      "JS loaders are not supported in Rust tests: {loader_request}"
    ))
  }
}
