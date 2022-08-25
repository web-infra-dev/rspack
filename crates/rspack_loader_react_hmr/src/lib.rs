use rspack_core::{
   CompilationContext, CompilerContext, ResolveResult, ResolverFactory,
};
use rspack_error::Result;
use rspack_loader_runner::{Loader, LoaderContext, LoaderResult};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ReactRefreshRuntimeLoader;

impl ReactRefreshRuntimeLoader {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait::async_trait]
impl Loader<CompilerContext, CompilationContext> for ReactRefreshRuntimeLoader {
  fn name(&self) -> &'static str {
    "react-refresh-runtime-loader"
  }

  async fn run(
    &self,
    context: &LoaderContext<'_, '_, CompilerContext, CompilationContext>,
  ) -> Result<Option<LoaderResult>> {
    if context.resource_path == HMR_ENTRY_PATH {
      Ok(Some(LoaderResult {
        content: HMR_ENTRY.to_string().into(),
        meta: None,
      }))
    } else if context.resource_path == HMR_RUNTIME_PATH {
      let react_refresh_runtime = load_hmr_runtime(&context.compiler_context.options.context);
      // let entry_code = context.source;
      Ok(Some(LoaderResult {
        content: react_refresh_runtime.into(),
        meta: None,
      }))
    } else {
      Ok(None)
    }
  }

  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    self
  }
}

fn load_hmr_runtime(context: &String) -> String {
  let resolver = ResolverFactory::new().get(Default::default());

  match resolver.resolve(Path::new(&context), "react-refresh/package.json") {
    Ok(ResolveResult::Info(info)) => {
      let path = info.path;
      format!(
        "{}\n{}",
        fs::read_to_string(
          path
            .parent()
            .unwrap()
            .join("cjs/react-refresh-runtime.development.js")
            .to_str()
            .unwrap()
        )
        .unwrap(),
        r#"function debounce(fn, delay) {
          var handle
          return () => {
            clearTimeout(handle)
            handle = setTimeout(fn, delay)
          }
        }
        exports.queueUpdate = debounce(exports.performReactRefresh, 16)
        export default exports
        "#
      )
    }
    _ => {
      panic!("Not found react-refresh, please install it.");
    }
  }
}

pub static HMR_ENTRY_PATH: &str = "/react-hmr-entry.js";

pub static HMR_RUNTIME_PATH: &str = "/@react-refresh.js";

pub static HMR_ENTRY: &str = r#"import RefreshRuntime from "/@react-refresh.js";
RefreshRuntime.injectIntoGlobalHook(globalThis);
globalThis.$RefreshReg$ = () => {};
globalThis.$RefreshSig$ = () => (type) => type;"#;
