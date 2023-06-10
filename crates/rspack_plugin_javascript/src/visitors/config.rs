use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use swc_core::common::collections;
use swc_core::common::{comments::SingleThreadedComments, FileName, Mark, SourceMap};
use swc_core::ecma::loader::resolvers::lru::CachingResolver;
use swc_core::ecma::loader::resolvers::node::NodeModulesResolver;
use swc_core::ecma::loader::resolvers::tsc::TsConfigResolver;
use swc_core::ecma::transforms::base::feature::FeatureFlag;
use swc_core::ecma::transforms::base::pass::noop;
use swc_core::ecma::transforms::module;
use swc_core::ecma::transforms::module::path::NodeImportResolver;
use swc_core::ecma::transforms::module::rewriter::import_rewriter;
use swc_core::ecma::visit;

pub(crate) type CompiledPaths = Vec<(String, Vec<String>)>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ModuleConfig {
  #[serde(rename = "commonjs")]
  CommonJs(module::common_js::Config),
  #[serde(rename = "umd")]
  Umd(module::umd::Config),
  #[serde(rename = "amd")]
  Amd(module::amd::Config),
  #[serde(rename = "systemjs")]
  SystemJs(module::system_js::Config),
  #[serde(rename = "es6")]
  Es6,
  #[serde(rename = "nodenext")]
  NodeNext,
}

impl ModuleConfig {
  pub fn build<'cmt>(
    cm: Arc<SourceMap>,
    comments: Option<&'cmt SingleThreadedComments>,
    base_url: PathBuf,
    paths: CompiledPaths,
    base: &FileName,
    unresolved_mark: Mark,
    config: Option<ModuleConfig>,
    available_features: FeatureFlag,
  ) -> Box<dyn visit::Fold + 'cmt> {
    let base = match base {
      FileName::Real(v) if !paths.is_empty() => {
        FileName::Real(v.canonicalize().unwrap_or_else(|_| v.to_path_buf()))
      }
      _ => base.clone(),
    };
    let skip_resolver = base_url.as_os_str().is_empty() && paths.is_empty();

    match config {
      None | Some(ModuleConfig::Es6) | Some(ModuleConfig::NodeNext) => {
        if skip_resolver {
          Box::new(noop())
        } else {
          let resolver = build_resolver(base_url, paths);

          Box::new(import_rewriter(base, resolver))
        }
      }
      Some(ModuleConfig::CommonJs(config)) => {
        if skip_resolver {
          Box::new(module::common_js::common_js(
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        } else {
          let resolver = build_resolver(base_url, paths);
          Box::new(module::common_js::common_js_with_resolver(
            resolver,
            base,
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        }
      }
      Some(ModuleConfig::Umd(config)) => {
        if skip_resolver {
          Box::new(module::umd::umd(
            cm,
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        } else {
          let resolver = build_resolver(base_url, paths);

          Box::new(module::umd::umd_with_resolver(
            cm,
            resolver,
            base,
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        }
      }
      Some(ModuleConfig::Amd(config)) => {
        if skip_resolver {
          Box::new(module::amd::amd(
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        } else {
          let resolver = build_resolver(base_url, paths);

          Box::new(module::amd::amd_with_resolver(
            resolver,
            base,
            unresolved_mark,
            config,
            available_features,
            comments,
          ))
        }
      }
      Some(ModuleConfig::SystemJs(config)) => {
        if skip_resolver {
          Box::new(module::system_js::system_js(unresolved_mark, config))
        } else {
          let resolver = build_resolver(base_url, paths);

          Box::new(module::system_js::system_js_with_resolver(
            resolver,
            base,
            unresolved_mark,
            config,
          ))
        }
      }
    }
  }
}

type SwcImportResolver =
  Arc<NodeImportResolver<CachingResolver<TsConfigResolver<NodeModulesResolver>>>>;

fn build_resolver(base_url: PathBuf, paths: CompiledPaths) -> Box<SwcImportResolver> {
  static CACHE: Lazy<DashMap<(PathBuf, CompiledPaths), SwcImportResolver, ahash::RandomState>> =
    Lazy::new(Default::default);

  if let Some(cached) = CACHE.get(&(base_url.clone(), paths.clone())) {
    return Box::new((*cached).clone());
  }

  let r = {
    let r = TsConfigResolver::new(
      NodeModulesResolver::new(Default::default(), Default::default(), true),
      base_url.clone(),
      paths.clone(),
    );
    let r = CachingResolver::new(40, r);

    let r = NodeImportResolver::new(r);
    Arc::new(r)
  };

  CACHE.insert((base_url, paths), r.clone());

  Box::new(r)
}
