use std::{
  alloc::Global,
  path::{Path, PathBuf},
  sync::{Arc, LazyLock},
};

use anyhow::{Context, Error};
use rspack_util::{itoa, swc::minify_file_comments};
use swc_core::{
  base::config::{Config, ConfigFile, Options as SwcOptions, Rc},
  common::{FileName, FilePathMapping, Globals, Mark, SourceMap, GLOBALS},
};

fn parse_swcrc(s: &str) -> Result<Rc, Error> {
  fn convert_json_err(e: serde_json::Error) -> Error {
    let line = e.line();
    let column = e.column();

    let msg = match e.classify() {
      Category::Io => "io error",
      Category::Syntax => "syntax error",
      Category::Data => "unmatched data",
      Category::Eof => "unexpected eof",
    };
    Error::new(e).context(format!(
      "failed to deserialize .swcrc (json) file: {}: {}:{}",
      msg,
      itoa!(line),
      itoa!(column)
    ))
  }

  let v = parse_to_serde_value(
    s.trim_start_matches('\u{feff}'),
    &jsonc_parser::ParseOptions {
      allow_comments: true,
      allow_trailing_commas: true,
      allow_loose_object_property_names: false,
    },
  )?
  .ok_or_else(|| Error::msg("failed to deserialize empty .swcrc (json) file"))?;

  if let Ok(rc) = serde_json::from_value(v.clone()) {
    return Ok(rc);
  }

  serde_json::from_value(v)
    .map(Rc::Single)
    .map_err(convert_json_err)
}

use crate::ast::Ast;

pub struct JavaScriptCompiler {
  globals: Globals,
}

impl JavaScriptCompiler {
  pub fn new() -> Self {
    // Initialize globals for swc
    let globals = Globals::default();

    Self { globals }
  }

  pub fn parse<S: Into<String>>(
    &self,
    source: S,
    resouce_path: PathBuf,
    mut options: SwcOptions,
  ) -> Ast {
    self.run(|| {
      let top_level_mark = Mark::new();
      let unresolved_mark = Mark::new();

      options.top_level_mark = Some(top_level_mark);
      options.unresolved_mark = Some(unresolved_mark);
    });

    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    let fm = cm.new_source_file(Arc::new(FileName::Real(resouce_path)), source.into());

    // let config = Self::read_swc_config(&options, name)

    todo!("implement parse")
  }

  pub fn transform(&self) -> Ast {
    todo!("implement transform")
  }

  pub fn minify(&self) -> Ast {
    todo!("implement minify")
  }

  pub fn print(&self, ast: Ast) -> Result<PrintResult, Error> {
    todo!("implement print")
  }

  fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.globals, op)
  }

  fn read_swc_config(opts: &SwcOptions, name: &FileName) -> Result<Option<Config>, Error> {
    let load_swcrc = |path: &Path| {
      let content = std::fs::read_to_string(path).context("failed to read config (.swcrc) file")?;

      parse_swcrc(&content)
    };

    static CUR_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
      if cfg!(target_arch = "wasm32") {
        PathBuf::new()
      } else {
        ::std::env::current_dir().expect("should be available")
      }
    });

    let res: Result<_, Error> = {
      let SwcOptions {
        ref root,
        root_mode,
        swcrc,
        config_file,
        ..
      } = opts;

      let root = root.as_ref().unwrap_or(&CUR_DIR);

      let swcrc_path = match config_file {
        Some(ConfigFile::Str(s)) => Some(PathBuf::from(s.clone())),
        _ => {
          if *swcrc {
            if let FileName::Real(ref path) = name {
              find_swcrc(path, root, *root_mode)
            } else {
              None
            }
          } else {
            None
          }
        }
      };

      let config_file = match swcrc_path.as_deref() {
        Some(s) => Some(load_swcrc(s)?),
        _ => None,
      };
      let filename_path = match name {
        FileName::Real(p) => Some(&**p),
        _ => None,
      };

      if let Some(filename_path) = filename_path {
        if let Some(config) = config_file {
          let dir = swcrc_path
            .as_deref()
            .and_then(|p| p.parent())
            .expect(".swcrc path should have parent dir");

          let mut config = config
            .into_config(Some(filename_path))
            .context("failed to process config file")?;

          if let Some(c) = &mut config {
            if c.jsc.base_url != PathBuf::new() {
              let joined = dir.join(&c.jsc.base_url);
              c.jsc.base_url = if cfg!(target_os = "windows") && c.jsc.base_url.as_os_str() == "." {
                dir.canonicalize().with_context(|| {
                  format!(
                    "failed to canonicalize base url using the path of \
                                    .swcrc\nDir: {}\n(Used logic for windows)",
                    dir.display(),
                  )
                })?
              } else {
                joined.canonicalize().with_context(|| {
                  format!(
                    "failed to canonicalize base url using the path of \
                                    .swcrc\nPath: {}\nDir: {}\nbaseUrl: {}",
                    joined.display(),
                    dir.display(),
                    c.jsc.base_url.display()
                  )
                })?
              };
            }
          }

          return Ok(config);
        }

        let config_file = config_file.unwrap_or_default();
        let config = config_file.into_config(Some(filename_path))?;

        return Ok(config);
      }

      let config = match config_file {
        Some(config_file) => config_file.into_config(None)?,
        None => Rc::default().into_config(None)?,
      };

      match config {
        Some(config) => Ok(Some(config)),
        None => {
          anyhow::bail!("no config matched for file ({})", name)
        }
      }
    };

    res.with_context(|| format!("failed to read .swcrc file for input file at `{}`", name))
  }
}
