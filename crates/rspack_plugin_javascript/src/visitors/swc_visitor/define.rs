use std::{
  collections::{HashMap, HashSet},
  env,
  sync::Arc,
  usize,
};

use dashmap::DashMap;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rspack_core::Define;
use serde::{Deserialize, Serialize};
use swc_core::common::collections::{AHashMap, AHashSet};
use swc_core::common::{errors::Handler, SourceMap};
use swc_core::ecma::ast::Expr;
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::visit::Fold;
use swc_core::{
  common::FileName,
  ecma::{
    parser::{parse_file_as_expr, Syntax},
    transforms::optimization::{inline_globals2, GlobalExprMap},
    utils::NodeIgnoringSpan,
  },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GlobalInliningPassEnvs {
  List(AHashSet<String>),
  Map(AHashMap<JsWord, JsWord>),
}

impl Default for GlobalInliningPassEnvs {
  fn default() -> Self {
    let mut v = HashSet::default();
    v.insert(String::from("NODE_ENV"));
    v.insert(String::from("SWC_ENV"));

    GlobalInliningPassEnvs::List(v)
  }
}

impl GlobalPassOption {
  pub fn build(self, cm: &SourceMap, handler: &Handler) -> impl 'static + Fold {
    type ValuesMap = Arc<AHashMap<JsWord, Expr>>;

    fn expr(cm: &SourceMap, handler: &Handler, src: String) -> Box<Expr> {
      let fm = cm.new_source_file(FileName::Anon, src);

      let mut errors = vec![];
      let expr = parse_file_as_expr(
        &fm,
        Syntax::Es(Default::default()),
        Default::default(),
        None,
        &mut errors,
      );

      for e in errors {
        e.into_diagnostic(handler).emit()
      }

      match expr {
        Ok(v) => v,
        _ => panic!("{} is not a valid expression", fm.src),
      }
    }

    fn mk_map(
      cm: &SourceMap,
      handler: &Handler,
      values: impl Iterator<Item = (JsWord, JsWord)>,
      is_env: bool,
    ) -> ValuesMap {
      let mut m = HashMap::default();

      for (k, v) in values {
        let v = if is_env {
          format!("'{}'", v)
        } else {
          (*v).into()
        };
        let v_str = v.clone();

        let e = expr(cm, handler, v_str);

        m.insert((*k).into(), *e);
      }

      Arc::new(m)
    }

    let env_map = if cfg!(target_arch = "wasm32") {
      Arc::new(Default::default())
    } else {
      match &self.envs {
        GlobalInliningPassEnvs::List(env_list) => {
          static CACHE: Lazy<DashMap<Vec<String>, ValuesMap, ahash::RandomState>> =
            Lazy::new(Default::default);

          let cache_key = env_list.iter().cloned().collect::<Vec<_>>();
          if let Some(v) = CACHE.get(&cache_key).as_deref().cloned() {
            v
          } else {
            let map = mk_map(
              cm,
              handler,
              env::vars()
                .filter(|(k, _)| env_list.contains(k))
                .map(|(k, v)| (k.into(), v.into())),
              true,
            );
            CACHE.insert(cache_key, map.clone());
            map
          }
        }

        GlobalInliningPassEnvs::Map(map) => {
          static CACHE: Lazy<DashMap<Vec<(JsWord, JsWord)>, ValuesMap, ahash::RandomState>> =
            Lazy::new(Default::default);

          let cache_key = self
            .vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
          if let Some(v) = CACHE.get(&cache_key) {
            (*v).clone()
          } else {
            let map = mk_map(
              cm,
              handler,
              map.iter().map(|(k, v)| (k.clone(), v.clone())),
              false,
            );
            CACHE.insert(cache_key, map.clone());
            map
          }
        }
      }
    };

    let global_exprs = {
      static CACHE: Lazy<DashMap<Vec<(JsWord, JsWord)>, GlobalExprMap, ahash::RandomState>> =
        Lazy::new(Default::default);

      let cache_key = self
        .vars
        .iter()
        .filter(|(k, _)| k.contains('.'))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();

      if let Some(v) = CACHE.get(&cache_key) {
        (*v).clone()
      } else {
        let map = self
          .vars
          .iter()
          .filter(|(k, _)| k.contains('.'))
          .map(|(k, v)| {
            (
              NodeIgnoringSpan::owned(*expr(cm, handler, k.to_string())),
              *expr(cm, handler, v.to_string()),
            )
          })
          .collect::<AHashMap<_, _>>();
        let map = Arc::new(map);
        CACHE.insert(cache_key, map.clone());
        map
      }
    };

    let global_map = {
      static CACHE: Lazy<DashMap<Vec<(JsWord, JsWord)>, ValuesMap, ahash::RandomState>> =
        Lazy::new(Default::default);

      let cache_key = self
        .vars
        .iter()
        .filter(|(k, _)| !k.contains('.'))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
      if let Some(v) = CACHE.get(&cache_key) {
        (*v).clone()
      } else {
        let map = mk_map(
          cm,
          handler,
          self.vars.into_iter().filter(|(k, _)| !k.contains('.')),
          false,
        );
        CACHE.insert(cache_key, map.clone());
        map
      }
    };

    inline_globals2(env_map, global_map, global_exprs, Arc::new(self.typeofs))
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct GlobalPassOption {
  #[serde(default)]
  pub vars: IndexMap<JsWord, JsWord, ahash::RandomState>,
  #[serde(default)]
  pub envs: GlobalInliningPassEnvs,

  #[serde(default)]
  pub typeofs: AHashMap<JsWord, JsWord>,
}

pub fn define(opts: &Define, handler: &Handler, cm: &Arc<SourceMap>) -> impl Fold {
  let mut global_opts: GlobalPassOption = Default::default();
  for (key, value) in opts {
    global_opts
      .vars
      .insert(JsWord::from(key.as_str()), JsWord::from(value.as_str()));
  }
  global_opts.build(cm, handler)
}
