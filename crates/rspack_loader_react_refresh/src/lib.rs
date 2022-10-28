use rspack_core::{
  BundleEntries, CompilationContext, CompilerContext, ResolveResult, ResolverFactory,
};
use rspack_error::Result;
use rspack_loader_runner::{Loader, LoaderContext, LoaderResult};
use std::path::Path;

#[derive(Debug, Default)]
pub struct ReactRefreshRuntimeLoader;

#[async_trait::async_trait]
impl Loader<CompilerContext, CompilationContext> for ReactRefreshRuntimeLoader {
  fn name(&self) -> &'static str {
    "react-refresh-loader"
  }

  async fn run(
    &self,
    context: &LoaderContext<'_, '_, CompilerContext, CompilationContext>,
  ) -> Result<Option<LoaderResult>> {
    // if !context.resource_path.contains("node_modules") {
    //   dbg!(&context.resource_path);
    // }
    if is_entry_uri(
      context.resource_path,
      &context.compiler_context.options.entry,
    ) {
      let react_refresh = load_hmr_runtime(&context.compiler_context.options.context);
      // TODO: need magic string
      let content = match &context.source {
        rspack_core::Content::String(content) => {
          format!("{}\n{}", react_refresh, content)
        }
        rspack_core::Content::Buffer(buffer) => {
          let content = String::from_utf8(buffer.to_vec()).unwrap();
          format!("{}\n{}", react_refresh, content)
        }
      };
      Ok(Some(LoaderResult {
        content: content.into(),
        meta: None,
        source_map: None,
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

fn load_hmr_runtime(context: &Path) -> String {
  let resolver = ResolverFactory::new(Default::default()).get(Default::default());

  match resolver.resolve(context, "react-refresh/runtime") {
    Ok(ResolveResult::Info(info)) => {
      let path = info.path;
      let runtime = std::fs::read_to_string(
        path
          .parent()
          .unwrap()
          .join("cjs/react-refresh-runtime.development.js")
          .to_str()
          .unwrap(),
      )
      .unwrap();
      let debounce = r#"
      function debounce(fn, delay) {
        var handle
        return () => {
          clearTimeout(handle)
          handle = setTimeout(fn, delay)
        }
      }
      exports.queueUpdate = debounce(exports.performReactRefresh, 16);
      "#;

      let inject_module = format!(
        "__rspack_runtime__.installedModules['{}'] = __rspack_runtime__.installedModules['{}'] ||",
        REACT_REFRESH_MODULE_ID, REACT_REFRESH_MODULE_ID
      );
      let inject_module = format!(
        "{} function (module, exports) {{\n {};\n{}; \n}}",
        inject_module, runtime, debounce
      );

      let inject_hook = r#"
        var RefreshRuntime = __rspack_require__('/react-refresh');

        RefreshRuntime.injectIntoGlobalHook(globalThis);
        globalThis.$RefreshReg$ = () => {};
        globalThis.$RefreshSig$ = () => (type) => type;
      "#;

      format!("{}\n{}", inject_module, inject_hook)
    }
    _ => {
      println!(
        "[warning]: Not found react-refresh in {}, please install it.",
        context.display()
      );
      String::new()
    }
  }
}

fn is_entry_uri(uri: &str, entires: &BundleEntries) -> bool {
  // TODO: hack: exclude useless chunk, remove it after array entires.
  if uri.contains("rspack-hot-update") || uri.contains("rspack-dev-client") {
    return false;
  }
  for value in entires.values() {
    if value.path.eq(uri) {
      return true;
    }
  }
  false
}

static REACT_REFRESH_MODULE_ID: &str = "/react-refresh";
