use cow_utils::CowUtils;
use derive_more::Debug;
use napi::{bindgen_prelude::Buffer, Either};
use napi_derive::napi;
use rspack_core::rspack_sources::RawSource;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_copy::{
  CopyGlobOptions, CopyPattern, CopyRspackPluginOptions, Info, Related, ToOption, ToType,
  Transformer,
};

type TransformerFn = ThreadsafeFunction<(Buffer, String), Either<String, Buffer>>;
type RawTransformer = Either<RawTransformOptions, TransformerFn>;

type RawToFn = ThreadsafeFunction<RawToOptions, String>;

type RawTo = Either<String, RawToFn>;

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawTransformOptions {
  #[debug(skip)]
  #[napi(
    ts_type = "{ transformer: (input: string, absoluteFilename: string) => string | Buffer | Promise<string> | Promise<Buffer>  }"
  )]
  pub transformer: TransformerFn,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawToOptions {
  pub context: String,
  pub absolute_filename: Option<String>,
}

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawCopyPattern {
  pub from: String,
  #[debug(skip)]
  #[napi(
    ts_type = "string | ((pathData: { context: string; absoluteFilename?: string }) => string | Promise<string>)"
  )]
  pub to: Option<RawTo>,
  pub context: Option<String>,
  pub to_type: Option<String>,
  pub no_error_on_missing: bool,
  pub force: bool,
  pub priority: i32,
  pub glob_options: RawCopyGlobOptions,
  pub info: Option<RawInfo>,
  /// Determines whether to copy file permissions from the source to the destination.
  /// When set to true, the plugin will preserve executable permissions and other file modes.
  /// This is particularly useful when copying scripts or executable files.
  /// @default false
  pub copy_permissions: Option<bool>,
  #[debug(skip)]
  #[napi(
    ts_type = "{ transformer: (input: string, absoluteFilename: string) => string | Buffer | Promise<string> | Promise<Buffer>  } | ((input: Buffer, absoluteFilename: string) => string | Buffer | Promise<string> | Promise<Buffer>)"
  )]
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
      copy_permissions,
      transform,
    } = value;

    Self {
      from,
      to: to.map(|to| match to {
        Either::A(s) => ToOption::String(s),
        Either::B(f) => ToOption::Fn(Box::new(move |ctx| {
          let f = f.clone();
          Box::pin(async move {
            f.call_with_sync(RawToOptions {
              context: ctx.context.as_str().to_owned(),
              absolute_filename: Some(ctx.absolute_filename.as_str().to_owned()),
            })
            .await
          })
        })),
      }),
      context: context.map(Into::into),
      to_type: if let Some(to_type) = to_type {
        match to_type.cow_to_ascii_lowercase().as_ref() {
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
      copy_permissions,
      transform: transform.map(|transformer| match transformer {
        Either::A(transformer_with_cache_options) => Transformer::Opt((
          Box::new(move |input, absolute_filename| {
            let f = transformer_with_cache_options.transformer.clone();

            fn convert_to_enum(input: Either<String, Buffer>) -> RawSource {
              match input {
                Either::A(s) => RawSource::from(s),
                Either::B(b) => RawSource::from(Vec::<u8>::from(b)),
              }
            }

            Box::pin(async move {
              f.call_with_sync((input.into(), absolute_filename.to_owned()))
                .await
                .map(convert_to_enum)
            })
          }),
          None, // transformer_with_cache_options.cache,
        )),
        Either::B(f) => Transformer::Fn(Box::new(move |input, absolute_filename| {
          let f = f.clone();

          fn convert_to_enum(input: Either<String, Buffer>) -> RawSource {
            match input {
              Either::A(s) => RawSource::from(s),
              Either::B(b) => RawSource::from(Vec::<u8>::from(b)),
            }
          }

          Box::pin(async move {
            f.call_with_sync((input.into(), absolute_filename.to_owned()))
              .await
              .map(convert_to_enum)
          })
        })),
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
