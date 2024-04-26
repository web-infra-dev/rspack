use std::path::PathBuf;

use derivative::Derivative;
use napi::{bindgen_prelude::Buffer, Either};
use napi_derive::napi;
use rspack_core::rspack_sources::RawSource;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_copy::{
  CopyGlobOptions, CopyPattern, CopyRspackPluginOptions, Info, Related, ToType, Transformer,
};

type RawTransformer = ThreadsafeFunction<(String, String), Either<String, Buffer>>;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawCopyPattern {
  pub from: String,
  pub to: Option<String>,
  pub context: Option<String>,
  pub to_type: Option<String>,
  pub no_error_on_missing: bool,
  pub force: bool,
  pub priority: i32,
  pub glob_options: RawCopyGlobOptions,
  pub info: Option<RawInfo>,
  #[derivative(Debug = "ignore")]
  #[napi(ts_type = "(input: string, absoluteFilename: string) => string | Buffer")]
  pub transform: Option<RawTransformer>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawInfo {
  pub immutable: Option<bool>,
  pub minimized: Option<bool>,
  pub chunk_hash: Option<Vec<String>>,
  pub content_hash: Option<Vec<String>>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub related: Option<RawRelated>,
  pub version: Option<String>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawRelated {
  pub source_map: Option<String>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawCopyGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<String>>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCopyRspackPluginOptions {
  pub patterns: Vec<RawCopyPattern>,
}

impl From<RawCopyPattern> for CopyPattern {
  fn from(value: RawCopyPattern) -> Self {
    let RawCopyPattern {
      from,
      to,
      context,
      to_type,
      no_error_on_missing,
      force,
      priority,
      glob_options,
      info,
      transform,
    } = value;

    Self {
      from,
      to,
      context: context.map(PathBuf::from),
      to_type: if let Some(to_type) = to_type {
        match to_type.to_lowercase().as_str() {
          "dir" => Some(ToType::Dir),
          "file" => Some(ToType::File),
          "template" => Some(ToType::Template),
          _ => {
            //TODO how should we handle wrong input ?
            None
          }
        }
      } else {
        None
      },
      no_error_on_missing,
      info: info.map(Into::into),
      force,
      priority,
      glob_options: CopyGlobOptions {
        case_sensitive_match: glob_options.case_sensitive_match,
        dot: glob_options.dot,
        ignore: glob_options.ignore.map(|ignore| {
          ignore
            .into_iter()
            .map(|filter| glob::Pattern::new(filter.as_ref()).expect("Invalid pattern option"))
            .collect()
        }),
      },
      transform: transform.map(|transformer| {
        Transformer::Fn(Box::new(move |input, absolute_filename| {
          let f = transformer.clone();

          fn convert_to_enum(input: Either<String, Buffer>) -> RawSource {
            match input {
              Either::A(s) => RawSource::Source(s),
              Either::B(b) => RawSource::Buffer(b.to_vec()),
            }
          }

          Box::pin(async move {
            f.call((input.to_owned(), absolute_filename.to_owned()))
              .await
              .map(convert_to_enum)
          })
        }))
      }),
    }
  }
}

impl From<RawCopyRspackPluginOptions> for CopyRspackPluginOptions {
  fn from(val: RawCopyRspackPluginOptions) -> Self {
    Self {
      patterns: val.patterns.into_iter().map(Into::into).collect(),
    }
  }
}

impl From<RawInfo> for Info {
  fn from(value: RawInfo) -> Self {
    Self {
      immutable: value.immutable,
      minimized: value.minimized,
      chunk_hash: value.chunk_hash,
      content_hash: value.content_hash,
      development: value.development,
      hot_module_replacement: value.hot_module_replacement,
      related: value.related.map(|r| Related {
        source_map: r.source_map,
      }),
      version: value.version,
    }
  }
}
