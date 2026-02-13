# DB Transaction 和锁机制实现规范

## 概述

本文档详细描述 Transaction 的实现规范，包括锁机制、恢复逻辑、两阶段锁定策略等。

## 目录结构

```
root/
├── .temp/                  # 临时目录（事务使用）
│   ├── state.lock          # 进程锁（在 .temp 中）
│   ├── commit.lock         # 提交锁（在 .temp 中）
│   └── bucket1/            # 临时数据（与正式结构一致）
│       ├── 0.hot.pack
│       ├── 0.hot.index
│       └── bucket_meta.txt
├── bucket1/                # 正式数据目录
│   ├── bucket_meta.txt
│   ├── 0.hot.pack
│   └── ...
└── bucket2/
    └── ...
```

## 两阶段锁定策略

### Phase 1: 读锁阶段（数据准备）

- **目的**: 准备所有需要写入的数据
- **锁**: 持有 RwLock 的读锁
- **并发**: 允许多个读操作并发
- **操作**:
  1. 加载现有数据
  2. 合并变更
  3. 计算 hot/cold 分裂
  4. 序列化所有数据（生成 pack/index 内容）
  5. 返回待添加/删除文件列表

### Phase 2: 写锁阶段（事务提交）

- **目的**: 将准备好的数据原子性写入
- **锁**: 持有 RwLock 的写锁
- **并发**: 独占访问，阻塞所有其他操作
- **操作**:
  1. 获取写锁
  2. Transaction.begin() - 创建 .temp/state.lock
  3. 写入文件到 .temp/bucket/
  4. Transaction.commit() - 移动文件到正式目录
  5. 删除锁文件
  6. 释放写锁

### 设计优势

1. **最小化写锁持有时间**: 耗时的计算和序列化在读锁下完成
2. **提高并发性**: 数据准备期间允许并发读取
3. **简化事务逻辑**: Transaction 只负责原子文件操作
4. **清晰的职责分离**:
   - Bucket: 负责数据逻辑（分裂、合并）
   - Transaction: 负责文件原子性
   - DB: 负责锁协调

## API 设计

### DB 结构

```rust
use tokio::sync::RwLock;

pub struct DB {
  root: Utf8PathBuf,
  options: DBOptions,
  fs: Arc<dyn IntermediateFileSystem>,
  /// 读写锁，保护文件访问
  /// - 读锁：load() 和数据准备阶段
  /// - 写锁：transaction commit（移动文件）
  rw_lock: Arc<RwLock<()>>,
}

impl DB {
  pub fn new(
    root: Utf8PathBuf,
    options: DBOptions,
    fs: Arc<dyn IntermediateFileSystem>,
  ) -> DBResult<Self> {
    if options.page_count == 0 {
      return Err(DBError::InvalidFormat("page_count cannot be 0".to_string()));
    }

    Ok(Self {
      root,
      options,
      fs,
      rw_lock: Default::default(),
    })
  }

  /// 加载 bucket 数据
  pub async fn load(&self, scope: &str) -> DBResult<HashMap<Vec<u8>, Vec<u8>>> {
    // 获取读锁（整个加载过程持有）
    let _guard = self.rw_lock.read().await;

    let bucket = Bucket::new(
      scope.to_string(),
      &self.root,
      self.fs.clone(),
      self.options.clone(),
    );

    bucket.load().await
  }

  /// 保存数据（两阶段锁定）
  pub async fn save(
    &self,
    changes: HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>,
  ) -> DBResult<()> {
    // 见下文"DB::save 实际实现"
  }
}
```

### Transaction 实现

当前实现位于 `crates/rspack_storage/src/db/transaction/mod.rs`：

```rust
impl Transaction {
  /// 创建新事务
  ///
  /// temp_root 设置为 root/.temp
  /// lock_helper 使用 temp_root（即 .temp 目录）
  pub fn new(root: Utf8PathBuf, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    let temp_root = root.join(".temp");
    let lock_helper = LockHelper::new(temp_root.clone(), fs.clone());
    // ...
  }

  /// 开始事务，处理恢复逻辑
  pub async fn begin(&mut self) -> FSResult<()> {
    // 检查 .temp/state.lock
    let should_cleanup = if let Ok(Some(state_lock)) = self.lock_helper.state_lock().await {
      if state_lock.is_running() {
        // 进程存活直接 panic
        panic!(
          "Transaction already in progress by process {} at {}",
          state_lock.pid, self.root
        );
      } else {
        // 进程已死，检查 .temp/commit.lock
        if let Ok(Some(commit_lock)) = self.lock_helper.commit_lock().await {
          // 恢复提交操作
          self.recover_commit(commit_lock).await?;
          true
        } else {
          true
        }
      }
    } else {
      true
    };

    if should_cleanup {
      let _ = self.fs.remove_dir(&self.temp_root).await;
    }

    // 创建新的 .temp/state.lock
    let state_lock = StateLock::default();
    self.lock_helper.update_state_lock(Some(&state_lock)).await?;

    // 清空文件跟踪并确保 .temp 目录存在
    self.added_files.clear();
    self.removed_files.clear();
    self.fs.ensure_dir(&self.temp_root).await?;

    Ok(())
  }

  /// 恢复 commit 操作（从 .temp/commit.lock）
  async fn recover_commit(&self, commit_lock: CommitLock) -> FSResult<()> {
    // 继续移动文件（从 .temp/bucket/ 到 bucket/）
    for file in &commit_lock.files_to_add {
      let temp_path = self.temp_root.join(file);  // .temp/bucket1/0.hot.pack
      let root_path = self.root.join(file);        // bucket1/0.hot.pack
      let _ = self.fs.move_file(&temp_path, &root_path).await;
    }

    // 删除旧文件
    for file in &commit_lock.files_to_remove {
      let path = self.root.join(file);
      let _ = self.fs.remove_file(&path).await;
    }

    // 清理 .temp/commit.lock
    self.lock_helper.update_commit_lock(None).await?;

    Ok(())
  }

  pub async fn commit(&mut self) -> FSResult<()> {
    // 读取并验证 .temp/state.lock
      .lock_helper
      .state_lock()
      .await?
      .expect("state.lock should exist - did you call begin()?");

    // 关键检查：防止多进程竞态条件
    // 场景：
    //   T1: 进程 A 检查 state.lock → 不存在
    //   T2: 进程 B 检查 state.lock → 不存在
    //   T3: 进程 A 创建 state.lock (PID=100)
    //   T4: 进程 B 创建 state.lock (PID=200) ← 覆盖！
    //   T5: 进程 A commit() → 检测到 PID 不匹配 → panic
    if !state_lock.is_current() {
      panic!(
        "state.lock mismatch: expected current process (pid={}), found pid={}",
        std::process::id(),
        state_lock.pid
      );
    }

    // 写 .temp/commit.lock
    let commit_lock = CommitLock::new(
      self.added_files.iter().cloned().collect(),
      self.removed_files.iter().cloned().collect(),
    );
    self.lock_helper.update_commit_lock(Some(&commit_lock)).await?;

    // 执行 commit（移动文件从 .temp 到正式目录）
    self.execute_commit().await?;

    // 清理 .temp/commit.lock
    self.lock_helper.update_commit_lock(None).await?;

    // 清理 .temp/state.lock
    self.lock_helper.update_state_lock(None).await?;

    // 清空文件跟踪
    self.added_files.clear();
    self.removed_files.clear();

    Ok(())
  }

  async fn execute_commit(&self) -> FSResult<()> {
    // 移动新文件从 .temp 到 root
    for path in &self.added_files {
      let temp_path = self.temp_root.join(path);  // .temp/bucket1/0.hot.pack
      let root_path = self.root.join(path);        // bucket1/0.hot.pack
      self.fs.move_file(&temp_path, &root_path).await?;
    }

    // 删除旧文件
    for path in &self.removed_files {
      let root_path = self.root.join(path);
      let _ = self.fs.remove_file(&root_path).await;
    }

    // 清理 .temp 目录
    let _ = self.fs.remove_dir(&self.temp_root).await;

    Ok(())
  }
}
```

## DB::save 实际实现

当前 DB::save 已实现两阶段锁定策略：

```rust
impl DB {
  /// Save changes to the DB with two-phase locking
  pub async fn save(
    &self,
    changes: HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>,
  ) -> DBResult<()> {
    let mut all_files_to_add = Vec::new();
    let mut all_files_to_remove = Vec::new();

    // Phase 1: Read lock - prepare data to .temp directory
    {
      let _guard = self.rw_lock.read().await;

      for (scope, scope_changes) in changes {
        let mut bucket = Bucket::new(scope, &self.root, self.fs.clone(), self.options.clone());

        let old_data = bucket.load().await.unwrap_or_default();
        let mut new_data = old_data;

        for (key, value) in scope_changes {
          match value {
            Some(v) => new_data.insert(key, v),
            None => new_data.remove(&key),
          };
        }

        // prepare_save 返回相对路径（如 "bucket1/0.hot.pack"）
        let save_result = bucket.prepare_save(new_data).await?;
        all_files_to_add.extend(save_result.files_to_add);
        all_files_to_remove.extend(save_result.files_to_remove);
      }
    } // 释放读锁

    // Phase 2: Write lock - commit from .temp to root
    {
      let _guard = self.rw_lock.write().await;

      let mut tx = Transaction::new(self.root.clone(), self.fs.clone());
      tx.begin().await?;  // 创建 .temp/state.lock

      for (path, content) in all_files_to_add {
        // path 是相对路径，如 "bucket1/0.hot.pack"
        // 写入到 .temp/bucket1/0.hot.pack
        tx.add_file(&path, &content).await?;
      }

      for path in all_files_to_remove {
        tx.remove_file(&path);
      }

      tx.commit().await?;  // 移动文件，删除锁
    } // 释放写锁

    Ok(())
  }
}
```

### Bucket::prepare_save 实现

```rust
impl Bucket {
  pub async fn prepare_save(&mut self, data: HashMap<Vec<u8>, Vec<u8>>) -> DBResult<SaveResult> {
    let mut files_to_add = Vec::new();
    let mut files_to_remove = Vec::new();

    // ... 处理 hot/cold 分裂逻辑 ...

    // 返回相对路径（相对于 DB root）
    files_to_add.push((
      self.to_relative_path(&page.pack_path)?,  // "bucket1/0.hot.pack"
      pack_buf
    ));

    Ok(SaveResult {
      files_to_add,
      files_to_remove,
    })
  }

  /// Convert absolute path to relative path (from DB root)
  fn to_relative_path(&self, abs_path: &Utf8Path) -> DBResult<Utf8PathBuf> {
    abs_path
      .strip_prefix(&self.root)
      .map(|p| p.to_path_buf())
      .map_err(|_| DBError::InvalidFormat(format!("...")))
  }
}
```

## 使用示例

### 基本使用

```rust
// 1. 创建 DB
let db = DB::new(root, options, fs)?;

// 2. 并发 load（支持）
let handle1 = tokio::spawn({
  let db = db.clone();
  async move {
    db.load("snapshot").await
  }
});

let handle2 = tokio::spawn({
  let db = db.clone();
  async move {
    db.load("module_graph").await
  }
});

// 两个 load 可以并发执行（共享读锁）
let (data1, data2) = tokio::join!(handle1, handle2);

// 3. 保存数据（两阶段锁定）
let mut changes = HashMap::new();
changes.insert(
  "bucket1".to_string(),
  HashMap::from([
    (b"key1".to_vec(), Some(b"value1".to_vec())),
    (b"key2".to_vec(), None),  // 删除 key2
  ])
);

// save() 内部自动处理两阶段锁定和事务
db.save(changes).await?;
```

### 并发场景

```rust
// 场景1: 多个读操作可以并发
tokio::join!(
  db.load("bucket1"),
  db.load("bucket2"),
  db.load("bucket3"),
);

// 场景2: 读操作可以与 save 的准备阶段并发
// Thread 1: save() Phase 1（持有读锁，准备数据）
// Thread 2: load()（持有读锁，并发读取）
// ✓ 两者可以并发执行

// 场景3: save 的提交阶段独占
// Thread 1: save() Phase 2（持有写锁，移动文件）
// Thread 2: load()（等待读锁）
// ✗ 必须等待 Phase 2 完成
```

## 锁机制详解

### 读写锁设计（RwLock）

**为什么用 RwLock 而不是 Mutex？**

**问题场景：**

```
T1: load() 正在读取 root/bucket/page_0/hot.pack
T2: save() commit，移动新的 hot.pack 覆盖旧文件
T3: load() 继续读取 → 文件已被替换 → 数据不一致
```

**解决方案：读写锁**

- **读锁**：允许并发，用于 load() 和 add_file()（写 temp）
- **写锁**：独占互斥，用于 commit()（移动文件）

#### 读写锁（RwLock<()>）

**位置：** `DB.rw_lock`

**作用：**

- 保护文件访问，防止读写冲突
- 支持并发 load
- 支持并发写 temp（不影响 root）
- commit 时独占（保证文件替换的原子性）

**锁粒度：**

| 操作                     | 锁类型 | 持有时长         | 互斥关系       |
| ------------------------ | ------ | ---------------- | -------------- |
| `load()`                 | 读锁   | 整个加载过程     | 与 commit 互斥 |
| `save()` Phase 1（准备） | 读锁   | 数据准备和序列化 | 与 commit 互斥 |
| `save()` Phase 2（提交） | 写锁   | 文件移动和删除   | 与所有操作互斥 |
| `tx.begin()`             | 无锁   | -                | -              |
| `tx.add_file()`          | 无锁   | -                | -              |
| `tx.remove_file()`       | 无锁   | -                | -              |
| `tx.commit()`            | 写锁   | 验证锁并移动文件 | 与所有操作互斥 |

**并发场景：**

1. **多个 load 并发**：

```rust
// 场景：同时加载多个 scope
T1: load("snapshot") 获取读锁 ✓
T2: load("module_graph") 获取读锁 ✓（读锁共享）
T3: 两个 load 并发执行 ✓
```

2. **load + save Phase 1 并发**：

```rust
T1: load() 获取读锁 ✓
T2: save() Phase 1 获取读锁 ✓（读锁共享）
T3: load() 读取数据，save() 准备数据 ✓（并发）
```

3. **load + save Phase 2 互斥**：

```rust
T1: load() 获取读锁 ✓
T2: save() Phase 2 尝试获取写锁 ⏸️（等待）
T3: load() 完成，释放读锁
T4: save() Phase 2 获取写锁 ✓，移动文件
```

4. **多个 save Phase 1 并发**：

```rust
T1: save1() Phase 1 获取读锁 ✓
T2: save2() Phase 1 获取读锁 ✓（读锁共享）
T3: 两个 tx 并发写 .temp/ ✓
```

5. **commit 独占**：

```rust
T1: tx1.commit() 获取写锁 ✓
T2: tx2.commit() 尝试获取写锁 ⏸️（等待）
T3: load() 尝试获取读锁 ⏸️（等待）
T4: tx1.commit() 完成，释放写锁
T5: tx2.commit() 或 load() 获取锁 ✓
```

#### 文件锁（state.lock）

**位置：** `root/state.lock`

**格式：**

```json
{
  "pid": 12345,
  "timestamp": "2024-02-11T10:00:00Z"
}
```

**作用：**

- 保证跨进程互斥
- 支持崩溃恢复检测

**生命周期：**

- `Transaction::begin()` 创建
- `Transaction::commit()` 删除（成功提交后）
- 异常退出时保留（用于恢复检测）

**并发场景：**

```
进程 A: begin() → 创建 state.lock (PID=100)
进程 B: begin() → 检测到 state.lock (PID=100, 进程存活) → panic
```

### 两层锁协作

**层级：**

1. **RwLock（进程内）**：保护文件访问，防止读写冲突
2. **state.lock（跨进程）**：防止多进程同时操作

**时序：**

```
T1: tx.begin()
    ↓ 检查 state.lock（文件锁）
    ↓ 创建 state.lock

T2: tx.add_file()
    ↓ 获取 RwLock 读锁（内存锁）
    ↓ 写入 .temp/
    ↓ 释放 RwLock 读锁

T3: tx.commit()
    ↓ 获取 RwLock 写锁（内存锁）
    ↓ 验证 state.lock（文件锁）
    ↓ 写入 commit.lock
    ↓ 移动文件
    ↓ 删除 commit.lock
    ↓ 删除 state.lock（文件锁）
    ↓ 释放 RwLock 写锁（内存锁）
```

#### 文件锁（state.lock）

**位置：** `root/state.lock`

**格式：**

```json
{
  "pid": 12345,
  "timestamp": "2024-02-11T10:00:00Z"
}
```

**作用：**

- 保证跨进程互斥
- 支持崩溃恢复检测

**生命周期：**

- `Transaction::begin()` 创建
- `Transaction::commit()` 删除（成功提交后）
- 异常退出时保留（用于恢复检测）

**并发场景：**

```
进程 A: begin() → 创建 state.lock (PID=100)
进程 B: begin() → 检测到 state.lock (PID=100, 进程存活) → panic
```

### 竞态条件防护

**问题场景：**

```
T1: 进程 A 检查 state.lock → 不存在 ✓
T2: 进程 B 检查 state.lock → 不存在 ✓
T3: 进程 A 创建 state.lock (PID=100)
T4: 进程 B 创建 state.lock (PID=200) ← 覆盖了 A 的锁！
T5: 进程 A 和 B 同时操作文件 → 数据损坏
```

**解决方案：commit 时验证 PID**

```rust
// Transaction::commit()
let state_lock = self.lock_helper.state_lock().await?;

if !state_lock.is_current() {
  // 检测到 PID 不匹配，说明发生了竞态
  panic!("state.lock was overwritten by another process");
}

// 写入 commit.lock，继续操作
```

**时间线（加入验证后）：**

```
T1: 进程 A 检查 state.lock → 不存在 ✓
T2: 进程 B 检查 state.lock → 不存在 ✓
T3: 进程 A 创建 state.lock (PID=100)
T4: 进程 B 创建 state.lock (PID=200) ← 覆盖
T5: 进程 A commit() → 读取 state.lock (PID=200) → 验证失败 → panic ✓
T6: 进程 B commit() → 读取 state.lock (PID=200) → 验证成功 → 继续 ✓
```

**结果：**

- 进程 A 被 panic 终止，不会继续操作
- 进程 B 独占执行，数据安全
- 虽然不理想（A 失败了），但保证了数据一致性

## 恢复场景

### 场景 1: 正常启动

**状态：**

- state.lock: 不存在
- commit.lock: 不存在

**处理：**

```rust
begin() {
  // state.lock 不存在
  should_cleanup = true;

  // 清理 temp 目录
  remove_dir_all(.temp/);

  // 创建新的 state.lock
  create_state_lock(current_pid);

  // 创建 temp 目录
  create_dir_all(.temp/);
}
```

### 场景 2: 异常退出（begin 后，commit 前）

**状态：**

- state.lock: 存在（PID=12345，进程已死）
- commit.lock: 不存在
- .temp/: 可能有部分文件

**处理：**

```rust
begin() {
  // state.lock 存在，进程已死
  // commit.lock 不存在
  should_cleanup = true;

  // 清理 temp 目录（丢弃未提交的更改）
  remove_dir_all(.temp/);

  // 创建新的 state.lock
  create_state_lock(current_pid);

  // 创建 temp 目录
  create_dir_all(.temp/);
}
```

### 场景 3: 异常退出（commit 中）

**状态：**

- state.lock: 存在（PID=12345，进程已死）
- commit.lock: 存在
  ```
  files_to_add: [bucket/page_0/hot.pack, bucket/page_0/index]
  files_to_remove: [bucket/page_0/old.pack]
  ```
- .temp/: 部分文件可能已移动

**处理：**

```rust
begin() {
  // state.lock 存在，进程已死
  // commit.lock 存在

  // 恢复 merge 操作
  recover_commit(commit_lock) {
    // 尝试移动所有文件（忽略错误）
    for file in files_to_add {
      move(.temp/file, root/file); // 可能成功，可能失败
    }

    // 尝试删除旧文件（忽略错误）
    for file in files_to_remove {
      remove(root/file);
    }

    // 删除 commit.lock
    remove(commit.lock);
  }

  should_cleanup = true;

  // 清理 temp 目录
  remove_dir_all(.temp/);

  // 创建新的 state.lock
  create_state_lock(current_pid);

  // 创建 temp 目录
  create_dir_all(.temp/);
}
```

**文件丢失处理：**

- 恢复时不检查文件完整性
- 如果文件丢失，后续 `load()` 会检测到
- `load()` 会删除整个损坏的 bucket 并返回 Err

### 场景 4: 进程冲突

**状态：**

- state.lock: 存在（PID=12345，**进程存活**）

**处理：**

```rust
begin() {
  // state.lock 存在，进程还活着
  panic!("Transaction already in progress by process 12345");
}
```

**TODO：**

- 未来改为返回 Err 而不是 panic
- 提供更友好的错误处理

## 错误处理

### panic 场景

1. **进程冲突**：`begin()` 检测到 state.lock 对应进程存活
2. **PID 不匹配**：`commit()` 检测到 state.lock PID 被篡改
3. **state.lock 缺失**：`commit()` 时 state.lock 不存在（逻辑错误）

### Err 场景

1. **文件系统错误**：读写文件失败
2. **文件损坏**：`load()` 时检测到 pack 文件损坏
3. **配置错误**：`DB::new()` 时 `page_count = 0`

## 性能考虑

### 内存锁开销

- 单进程内串行执行 save()
- 对于 rspack 场景合理（save 频率低）
- 未来可优化为 bucket 级别的细粒度锁

### 文件锁开销

- 仅在 begin() 和 commit() 时读写 state.lock
- 对性能影响极小

### 恢复开销

- 仅在检测到异常退出时执行
- 正常启动只需检查 state.lock 是否存在

## TODO

- [ ] 将进程冲突时的 panic 改为返回 Err
- [ ] 考虑 bucket 级别的细粒度锁（未来优化）
- [ ] 添加 metrics 跟踪锁等待时间
