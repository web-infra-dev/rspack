use std::{
  ffi::{c_char, CStr, CString},
  sync::Arc,
};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, PluginContext, Resolver, RunnerContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};

#[cacheable]
pub struct TypeScriptLoader {
  identifier: Identifier,
}

impl Default for TypeScriptLoader {
  fn default() -> Self {
    Self {
      identifier: TYPESCRIPT_LOADER_IDENTIFIER.into(),
    }
  }
}

impl TypeScriptLoader {
  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:preact-refresh-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(TYPESCRIPT_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

extern "C" {
  fn TranspileModule(source: *const c_char) -> *const c_char;
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for TypeScriptLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let content = unsafe {
      TranspileModule(
        CString::new(content.try_into_string().unwrap())
          .unwrap()
          .as_ptr(),
      )
    };
    let content = unsafe { CStr::from_ptr(content) };

    loader_context.finish_with(content.to_string_lossy().to_string());
    Ok(())
  }
}

pub const TYPESCRIPT_LOADER_IDENTIFIER: &str = "builtin:ts-go-loader";

impl Identifiable for TypeScriptLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

#[plugin]
#[derive(Debug)]
pub struct TypeScriptLoaderPlugin;

impl TypeScriptLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for TypeScriptLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for TypeScriptLoaderPlugin {
  fn name(&self) -> &'static str {
    "TypeScriptLoaderPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for TypeScriptLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with(TYPESCRIPT_LOADER_IDENTIFIER) {
    let loader = Arc::new(TypeScriptLoader {
      identifier: loader_request.as_str().into(),
    });

    return Ok(Some(loader));
  }

  Ok(None)
}
