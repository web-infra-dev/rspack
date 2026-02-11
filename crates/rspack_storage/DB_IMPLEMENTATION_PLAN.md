# DB 实现计划

## 文件结构

```
crates/rspack_storage/src/db/
├── mod.rs                    # DB 主入口
├── options.rs                # DBOptions（已完成）
├── meta.rs                   # Meta 读写（已完成）
├── error.rs                  # DB 内部错误类型（新增）
├── bucket.rs                 # Bucket 操作
├── page.rs                   # Page 操作
├── pack/
│   ├── mod.rs                # Pack 序列化/反序列化
│   └── format.rs             # Pack 数据结构
├── index/
│   ├── mod.rs                # Index 读写
│   └── hash.rs               # Hash 生成
└── transaction/              # Transaction（需要修改）
    ├── mod.rs
    └── lock/
```

## 实现顺序

### Phase 1: Transaction 修改（当前）

- [x] 修改 `transaction/mod.rs` 的 begin() 恢复逻辑
- [x] 修改 commit() 添加 state.lock 清理
- [x] 添加 recover_commit() 方法
- [x] 简化进程检测逻辑

### Phase 2: 基础工具（pack + index）

- [ ] `db/error.rs` - 简单的内部错误类型
- [ ] `db/pack/format.rs` - Pack 数据结构
- [ ] `db/pack/mod.rs` - Pack 序列化/反序列化
- [ ] `db/index/hash.rs` - Hash 生成
- [ ] `db/index/mod.rs` - Index 读写

### Phase 3: Page 和 Bucket

- [ ] `db/page.rs` - Page 操作
- [ ] `db/bucket.rs` - Bucket 操作（包含 page 分配算法）

### Phase 4: DB 主入口

- [ ] `db/mod.rs` - DB 结构和 API
- [ ] 集成所有模块

## API 设计

### 最简 API

```rust
// 1. 创建 DB
let db = DB::new(root, options, fs)?;

// 2. 加载数据
let data = db.load("snapshot").await?;

// 3. 保存数据
db.save(HashMap::from([
  ("snapshot", HashMap::from([
    (key1, Some(value1)),  // 新增/更新
    (key2, None),          // 删除
  ])),
])).await?;
```

## 模块详细设计

### 1. `db/error.rs` - 内部错误

```rust
#[derive(Debug)]
pub enum Error {
  /// 配置错误
  InvalidConfig(String),

  /// 文件系统错误
  Fs(crate::fs::FSError),

  /// 数据损坏
  Corrupted(String),

  /// 其他错误
  Other(String),
}

impl From<crate::fs::FSError> for Error {
  fn from(e: crate::fs::FSError) -> Self {
    Self::Fs(e)
  }
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 2. `db/pack/format.rs` - Pack 数据结构

```rust
/// Pack 中的一个条目
pub struct PackEntry {
  pub key: Vec<u8>,
  pub value: Vec<u8>,
}

impl PackEntry {
  pub fn size(&self) -> usize {
    self.key.len() + self.value.len()
  }
}

/// Pack 文件内容
pub struct Pack {
  pub entries: Vec<PackEntry>,
}

impl Pack {
  pub fn new(entries: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
    let entries = entries
      .into_iter()
      .map(|(key, value)| PackEntry { key, value })
      .collect();
    Self { entries }
  }

  pub fn size(&self) -> usize {
    self.entries.iter().map(|e| e.size()).sum()
  }

  pub fn into_entries(self) -> Vec<(Vec<u8>, Vec<u8>)> {
    self.entries
      .into_iter()
      .map(|e| (e.key, e.value))
      .collect()
  }
}
```

### 3. `db/pack/mod.rs` - Pack 序列化

```rust
use super::error::Result;

/// 序列化 Pack
/// 格式：
/// Line 1: key sizes (space-separated)
/// Line 2: value sizes (space-separated)
/// Line 3+: binary data (all keys, then all values)
pub fn serialize(entries: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
  // 实现
}

/// 反序列化 Pack
pub fn deserialize(content: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
  // 实现
}
```

### 4. `db/index/hash.rs` - Hash 生成

```rust
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// 生成 exist_check_hash（按位或）
pub fn generate_exist_check_hash(keys: &[Vec<u8>]) -> u64 {
  let mut result = 0u64;
  for key in keys {
    let mut hasher = FxHasher::default();
    key.hash(&mut hasher);
    result |= hasher.finish();
  }
  result
}

/// 检查 key 是否可能存在
pub fn may_contain(key: &[u8], exist_check_hash: u64) -> bool {
  let mut hasher = FxHasher::default();
  key.hash(&mut hasher);
  let key_hash = hasher.finish();
  (key_hash & exist_check_hash) == key_hash
}

/// 生成 file_hash
pub fn generate_file_hash(content: &[u8]) -> String {
  let mut hasher = FxHasher::default();
  content.hash(&mut hasher);
  format!("{:x}", hasher.finish())
}
```

### 5. `db/index/mod.rs` - Index 读写

```rust
use crate::fs::{FileSystem, FSResult};
use rspack_paths::Utf8Path;

pub struct IndexEntry {
  pub pack_name: String,
  pub exist_check_hash: u64,
  pub file_hash: String,
}

/// 序列化 Index
/// 格式：pack_name,exist_check_hash,file_hash\n
pub fn serialize(entries: &[IndexEntry]) -> Vec<u8> {
  // 实现
}

/// 反序列化 Index
pub fn deserialize(content: &[u8]) -> super::error::Result<Vec<IndexEntry>> {
  // 实现
}

/// 读取 index 文件
pub async fn read_index(
  fs: &FileSystem,
  path: &Utf8Path,
) -> super::error::Result<Vec<IndexEntry>> {
  let mut reader = fs.read_file(path).await?;
  let content = reader.read_to_end().await?;
  deserialize(&content)
}

/// 写入 index 内容（返回字节，由调用方通过 transaction 写入）
pub fn write_index_content(entries: &[IndexEntry]) -> Vec<u8> {
  serialize(entries)
}
```

### 6. `db/page.rs` - Page 操作

```rust
use crate::db::{pack, index, error::Result};
use crate::fs::FileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashMap as HashMap;

pub struct Page {
  page_id: usize,
  page_path: Utf8PathBuf,
}

impl Page {
  /// 创建 Page
  pub fn new(bucket_path: &Utf8Path, page_id: usize, page_count: usize) -> Self {
    let page_path = if page_count == 1 {
      bucket_path.to_path_buf()
    } else {
      bucket_path.join(format!("page_{}", page_id))
    };

    Self { page_id, page_path }
  }

  /// 加载此 page 的所有数据
  pub async fn load(&self, fs: &FileSystem) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    // 1. 检查 page_path 是否存在
    // 2. 读取所有 pack 文件（hot.pack, cold_*.pack）
    // 3. 合并所有数据
    // 4. 返回
  }

  /// 准备保存数据（返回文件操作列表）
  pub async fn prepare_save(
    &self,
    fs: &FileSystem,
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    max_pack_size: usize,
  ) -> Result<PageSaveResult> {
    // 1. Hot/Cold 分离
    // 2. 生成 pack 文件内容
    // 3. 生成 index 文件内容
    // 4. 返回所有文件路径和内容
  }
}

pub struct PageSaveResult {
  pub files_to_add: Vec<(Utf8PathBuf, Vec<u8>)>,
  pub files_to_remove: Vec<Utf8PathBuf>,
}
```

### 7. `db/bucket.rs` - Bucket 操作

```rust
use crate::db::{page::Page, meta::BucketMeta, error::Result};
use crate::fs::FileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashMap as HashMap;
use std::sync::Arc;

pub struct Bucket {
  name: String,
  bucket_path: Utf8PathBuf,
  meta: BucketMeta,
}

impl Bucket {
  /// 打开或创建 bucket
  pub async fn open(
    fs: &FileSystem,
    root: &Utf8Path,
    name: &str,
    default_meta: BucketMeta,
  ) -> Result<Self> {
    // 1. bucket_path = root/name
    // 2. 读取 bucket_meta（如果存在）
    // 3. 如果不存在使用 default_meta
    // 4. 返回 Bucket
  }

  /// 加载所有数据
  pub async fn load(
    &self,
    fs: &FileSystem,
  ) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>> {
    // 1. 创建所有 Page
    // 2. 并行调用 page.load()
    // 3. 合并数据，包装成 Arc
    // 4. 返回
  }

  /// 准备保存（异步读取现有数据）
  pub async fn prepare_save(
    &self,
    fs: &FileSystem,
    changes: HashMap<Vec<u8>, Option<Vec<u8>>>,
  ) -> Result<BucketSaveResult> {
    // 1. 加载现有数据
    // 2. 应用增量修改
    // 3. 按 page 分配
    // 4. 对每个 page 调用 page.prepare_save()
    // 5. 收集所有文件操作
    // 6. 返回
  }

  /// Page 分配算法
  fn allocate_page(&self, key: &[u8]) -> usize {
    key.iter().map(|&b| b as usize).sum::<usize>() % self.meta.page_count
  }
}

pub struct BucketSaveResult {
  pub files_to_add: Vec<(Utf8PathBuf, Vec<u8>)>,
  pub files_to_remove: Vec<Utf8PathBuf>,
  pub meta_updated: bool,  // 是否需要更新 bucket_meta
}
```

### 8. `db/mod.rs` - DB 主入口

```rust
use tokio::sync::RwLock;
use rustc_hash::FxHashMap as HashMap;
use std::sync::Arc;

pub use self::error::{Error, Result};
pub use self::options::DBOptions;

mod error;
mod options;
mod meta;
mod bucket;
mod page;
mod pack;
mod index;
pub mod transaction;

pub struct DB {
  root: Utf8PathBuf,
  options: DBOptions,
  fs: Arc<dyn IntermediateFileSystem>,
  rw_lock: Arc<RwLock<()>>,
}

impl DB {
  pub fn new(
    root: Utf8PathBuf,
    options: DBOptions,
    fs: Arc<dyn IntermediateFileSystem>,
  ) -> Result<Self> {
    if options.page_count == 0 {
      return Err(Error::InvalidConfig("page_count cannot be 0".to_string()));
    }

    Ok(Self {
      root,
      options,
      fs,
      rw_lock: Arc::new(RwLock::new(())),
    })
  }

  /// 加载 bucket 数据
  pub async fn load(
    &self,
    scope: &'static str,
  ) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>> {
    let _guard = self.rw_lock.read().await;

    let bucket = Bucket::open(
      &FileSystem(self.fs.clone()),
      &self.root,
      scope,
      BucketMeta::new(self.options.page_count, self.options.max_pack_size),
    ).await?;

    bucket.load(&FileSystem(self.fs.clone())).await
  }

  /// 保存数据
  pub async fn save(
    &self,
    changes: HashMap<&'static str, HashMap<Vec<u8>, Option<Vec<u8>>>>,
  ) -> Result<()> {
    let mut tx = self.transaction();
    tx.begin().await?;

    for (scope, scope_changes) in changes {
      let bucket = Bucket::open(
        &FileSystem(self.fs.clone()),
        &self.root,
        scope,
        BucketMeta::new(self.options.page_count, self.options.max_pack_size),
      ).await?;

      let save_result = bucket.prepare_save(
        &FileSystem(self.fs.clone()),
        scope_changes,
      ).await?;

      // 添加文件到 transaction
      for (path, content) in save_result.files_to_add {
        tx.add_file(&path, &content).await?;
      }

      for path in save_result.files_to_remove {
        tx.remove_file(&path);
      }
    }

    tx.commit().await?;
    Ok(())
  }

  /// 创建 transaction
  fn transaction(&self) -> DBTransaction {
    DBTransaction {
      tx: transaction::Transaction::new(self.root.clone(), self.fs.clone()),
      rw_lock: self.rw_lock.clone(),
    }
  }
}

pub struct DBTransaction {
  tx: transaction::Transaction,
  rw_lock: Arc<RwLock<()>>,
}

impl DBTransaction {
  pub async fn begin(&mut self) -> crate::fs::FSResult<()> {
    self.tx.begin().await
  }

  pub async fn add_file(
    &mut self,
    path: &Utf8Path,
    content: &[u8],
  ) -> crate::fs::FSResult<()> {
    let _guard = self.rw_lock.read().await;
    self.tx.add_file(path, content).await
  }

  pub fn remove_file(&mut self, path: &Utf8Path) {
    self.tx.remove_file(path)
  }

  pub async fn commit(&mut self) -> crate::fs::FSResult<()> {
    let _guard = self.rw_lock.write().await;
    self.tx.commit().await
  }
}
```

## 设计决策

### 1. Bucket.prepare_save() 是异步的

- 需要读取现有 pack 文件
- 使用 `async fn prepare_save(&self, fs: &FileSystem, ...)`

### 2. Page 分配算法在 Bucket 中

- 作为 Bucket 的私有方法
- `fn allocate_page(&self, key: &[u8]) -> usize`

### 3. DB 内部错误类型

- 简单的 `enum Error`
- 不依赖外部 `crate::error::Error`
- 只在 DB 模块内部使用

## 当前任务

修改 `crates/rspack_storage/src/db/transaction/mod.rs`：

1. 修改 `begin()` 的恢复逻辑
   - 简化进程检测：进程存活直接 panic，不检查是否当前进程
   - 添加 `recover_commit()` 方法

2. 修改 `commit()`
   - 添加 state.lock 清理

3. 添加 `recover_commit()` 方法
   - 从 commit.lock 恢复文件移动操作
   - 忽略文件操作错误

## 下一步

等 transaction 修改完成并 review 后，再继续实现 Phase 2。
