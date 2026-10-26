use std::{
  env, fs,
  io::{BufWriter, Write},
  path::PathBuf,
  sync::LazyLock,
};

use napi_derive_backend::{Napi, ToTypeDef};

static PKG_NAME: LazyLock<String> =
  LazyLock::new(|| env::var("CARGO_PKG_NAME").expect("Expected `CARGO_PKG_NAME` to be set"));
static TYPE_DEF_FOLDER: LazyLock<Option<String>> =
  LazyLock::new(|| env::var("NAPI_TYPE_DEF_TMP_FOLDER").ok());

fn get_type_def_file() -> Option<PathBuf> {
  if let Some(folder) = TYPE_DEF_FOLDER.as_ref() {
    let file = PathBuf::from(folder).join(&*PKG_NAME);
    Some(file)
  } else {
    // the environment variable set by old `@napi-rs/cli`
    if env::var("TYPE_DEF_TMP_PATH").is_ok() {
      panic!(
        "[napi-rs] missing environment variables. please upgrade `@napi-rs/cli` to the latest version."
      );
    }
    None
  }
}

pub fn prepare_type_def_file() {
  remove_existed_def_file();
}

fn remove_existed_def_file() {
  if let Some(file) = get_type_def_file() {
    if file.exists() {
      if let Err(_e) = fs::remove_file(&file) {
        #[cfg(debug_assertions)]
        {
          println!("Failed to manipulate type def file {file:?}: {_e:?}");
        }
      }
    }
  }
}

pub fn output_type_def(napi: &Napi) {
  if let Some(file) = get_type_def_file() {
    if let Some(type_def) = napi.to_type_def() {
      fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file)
        .and_then(|file| {
          let mut writer = BufWriter::<fs::File>::new(file);
          writer.write_all(type_def.to_string().as_bytes())?;
          writer.write_all("\n".as_bytes())?;
          writer.flush()
        })
        .unwrap_or_else(|e| {
          println!("Failed to write type def file: {e:?}");
        });
    }
  }
}
