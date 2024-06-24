use itertools::Itertools;
use rspack_core::RunnerContext;
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};

use crate::{
  utils::{has_server_directive, server_action::generate_action_id},
  RSCAdditionalData,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RSCServerActionClientLoaderOptions {
  server_proxy: String,
}

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

#[async_trait::async_trait]
impl Loader<RunnerContext> for RSCServerActionClientLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let resource_path = loader_context.resource_path().to_path_buf();
    let content = std::mem::take(&mut loader_context.content).expect("content should be available");

    let rsc_info = loader_context.additional_data.get::<RSCAdditionalData>();
    if let Some(RSCAdditionalData {
      directives,
      exports,
    }) = rsc_info
    {
      if has_server_directive(directives) {
        let mut source = format!(
          r#"
import {{ createServerReference }} from "{}";
        "#,
          self.options.server_proxy,
        );
        let code = exports
          .iter()
          .map(|f| {
            let id = generate_action_id(resource_path.to_str().unwrap(), &f.n);
            format!(r#"export const {} = createServerReference("{}");"#, f.n, id)
          })
          .join("\n");
        source = format!("{}{}", source, code);
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

impl Identifiable for RSCServerActionClientLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
