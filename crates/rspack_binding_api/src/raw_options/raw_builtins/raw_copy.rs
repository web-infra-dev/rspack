use cow_utils::CowUtils;
use derive_more::Debug;
use napi::{
  Either,
  bindgen_prelude::{Buffer, FnArgs, Promise},
};
use napi_derive::napi;
use rspack_core::rspack_sources::{RawBufferSource, RawStringSource, SourceExt};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_copy::{
  CopyGlobOptions, CopyPattern, CopyRspackPluginOptions, Info, Related, ToOption, ToType,
  TransformerFn,
};

type RawTransformer = ThreadsafeFunction<FnArgs<(Buffer, String)>, Promise<Either<String, Buffer>>>;

type RawToFn = ThreadsafeFunction<RawToOptions, String>;

type RawTo = Either<String, RawToFn>;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawToOptions {
  pub context: String,
  pub absolute_filename: Option<String>,
}

#[derive(Debug, Clone)]
#[napi(object, object_to_js = false)]
pub struct RawCopyPattern {
  /// The source path of the copy operation, which can be an absolute path, a relative
  /// path, or a glob pattern. It can refer to a file or a directory. If a relative path
  /// is passed, it is relative to the `context` option.
  /// @default undefined
  pub from: String,
  /// The destination path of the copy operation, which can be an absolute path, a
  /// relative path, or a template string. If not specified, it is equal to Rspack's
  /// `output.path`.
  /// @default Rspack's `output.path`
  #[debug(skip)]
  #[napi(
    ts_type = "string | ((pathData: { context: string; absoluteFilename?: string }) => string | Promise<string>)"
  )]
  pub to: Option<RawTo>,
  /// `context` is a path to be prepended to `from` and removed from the start of the
  /// result paths. `context` can be an absolute path or a relative path. If it is a
  /// relative path, then it will be converted to an absolute path based on Rspack's
  /// `context`.
  /// `context` should be explicitly set only when `from` contains a glob. Otherwise,
  /// `context` is automatically set based on whether `from` is a file or a directory:
  /// - If `from` is a file, then `context` is its directory. The result path will be
  /// the filename alone.
  /// - If `from` is a directory, then `context` equals `from`. The result paths will
  /// be the paths of the directory's contents (including nested contents), relative
  /// to the directory.
  /// @default Rspack's `context`
  pub context: Option<String>,
  /// Specify the type of [to](#to), which can be a directory, a file, or a template
  /// name in Rspack. If not specified, it will be automatically inferred.
  /// The automatic inference rules are as follows:
  /// - `dir`: If `to` has no extension, or ends on `/`.
  /// - `file`: If `to` is not a directory and is not a template.
  /// - `template`: If `to` contains a template pattern.
  /// @default undefined
  pub to_type: Option<String>,
  /// Whether to ignore the error if there are missing files or directories.
  /// @default false
  pub no_error_on_missing: bool,
  /// Whether to force the copy operation to overwrite the destination file if it
  /// already exists.
  /// @default false
  pub force: bool,
  /// The priority of the copy operation. The higher the priority, the earlier the copy
  /// operation will be executed. When `force` is set to `true`, if a matching file is
  /// found, the one with higher priority will overwrite the one with lower priority.
  /// @default 0
  pub priority: i32,
  /// Set the glob options for the copy operation.
  /// @default undefined
  pub glob_options: RawCopyGlobOptions,
  /// Allows to add some assets info to the copied files, which may affect some behaviors
  /// in the build process. For example, by default, the copied JS and CSS files will be
  /// minified by Rspack's minimizer, if you want to skip minification for copied files,
  /// you can set `info.minimized` to `true`.
  /// @default undefined
  pub info: Option<RawInfo>,
  /// Determines whether to copy file permissions from the source to the destination.
  /// When set to true, the plugin will preserve executable permissions and other file modes.
  /// This is particularly useful when copying scripts or executable files.
  /// @default false
  pub copy_permissions: Option<bool>,
  /// Allows to modify the file contents.
  /// @default undefined
  #[debug(skip)]
  #[napi(
    ts_type = "{ transformer: (input: Buffer, absoluteFilename: string) => string | Buffer | Promise<string> | Promise<Buffer>  } | ((input: Buffer, absoluteFilename: string) => string | Buffer | Promise<string> | Promise<Buffer>)"
  )]
  pub transform: Option<RawTransformer>,
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawInfo {
  pub immutable: Option<bool>,
  /// Whether to skip minification for the copied files.
  /// @default false
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
  /// Whether the match is case sensitive
  /// @default true
  pub case_sensitive_match: Option<bool>,
  /// Whether to match files starting with `.`
  /// @default true
  pub dot: Option<bool>,
  /// An array of strings in glob format, which can be used to ignore specific paths
  /// @default undefined
  pub ignore: Option<Vec<String>>,
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCopyRspackPluginOptions {
  /// An array of objects that describe the copy operations to be performed.
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
      transform_fn: transform.map(|transformer| -> TransformerFn {
        Box::new(move |input, absolute_filename| {
          let f = transformer.clone();
          Box::pin(async move {
            f.call_with_promise((input.into(), absolute_filename.to_owned()).into())
              .await
              .map(|input| match input {
                Either::A(s) => RawStringSource::from(s).boxed(),
                Either::B(b) => RawBufferSource::from(Vec::<u8>::from(b)).boxed(),
              })
          })
        })
      }),
      cache: None,
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
