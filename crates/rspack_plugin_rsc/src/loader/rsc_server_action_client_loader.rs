use itertools::Itertools;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};

use crate::{
  utils::{has_server_directive, server_action::generate_action_id},
  RSCAdditionalData,
};

#[cacheable]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCServerActionClientLoaderOptions {
  server_proxy: String,
}

#[cacheable]
#[derive(Debug)]
pub struct RSCServerActionClientLoader {
  identifier: Identifier,
  options: RSCServerActionClientLoaderOptions,
}

impl RSCServerActionClientLoader {
  pub fn new(options: RSCServerActionClientLoaderOptions) -> Self {
    Self {
      identifier: RSC_SERVER_ACTION_CLIENT_LOADER_IDENTIFIER.into(),
      options: options.into(),
    }
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:rsc-server-action-client-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(RSC_SERVER_ACTION_CLIENT_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

pub const RSC_SERVER_ACTION_CLIENT_LOADER_IDENTIFIER: &str =
  "builtin:rsc-server-action-client-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for RSCServerActionClientLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let source = content.try_into_string()?;
    let resource_path_str = loader_context
      .resource_path()
      .and_then(|f| Some(f.as_str()))
      .unwrap_or("");
    let resource_query_str = loader_context.resource_query().unwrap_or("");
    let resource = format!("{}{}", resource_path_str, resource_query_str);

    let rsc_info = loader_context
      .additional_data()
      .and_then(|data| data.get::<RSCAdditionalData>());
    if let Some(RSCAdditionalData {
      directives,
      exports,
    }) = rsc_info
    {
      if has_server_directive(directives) {
        let mut has_default = false;
        let mut source = format!(
          r#"
import {{ createServerReference }} from "{}";
        "#,
          self.options.server_proxy,
        );
        let code = exports
          .iter()
          .map(|f| {
            let id = generate_action_id(&resource, &f.n);
            if f.n.eq("default") {
              has_default = true;
              format!(r#"const _default = createServerReference("{}");"#, id)
            } else {
              format!(r#"export const {} = createServerReference("{}");"#, f.n, id)
            }
          })
          .join("\n");
        source = format!("{}{}", source, code);
        if has_default {
          source += r#"
export default _default;
"#
        }
        loader_context.finish_with(source);
      } else {
        loader_context.finish_with(source);
      }
    } else {
      loader_context.finish_with(source);
    }
    Ok(())
  }
}

impl Identifiable for RSCServerActionClientLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
