use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};

use crate::{export_visitor::DEFAULT_EXPORT, has_client_directive, RSCAdditionalData};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCProxyLoaderOptions {
  client_proxy: String,
}

#[derive(Debug)]
pub struct RSCProxyLoader {
  identifier: Identifier,
  options: RSCProxyLoaderOptions,
}

impl RSCProxyLoader {
  pub fn new(options: RSCProxyLoaderOptions) -> Self {
    Self {
      identifier: RSC_PROXY_LOADER_IDENTIFIER.into(),
      options: options.into(),
    }
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:rsc-proxy-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(RSC_PROXY_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

pub const RSC_PROXY_LOADER_IDENTIFIER: &str = "builtin:rsc-proxy-loader";

#[async_trait::async_trait]
impl Loader<RunnerContext> for RSCProxyLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let content = std::mem::take(&mut loader_context.content).expect("content should be available");
    let resource_path = loader_context.resource_path().and_then(|f| f.to_str());

    let rsc_info = loader_context.additional_data.get::<RSCAdditionalData>();
    if let Some(RSCAdditionalData {
      directives,
      exports,
    }) = rsc_info
    {
      if has_client_directive(directives) {
        let mut source = format!(
          r#"
import {{ createProxy }} from "{}"
const proxy = createProxy({:?})

// Accessing the __esModule property and exporting $$typeof are required here.
// The __esModule getter forces the proxy target to create the default export
// and the $$typeof value is for rendering logic to determine if the module
// is a client boundary.
const {{ __esModule, $$typeof }} = proxy;
const __default__ = proxy.default
        "#,
          self.options.client_proxy,
          resource_path.unwrap()
        );
        let mut cnt = 0;
        for export in exports.into_iter() {
          let n = &export.n;
          if n == "" {
            source += r#"\nexports[\'\'] = proxy[\'\'];"#;
          } else if n == DEFAULT_EXPORT {
            source += r#"
export { __esModule, $$typeof };
export default __default__;
            "#;
          } else {
            source += &format!(
              r#"
const e{} = proxy["{}"];
export {{ e{} as {} }};
            "#,
              cnt, n, cnt, n
            );
            cnt += 1;
          }
        }
        loader_context.content = Some(source.into());
      } else {
        loader_context.content = Some(content);
      }
    } else {
      loader_context.content = Some(content);
    }
    Ok(())
  }
}

impl Identifiable for RSCProxyLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
