#![feature(let_chains)]

use std::default::Default;
use std::{
  env,
  iter::Peekable,
  path::{Path, PathBuf},
  sync::{mpsc, Arc},
};

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  rspack_sources::SourceMap, DependencyCategory, DependencyType, LoaderRunnerContext, Resolve,
  ResolveOptionsWithDependencyType, ResolveResult, Resolver, ResolverFactory,
};
use rspack_error::{
  errors_to_diagnostics, internal_error, Diagnostic, DiagnosticKind, Error, InternalError, Result,
  Severity, TraceableError,
};
use rspack_loader_runner::{Content, Identifiable, Identifier, Loader, LoaderContext};
use serde::{Deserialize, Serialize};
use str_indices::utf16;
use swc_core::base::config::{Config, Options, SourceMapsConfig};
use swc_core::base::{try_with_handler, Compiler};
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::{FileName, FilePathMapping, GLOBALS};
use swc_core::ecma::transforms::base::pass::noop;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SwcLoaderOptions {
  Swc_options: SwcOptions,
  // `None` means open or close source map depends on whether in production mode.
  source_map: Option<bool>,
  additional_data: Option<String>,
  rspack_importer: bool,
}

impl Default for SwcLoaderOptions {
  fn default() -> Self {
    Self {
      rspack_importer: true,
      source_map: Default::default(),
      additional_data: Default::default(),
      Swc_options: Default::default(),
    }
  }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SwcOptions {
  indented_syntax: Option<bool>,
  include_paths: Vec<PathBuf>,
  charset: Option<bool>,
  quiet_deps: Option<bool>,
  verbose: Option<bool>,
}

#[derive(Debug)]
pub struct SwcLoader {
  options: Config,
}

impl SwcLoader {
  pub fn new(options: Config) -> Self {
    Self { options }
  }
}

/// Get global sourcemap
pub fn compiler() -> Arc<Compiler> {
  static C: Lazy<Arc<Compiler>> = Lazy::new(|| {
    let cm = Arc::new(swc_core::common::SourceMap::new(FilePathMapping::empty()));
    Arc::new(Compiler::new(cm))
  });

  C.clone()
}

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for SwcLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    dbg!(&loader_context.content);
    dbg!(&loader_context.source_map);
    dbg!(&loader_context.resource_path);
    dbg!(&loader_context.resource_query);
    let resource_path = loader_context.resource_path;
    let content = loader_context
      .content
      .to_owned()
      .expect("content should available");
    let c = compiler();
    let mut errors: Vec<Error> = Default::default();

    GLOBALS.set(&Default::default(), || {
      match try_with_handler(c.cm.clone(), Default::default(), |handler| {
        c.run(|| {
          let fm = c.cm.new_source_file(
            FileName::Real(resource_path.clone().into()),
            content.try_into_string()?,
          );
          let comments = SingleThreadedComments::default();
          let out = match c.process_js_with_custom_pass(
            fm,
            None,
            handler,
            &Options {
              source_maps: Some(SourceMapsConfig::Bool(true)),
              ..Default::default()
            },
            comments,
            |a| noop(),
            |a| noop(),
          ) {
            Ok(out) => Some(out),
            Err(e) => {
              errors.push(Error::Anyhow { source: e });
              None
            }
          };
          if let Some(out) = out {
            loader_context.content = Some(out.code.into());
            loader_context.source_map = if let Some(map) = out.map {
              match SourceMap::from_json(&map).map_err(|e| internal_error!(e.to_string())) {
                Ok(map) => Some(map),
                Err(e) => {
                  errors.push(e);
                  None
                }
              }
            } else {
              None
            };
          }

          Ok(())
        })
      }) {
        Ok(_) => {}
        Err(e) => errors.push(Error::Anyhow { source: e }),
      }
    });
    loader_context
      .diagnostic
      .append(&mut errors_to_diagnostics(errors));
    Ok(())
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    "builtin:swc-loader".into()
  }
}
