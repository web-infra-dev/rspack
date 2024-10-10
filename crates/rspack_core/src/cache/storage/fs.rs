use std::{
  fs::File,
  hash::Hasher,
  io::{BufRead, BufReader, BufWriter, Read, Write},
  path::PathBuf,
  sync::Mutex,
};

use rustc_hash::{FxHashMap as HashMap, FxHasher};

use super::Storage;

#[derive(Debug)]
pub struct FsStorage {
  location: PathBuf,
  inner: Mutex<HashMap<&'static str, HashMap<Vec<u8>, Option<Vec<u8>>>>>,
}

impl FsStorage {
  pub fn new(location: PathBuf) -> Self {
    Self {
      location,
      inner: Default::default(),
    }
  }
}

impl Storage for FsStorage {
  fn get_all(&self, scope: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    let dir_path = self.location.join(scope);
    if !dir_path.exists() {
      return vec![];
    }

    let mut res = vec![];
    for dir in dir_path.read_dir().unwrap() {
      if let Ok(path_entry) = dir {
        for dir in path_entry.path().read_dir().unwrap() {
          if let Ok(file_entry) = dir {
            let file_path = file_entry.path();
            let mut map = read_data(&file_path);
            res.extend(map.drain());
          }
        }
      }
    }
    res
  }
  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>) {
    let mut inner = self.inner.lock().unwrap();
    let scope_map = inner.entry(scope).or_default();
    scope_map.insert(key, Some(value));
  }
  fn remove(&self, scope: &'static str, key: &[u8]) {
    let mut inner = self.inner.lock().unwrap();
    let scope_map = inner.entry(scope).or_default();
    scope_map.insert(key.to_vec(), None);
  }
  fn idle(&self) {
    let location = self.location.clone();
    let data = std::mem::replace(&mut *self.inner.lock().unwrap(), Default::default());
    // TODO add save process manager
    tokio::spawn(async { save(location, data) });
  }
}

fn save(location: PathBuf, data: HashMap<&'static str, HashMap<Vec<u8>, Option<Vec<u8>>>>) {
  for (scope, mut map) in data {
    for (k, v) in map.drain() {
      let hash = {
        let mut hasher = FxHasher::default();
        hasher.write(&k);
        format!("{:016x}", hasher.finish())
      };
      let dir_path = location.join(scope).join(&hash[0..2]);
      std::fs::create_dir_all(&dir_path).unwrap();
      let file_path = dir_path.join(&hash[2..]);
      let mut data = read_data(&file_path);
      if let Some(v) = v {
        data.insert(k, v);
      } else {
        data.remove(&k);
      }
      write_data(&file_path, &data)
    }
  }
}

fn read_data(file_path: &PathBuf) -> HashMap<Vec<u8>, Vec<u8>> {
  if !file_path.exists() {
    return HashMap::default();
  }
  let mut reader = BufReader::new(File::open(file_path).unwrap());
  let mut meta_line = String::new();
  reader.read_line(&mut meta_line).unwrap();
  // remove \n in meta_line
  meta_line.pop();
  let meta_info: Vec<usize> = meta_line
    .split(" ")
    .map(|item| item.parse::<usize>().unwrap())
    .collect();

  let mut res = HashMap::default();
  let mut index = 0;
  while index < meta_info.len() {
    let key_len = *meta_info.get(index).unwrap();
    let value_len = *meta_info.get(index + 1).unwrap();
    index += 2;
    let mut key = vec![0u8; key_len];
    reader.read_exact(&mut key).unwrap();
    let mut value = vec![0u8; value_len];
    reader.read_exact(&mut value).unwrap();
    res.insert(key, value);
  }
  res
}

fn write_data(file_path: &PathBuf, data: &HashMap<Vec<u8>, Vec<u8>>) {
  if data.is_empty() {
    std::fs::remove_file(file_path).unwrap();
    return;
  }

  let mut writer = BufWriter::new(File::create(file_path).unwrap());
  let mut meta_info = Vec::with_capacity(data.len() * 2);
  for (k, v) in data {
    meta_info.push(k.len().to_string());
    meta_info.push(v.len().to_string());
  }

  writer
    .write_fmt(format_args!("{}\n", meta_info.join(" ")))
    .unwrap();
  for (k, v) in data {
    writer.write(&k).unwrap();
    writer.write(&v).unwrap();
  }
}
