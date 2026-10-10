#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_span;

use std::{
  env, io,
  path::{Path, PathBuf},
  process::{Command, ExitCode},
  sync::Arc,
};

use rustc_span::source_map::{FileLoader, RealFileLoader};

struct SourceLoader {
  package: PathBuf,
  crate_name: String,
  crate_path: &'static str,
  generated: Option<PathBuf>,
}

impl FileLoader for SourceLoader {
  fn file_exists(&self, path: &Path) -> bool {
    RealFileLoader.file_exists(path)
  }

  fn read_file(&self, path: &Path) -> io::Result<String> {
    let source = RealFileLoader.read_file(path)?;
    // Never instrument registry dependencies, generated OUT_DIR files, or text assets.
    if path.extension().is_some_and(|extension| extension == "rs")
      && path.canonicalize()?.starts_with(&self.package)
      && !self
        .generated
        .as_ref()
        .is_some_and(|directory| path.starts_with(directory))
    {
      rspack_allocative_driver::instrument_crate(&source, self.crate_path, &self.crate_name)
        .map_err(|error| {
          io::Error::other(format!(
            "allocative instrumentation of {}: {error}",
            path.display()
          ))
        })
    } else {
      Ok(source)
    }
  }

  fn read_binary_file(&self, path: &Path) -> io::Result<Arc<[u8]>> {
    RealFileLoader.read_binary_file(path)
  }

  fn current_directory(&self) -> io::Result<PathBuf> {
    RealFileLoader.current_directory()
  }
}

struct Callbacks {
  package: PathBuf,
  crate_name: String,
  crate_path: &'static str,
}

impl rustc_driver::Callbacks for Callbacks {
  fn config(&mut self, config: &mut rustc_interface::interface::Config) {
    config.file_loader = Some(Box::new(SourceLoader {
      package: self.package.clone(),
      crate_name: self.crate_name.clone(),
      crate_path: self.crate_path,
      generated: env::var_os("OUT_DIR").map(PathBuf::from),
    }));
  }
}

fn main() -> ExitCode {
  match run() {
    Ok(code) => code,
    Err(error) => {
      eprintln!("rspack-allocative-driver: {error}");
      ExitCode::FAILURE
    }
  }
}

fn run() -> io::Result<ExitCode> {
  // Cargo's workspace wrapper receives the actual compiler as its first argument.
  let mut args: Vec<String> = env::args().skip(1).collect();
  let compiler = args
    .first()
    .ok_or_else(|| io::Error::other("expected a rustc path"))?
    .clone();
  let enabled = args.iter().enumerate().any(|(index, arg)| {
    arg == "--cfg=allocative"
      || (arg == "--cfg"
        && args
          .get(index + 1)
          .is_some_and(|value| value == "allocative"))
  });
  let dependency = |name: &str| {
    args.iter().any(|arg| {
      arg.starts_with(&format!("{name}=")) || arg.starts_with(&format!("--extern={name}="))
    })
  };
  let crate_path = if dependency("allocative") {
    Some("::allocative")
  } else if dependency("rspack_util") {
    Some("::rspack_util::allocative")
  } else if dependency("rspack_core") {
    Some("::rspack_core::allocative")
  } else {
    None
  };
  let Some(crate_path) = crate_path.filter(|_| enabled) else {
    let status = Command::new(&compiler).args(&args[1..]).status()?;
    return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
  };
  if Path::new(&compiler)
    .file_stem()
    .is_some_and(|name| name != "rustc")
  {
    return Err(io::Error::other(
      "allocative instrumentation requires rustc; run Clippy with snapshots disabled",
    ));
  }
  let package = env::var_os("CARGO_MANIFEST_DIR")
    .ok_or_else(|| io::Error::other("Cargo must set CARGO_MANIFEST_DIR"))?;
  if !args
    .iter()
    .any(|arg| arg == "--sysroot" || arg.starts_with("--sysroot="))
  {
    let output = Command::new(&compiler)
      .args(["--print", "sysroot"])
      .output()?;
    if !output.status.success() {
      return Err(io::Error::other("rustc could not report its sysroot"));
    }
    args.push("--sysroot".to_owned());
    args.push(
      String::from_utf8(output.stdout)
        .map_err(io::Error::other)?
        .trim()
        .to_owned(),
    );
  }
  let mut callbacks = Callbacks {
    package: PathBuf::from(package).canonicalize()?,
    // Use rustc's crate name so integration targets cannot match a library's trait policy.
    crate_name: args
      .windows(2)
      .find(|pair| pair[0] == "--crate-name")
      .map(|pair| pair[1].clone())
      .ok_or_else(|| io::Error::other("Cargo must pass --crate-name"))?,
    crate_path,
  };
  Ok(rustc_driver::catch_with_exit_code(|| {
    rustc_driver::run_compiler(&args, &mut callbacks)
  }))
}
