# DB Transaction 和锁机制实现规范

## 概述

本文档详细描述 Transaction 的实现规范，包括锁机制、恢复逻辑、API 设计等。

## API 设计

### DB 结构

```rust
use tokio::sync::RwLock;

pub struct DB {
  root: Utf8PathBuf,
  options: DBOptions,
  fs: Arc<dyn IntermediateFileSystem>,
  /// 读写锁，保护文件访问
  /// - 读锁：load() 和 transaction 写 temp
  /// - 写锁：transaction commit（移动文件）
  rw_lock: Arc<RwLock<()>>,
}

impl DB {
  pub fn new(
    root: Utf8PathBuf,
    options: DBOptions,
    fs: Arc<dyn IntermediateFileSystem>,
  ) -> Result<Self> {
    if options.page_count == 0 {
      return Err(Error::from_reason(
        None,
        None,
        "page_count cannot be 0".to_string()
      ));
    }

    Ok(Self {
      root,
      options,
      fs,
      rw_lock: Arc::new(RwLock::new(())),
    })
  }

  /// 创建一个新的 transaction
  pub async fn transaction(&self) -> Result<DBTransaction<'_>> {
    let tx = Transaction::new(self.root.clone(), self.fs.clone());

    Ok(DBTransaction {
      tx,
      rw_lock: self.rw_lock.clone(),
    })
  }

  /// 加载 bucket 数据
  pub async fn load(&self, scope: &str) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>> {
    // 获取读锁（整个加载过程持有）
    let _guard = self.rw_lock.read().await;

    // 读取文件
    let data = self.load_bucket(scope).await?;

    Ok(data)
    // 释放读锁
  }
}

/// Transaction 包装器，持有读写锁引用
pub struct DBTransaction<'a> {
  tx: Transaction,
  rw_lock: Arc<RwLock<()>>,
}

impl<'a> DBTransaction<'a> {
  pub async fn begin(&mut self) -> FSResult<()> {
    // begin 不需要锁
    self.tx.begin().await
  }

  pub async fn add_file(&mut self, path: impl AsRef<Utf8Path>, content: &[u8]) -> FSResult<()> {
    // 获取读锁（写 temp 目录，不影响 root）
    let _guard = self.rw_lock.read().await;

    self.tx.add_file(path, content).await

    // 释放读锁
  }

  pub fn remove_file(&mut self, path: impl AsRef<Utf8Path>) {
    // 只是标记，不需要锁
    self.tx.remove_file(path)
  }

  pub async fn commit(&mut self) -> FSResult<()> {
    // 获取写锁（移动文件，需要互斥所有操作）
    let _guard = self.rw_lock.write().await;

    self.tx.commit().await

    // 释放写锁
  }
}
```

### Transaction 修改

修改 `crates/rspack_storage/src/db/transaction/mod.rs`：

```rust
impl Transaction {
  // new() 保持不变，不调用 begin
  pub fn new(root: Utf8PathBuf, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    // 现有实现不变
  }

  /// 开始事务，处理恢复逻辑
  pub async fn begin(&mut self) -> FSResult<()> {
    // 检查 state.lock
    let should_cleanup = if let Ok(Some(state_lock)) = self.lock_helper.state_lock().await {
      if state_lock.is_running() {
        // 简化逻辑：进程存活直接 panic，不检查是否当前进程
        // 原因：正常情况下 state.lock 在 commit 时删除
        //      如果存在且进程存活，说明有冲突
        panic!(
          "Transaction already in progress by process {} at {}",
          state_lock.pid, self.root
        );
      } else {
        // 进程已死，检查 commit.lock
        if let Ok(Some(commit_lock)) = self.lock_helper.commit_lock().await {
          // 恢复 merge 操作
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
      let _ = self.remove_dir_internal(&self.temp_root).await;
    }

    // 创建新的 state.lock
    let state_lock = StateLock::default();
    self.lock_helper.update_state_lock(Some(&state_lock)).await?;

    // 清空文件跟踪
    self.added_files.clear();
    self.removed_files.clear();

    // 确保 temp 目录存在
    let _ = self.remove_dir_internal(&self.temp_root).await;
    self.ensure_dir_internal(&self.temp_root).await?;

    Ok(())
  }

  /// 恢复 commit 操作（从 commit.lock）
  async fn recover_commit(&self, commit_lock: CommitLock) -> FSResult<()> {
    // 继续移动文件（拷贝语义，可重复执行）
    for file in &commit_lock.files_to_add {
      let temp_path = self.temp_root.join(file);
      let root_path = self.root.join(file);

      // 不检查文件是否存在，直接尝试移动
      // 忽略错误：
      // - 文件已移动：temp 中不存在，move 失败，忽略
      // - 文件丢失：两边都不存在，move 失败，忽略
      //   （文件丢失会在后续 load() 时检测并处理）
      if let Some(parent) = root_path.parent() {
        let _ = self.ensure_dir_internal(parent).await;
      }
      let _ = self.move_file_internal(&temp_path, &root_path).await;
    }

    // 删除旧文件（忽略错误）
    for file in &commit_lock.files_to_remove {
      let path = self.root.join(file);
      let _ = self.remove_file_internal(&path).await;
    }

    // 清理 commit.lock
    self.lock_helper.update_commit_lock(None).await?;

    Ok(())
  }

  pub async fn commit(&mut self) -> FSResult<()> {
    // 读取并验证 state.lock
    let state_lock = self
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

    // 写 commit.lock
    let commit_lock = CommitLock::new(
      self.added_files.iter().cloned().collect(),
      self.removed_files.iter().cloned().collect(),
    );
    self.lock_helper.update_commit_lock(Some(&commit_lock)).await?;

    // 执行 commit
    self.execute_commit().await?;

    // 清理 commit.lock
    self.lock_helper.update_commit_lock(None).await?;

    // 清理 state.lock
    self.lock_helper.update_state_lock(None).await?;

    // 清空文件跟踪
    self.added_files.clear();
    self.removed_files.clear();

    Ok(())
  }
}
```

## 使用示例

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

// 3. 保存数据
async fn save(db: &DB, changes: HashMap<&str, HashMap<Vec<u8>, Option<Vec<u8>>>>) -> Result<()> {
  // 创建 transaction
  let mut tx = db.transaction().await?;

  // 开始事务（自动处理恢复）
  tx.begin().await?;

  // 添加文件（获取读锁，可与 load 并发）
  for (bucket, page_data) in process_changes(changes) {
    tx.add_file(
      format!("{}/page_{}/hot.pack", bucket, page_id),
      pack_content,
    ).await?; // 每次调用获取/释放读锁

    tx.add_file(
      format!("{}/page_{}/index", bucket, page_id),
      index_content,
    ).await?;
  }

  // 标记删除（无锁）
  tx.remove_file("bucket/page_0/old.pack");

  // 提交（获取写锁，互斥所有操作）
  tx.commit().await?;

  Ok(())
}
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

| 操作               | 锁类型 | 持有时长     | 互斥关系       |
| ------------------ | ------ | ------------ | -------------- |
| `load()`           | 读锁   | 整个加载过程 | 与 commit 互斥 |
| `tx.begin()`       | 无锁   | -            | -              |
| `tx.add_file()`    | 读锁   | 单次调用     | 与 commit 互斥 |
| `tx.remove_file()` | 无锁   | -            | -              |
| `tx.commit()`      | 写锁   | 整个提交过程 | 与所有操作互斥 |

**并发场景：**

1. **多个 load 并发**：

```rust
// 场景：同时加载多个 scope
T1: load("snapshot") 获取读锁 ✓
T2: load("module_graph") 获取读锁 ✓（读锁共享）
T3: 两个 load 并发执行 ✓
```

2. **load + add_file 并发**：

```rust
T1: load() 获取读锁 ✓
T2: tx.add_file() 获取读锁 ✓（读锁共享）
T3: load() 读取 root/，tx 写入 .temp/ ✓（不冲突）
```

3. **load + commit 互斥**：

```rust
T1: load() 获取读锁 ✓
T2: tx.commit() 尝试获取写锁 ⏸️（等待）
T3: load() 完成，释放读锁
T4: tx.commit() 获取写锁 ✓，移动文件
```

4. **多个 add_file 并发**：

```rust
T1: tx1.add_file() 获取读锁 ✓
T2: tx2.add_file() 获取读锁 ✓（读锁共享）
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
