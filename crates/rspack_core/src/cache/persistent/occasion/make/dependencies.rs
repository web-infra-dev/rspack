use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use rspack_cacheable::{
  cacheable, from_bytes, to_bytes, with::Inline, DeserializeError, SerializeError,
};
use rspack_paths::ArcPath;
use rustc_hash::FxHashMap as HashMap;

use super::Storage;
use crate::FileCounter;

const SCOPE: &str = "occasion_make_dependencies";

/// Dependency type
#[cacheable]
enum DepType {
  /// file_dependencies
  File,
  /// context_dependencies
  Context,
  /// missing_dependencies
  Missing,
  /// build_dependencies
  Build,
}

/// The key struct of current storage scope
#[cacheable]
struct Dependency {
  r#type: DepType,
  path: ArcPath,
}

#[cacheable(as=Dependency)]
struct DependencyRef<'a> {
  r#type: DepType,
  #[cacheable(with=Inline)]
  path: &'a ArcPath,
}

pub fn save_dependencies_info(
  file_dependencies: &FileCounter,
  context_dependencies: &FileCounter,
  missing_dependencies: &FileCounter,
  build_dependencies: &FileCounter,
  storage: &Arc<dyn Storage>,
) -> Result<(), SerializeError> {
  let f = file_dependencies
    .updated_files_count_info()
    .map(|(path, count)| {
      (
        DependencyRef {
          r#type: DepType::File,
          path,
        },
        count,
      )
    });

  let c = context_dependencies
    .updated_files_count_info()
    .map(|(path, count)| {
      (
        DependencyRef {
          r#type: DepType::Context,
          path,
        },
        count,
      )
    });

  let m = missing_dependencies
    .updated_files_count_info()
    .map(|(path, count)| {
      (
        DependencyRef {
          r#type: DepType::Missing,
          path,
        },
        count,
      )
    });

  let b = build_dependencies
    .updated_files_count_info()
    .map(|(path, count)| {
      (
        DependencyRef {
          r#type: DepType::Build,
          path,
        },
        count,
      )
    });

  f.chain(c)
    .chain(m)
    .chain(b)
    .par_bridge()
    .try_for_each(|(dep_ref, count)| {
      let dep_ref = to_bytes(&dep_ref, &())?;
      if count == 0 {
        storage.remove(SCOPE, &dep_ref);
      } else {
        storage.set(SCOPE, dep_ref, count.to_ne_bytes().to_vec());
      }
      Ok(())
    })
}

pub async fn recovery_dependencies_info(
  storage: &Arc<dyn Storage>,
) -> Result<(FileCounter, FileCounter, FileCounter, FileCounter), DeserializeError> {
  let file_dep = Mutex::new(HashMap::default());
  let context_dep = Mutex::new(HashMap::default());
  let missing_dep = Mutex::new(HashMap::default());
  let build_dep = Mutex::new(HashMap::default());
  storage
    .load(SCOPE)
    .await
    .unwrap_or_default()
    .into_par_iter()
    .try_for_each(|(k, v)| {
      let count = usize::from_ne_bytes(
        v.as_ref()
          .clone()
          .try_into()
          .map_err(|_| DeserializeError::MessageError("deserialize count failed"))?,
      );
      let Dependency { r#type, path } = from_bytes(&k, &())?;
      match r#type {
        DepType::File => file_dep
          .lock()
          .expect("should get file dep")
          .insert(path, count),
        DepType::Context => context_dep
          .lock()
          .expect("should get context dep")
          .insert(path, count),
        DepType::Missing => missing_dep
          .lock()
          .expect("should get missing dep")
          .insert(path, count),
        DepType::Build => build_dep
          .lock()
          .expect("should get build dep")
          .insert(path, count),
      };
      Ok(())
    })?;

  Ok((
    FileCounter::new(file_dep.into_inner().expect("into_inner should be success")),
    FileCounter::new(
      context_dep
        .into_inner()
        .expect("into_inner should be success"),
    ),
    FileCounter::new(
      missing_dep
        .into_inner()
        .expect("into_inner should be success"),
    ),
    FileCounter::new(
      build_dep
        .into_inner()
        .expect("into_inner should be success"),
    ),
  ))
}
