# DB 使用流程

## 概述

本文档描述 DB 实现的完整使用流程，包括数据生命周期、操作和内部处理过程。

## 核心原则

1. **无缓存**：DB 不在内存中缓存数据，所有读取都从磁盘进行。
2. **增量保存**：Save 操作接收增量修改，而非完整数据。
3. **原子提交**：所有文件操作使用 Transaction 保证原子性。
4. **部分 scope 保存**：支持只保存修改过的 scope。

## 数据流

```
用户 -> Storage trait -> DB 实现
         set/remove       save(增量)
```

## 使用生命周期

### 1. 初始化

```rust
let db = DB::new(root_path, options, fs);
```

**发生的事情：**

- DB 被创建但不加载任何数据
- 此时没有文件系统读取操作
- 目录结构尚未创建

### 2. 加载所有数据（启动时调用一次）

```rust
// Storage 为每个 scope 调用 load
let snapshot_data = storage.load("snapshot").await?;
let module_graph_data = storage.load("module_graph").await?;
let meta_data = storage.load("meta").await?;
```

**DB 对每个 `load(scope)` 的处理：**

1. 检查 bucket 目录是否存在
   - 如果不存在：返回空的 `Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>`
2. 读取 bucket_meta 文件
   - 获取 `page_count` 和 `max_pack_size`
   - 如果 meta 丢失或无效：视为损坏，删除整个 bucket，返回 Err

3. 扫描所有 page 目录 (`page_0/`, `page_1/`, ...)
   - 对每个 page，读取所有 pack 文件 (hot.pack, cold\_\*.pack)
   - 解析 pack 文件并收集所有 key-value 对
4. 返回所有数据为 `Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>`

**关键点：**

- 频率：**非常低**，通常在启动时只调用一次
- 无缓存：数据不保留在 DB 内存中
- 并行加载：可以在 bucket 内并行加载所有 pack 文件
- 损坏处理：如果任何文件损坏，删除整个 bucket 并返回 Err

### 3. 运行时修改（频繁）

```rust
storage.set("module_graph", key, value);
storage.remove("snapshot", key);
```

**发生的事情：**

- Storage trait 实现在内存中跟踪这些修改
- DB **不参与**此阶段
- 无磁盘 I/O

### 4. 保存（在空闲时调用）

```rust
let receiver = storage.trigger_save()?;
receiver.await??;
```

**Storage 做的事情：**

- 收集所有修改为：`HashMap<scope, HashMap<Key, Option<Value>>>`
  - `Some(value)` = 创建或更新
  - `None` = 删除
- 将增量修改传递给 DB

**DB 做的事情：**

#### 步骤 1: 读取现有数据

对每个修改过的 scope：

- 从磁盘读取所有现有 pack 文件
- 构建当前状态：`HashMap<Key, Value>`

#### 步骤 2: 应用增量修改

```rust
for (key, value_opt) in incremental_changes {
  match value_opt {
    Some(value) => current_data.insert(key, value),  // 创建或更新
    None => current_data.remove(key),                // 删除
  }
}
```

#### 步骤 3: 分配到 Page

```rust
// Page 分配算法：字节和 % page_count
fn allocate_page(key: &[u8], page_count: usize) -> usize {
  key.iter().map(|&b| b as usize).sum::<usize>() % page_count
}

// 按 page 分组数据
let mut pages: HashMap<usize, Vec<(Key, Value)>> = HashMap::new();
for (key, value) in current_data {
  let page_id = allocate_page(&key, page_count);
  pages.entry(page_id).or_default().push((key, value));
}
```

#### 步骤 4: Hot/Cold 分离（每个 Page）

对每个 page：

1. **读取现有 pack 文件：**
   - hot.pack（如果存在）
   - cold\_\*.pack 文件（如果存在）

2. **与新数据合并：**
   - 所有新/更新的数据首先进入 hot.pack
   - 现有 hot 数据保留在 hot.pack
   - 现有 cold 数据保留在 cold pack（不变）

3. **检查 hot.pack 大小：**

   ```rust
   if hot_pack_size > max_pack_size * 2 {
     // 需要冻结部分数据到 cold

     // 策略：保留最新数据在 hot，冻结最旧的到 cold
     // 将 hot.pack 拆分为：
     // - 新的 hot.pack（大小 ≤ max_pack_size）
     // - 新的 cold_XXX.pack（大小 ≤ max_pack_size）

     let new_cold_name = format!("cold_{}.pack", generate_unique_id());
   }
   ```

4. **生成新的 pack 文件：**
   - 写入新的 hot.pack（或更新的 hot.pack）
   - 写入新的 cold_XXX.pack（如果冻结）
   - 保持现有 cold\_\*.pack 文件不变

5. **更新 index 文件：**

   ```
   hot.pack,<content_hash>,<file_hash>
   cold_abc.pack,<content_hash>,<file_hash>
   cold_xyz.pack,<content_hash>,<file_hash>
   ```

   - `content_hash`：此 pack 中所有 key hash 的按位或 (u64)
   - `file_hash`：文件内容的哈希，用于防篡改（字符串）

#### 步骤 5: 通过 Transaction 原子提交

```rust
let mut tx = Transaction::new(root, fs);

// 开始事务（创建 state.lock）
tx.begin().await?;

// 将所有新/修改的文件添加到临时目录
for each modified bucket {
  for each modified page {
    // 添加新的 pack 文件
    tx.add_file("bucket/page_X/hot.pack", content).await?;
    tx.add_file("bucket/page_X/cold_YYY.pack", content).await?;

    // 添加更新的 index
    tx.add_file("bucket/page_X/index", content).await?;

    // 标记旧 pack 文件待删除（如果有）
    tx.remove_file("bucket/page_X/old_file.pack");
  }

  // 如需要则更新 bucket_meta（罕见）
  tx.add_file("bucket/bucket_meta", content).await?;
}

// 如果 bucket 有变化则更新 db_meta
tx.add_file("db_meta", content).await?;

// 提交事务
// 1. 写入 commit.lock 记录所有操作
// 2. 从 .temp/ 移动文件到 root/
// 3. 删除旧文件
// 4. 移除 commit.lock
tx.commit().await?;
```

**Transaction 保证：**

- 所有文件首先写入 `.temp/`
- commit.lock 记录所有操作
- 原子的移动/删除操作
- 崩溃恢复（见 Transaction 文档）

### 5. 重置

```rust
storage.reset().await;
```

**发生的事情：**

- 删除整个 root 目录
- 清空所有内存状态（如果有）

## 文件操作总结

### Load（只读）

- 读取 bucket_meta
- 读取所有 page 中的所有 pack 文件
- 解析 pack 格式
- 返回数据

### Save（写入）

- 读取现有 pack 文件（仅针对修改的 page）
- 合并增量修改
- 应用 Hot/Cold 分离
- 通过 Transaction 写入新 pack 文件
- 通过 Transaction 更新 index 文件
- 原子提交

## Edge Cases

### Page Count 配置

**page_count = 0:**

- DB 初始化时返回错误
- 不支持

**page_count = 1:**

- 特殊优化：跳过 page 目录层级
- 文件结构变为：
  ```
  root/
  ├── db_meta
  ├── bucket1/
  │   ├── bucket_meta
  │   ├── index           # 没有 page_0/ 目录
  │   ├── hot.pack
  │   └── cold_*.pack
  ```

### 损坏处理

**bucket_meta 损坏：**

- 删除整个 bucket
- 向调用者返回 Err

**pack 文件损坏：**

- 删除整个 bucket（包含该 page）
- 向调用者返回 Err

**index 文件损坏：**

- 仍可直接读取 pack 文件
- 重建 index 或删除整个 bucket
- 向调用者返回 Err

**db_meta 丢失：**

- 视为空 DB
- 在首次 save 时创建新的 db_meta

### 空 Key 处理

**空 key（零长度 Vec<u8>）：**

- Page 分配：`0 % page_count = 0`
- 总是进入 page 0
- 有效且支持

### 并发操作

**多个 load() 调用：**

- 安全：只读操作
- 可以并行运行

**save() 期间的 load()：**

- 被 state.lock（Transaction）阻塞
- save() 在整个操作期间持有 state.lock

**多个 save() 调用：**

- 第二个 save() 被 state.lock 阻塞
- 必须等待第一个 save() 完成

## 性能特征

### Load 操作

- 时间：O(total_pack_files)
- I/O：读取所有 pack 文件
- 并行性：可以在每个 bucket 内并行读取 pack 文件

### Save 操作

- 时间：O(modified_data + existing_data_in_affected_pages)
- I/O：
  - 读取：仅受影响 page 中的现有 pack 文件
  - 写入：新 pack 文件 + index 文件
- 并行性：可以并行处理不同的 bucket

### 内存使用

- Load：O(total_data_size) - 所有数据返回给调用者
- Save：O(modified_scope_data) - 仅处理修改的 scope
- DB 本身：O(1) - 无缓存

## 锁机制详解

### 两层锁设计

DB 使用两层锁来保证并发安全：

#### 1. 内存锁（Mutex）

```rust
pub struct DB {
  tx_lock: Arc<Mutex<()>>,
}

pub async fn transaction(&self) -> Result<DBTransaction> {
  let guard = self.tx_lock.lock().await;
  // ...
  Ok(DBTransaction {
    tx,
    _guard: guard, // 持有锁直到 transaction drop
  })
}
```

**作用：**

- 保证同一进程内，同时只有一个 transaction 在执行
- 防止同进程内多个 save() 并发

#### 2. 文件锁（state.lock）

```rust
// Transaction::begin() 创建
state.lock 内容：
{
  "pid": 12345,
  "timestamp": "2024-02-11T10:00:00Z"
}

// Transaction::commit() 删除
```

**作用：**

- 保证跨进程互斥，防止多个程序同时操作同一 DB
- 支持崩溃恢复

### 竞态条件防护

**场景：程序 A 和程序 B 同时检查 state.lock**

```
时间线：
T1: 程序 A 检查 state.lock → 不存在
T2: 程序 B 检查 state.lock → 不存在
T3: 程序 A 创建 state.lock (PID=100)
T4: 程序 B 创建 state.lock (PID=200) ← 覆盖了 A 的锁！
T5: 程序 A 执行 commit()
    读取 state.lock → PID=200
    检查：200 != 100 → panic! ✓
```

**防护措施：**

- commit() 时必须验证 state.lock 的 PID 是否为当前进程
- 如果不匹配，说明发生了竞态，panic 终止操作

```rust
pub async fn commit(&mut self) -> FSResult<()> {
  let state_lock = self.lock_helper.state_lock().await?
    .expect("state.lock should exist");

  // 关键检查：防止竞态条件
  if !state_lock.is_current() {
    panic!(
      "state.lock mismatch: expected current process (pid={}), found pid={}",
      std::process::id(),
      state_lock.pid
    );
  }

  // 写入 commit.lock
  // ...
}
```

### state.lock 生命周期

```
Transaction::begin()
  ↓
创建 state.lock（记录当前进程 PID）
  ↓
Transaction::add_file()
Transaction::remove_file()
  ↓
Transaction::commit()
  ↓
写入 commit.lock
执行文件移动/删除
删除 commit.lock
删除 state.lock ← 事务结束清理
清空 temp 目录
```

**关键点：**

- state.lock 仅在 Transaction 内创建和删除
- 正常流程下，commit 成功后 state.lock 被删除
- 异常退出时，state.lock 保留用于恢复检测

### 恢复场景

| 场景                  | state.lock | 进程状态 | commit.lock | 处理                                |
| --------------------- | ---------- | -------- | ----------- | ----------------------------------- |
| 正常启动              | 不存在     | -        | -           | 清理 temp，创建 state.lock          |
| 异常退出（begin 后）  | 存在       | 已死     | 不存在      | 清理 temp，创建新 state.lock        |
| 异常退出（commit 中） | 存在       | 已死     | 存在        | 恢复 merge，清理，创建新 state.lock |
| 进程冲突              | 存在       | **存活** | -           | **panic（TODO: 改为 Err）**         |

### 恢复逻辑详解

```rust
pub async fn begin(&mut self) -> FSResult<()> {
  if let Ok(Some(state_lock)) = self.lock_helper.state_lock().await {
    if state_lock.is_running() {
      // 进程还活着，直接 panic
      // 不需要检查是否当前进程，因为：
      // 1. 正常情况下 state.lock 在 commit 时删除
      // 2. 如果存在，说明有其他操作正在进行
      panic!("Transaction already in progress by process {}", state_lock.pid);
    } else {
      // 进程已死，检查 commit.lock
      if let Ok(Some(commit_lock)) = self.lock_helper.commit_lock().await {
        // 恢复 merge 操作
        self.recover_commit(commit_lock).await?;
      }
      // 清理 temp
    }
  }

  // 创建新的 state.lock
  let state_lock = StateLock::default();
  self.lock_helper.update_state_lock(Some(&state_lock)).await?;

  Ok(())
}

async fn recover_commit(&self, commit_lock: CommitLock) -> FSResult<()> {
  // 继续移动文件（拷贝语义，可重复执行）
  for file in &commit_lock.files_to_add {
    let temp_path = self.temp_root.join(file);
    let root_path = self.root.join(file);

    // 不检查文件是否存在，直接尝试移动
    // - 成功：文件从 temp 移动到 root
    // - 失败：可能已经移动过，或文件丢失
    // 文件丢失会在后续 load() 时检测
    let _ = self.move_file_internal(&temp_path, &root_path).await;
  }

  // 删除旧文件（忽略错误）
  for file in &commit_lock.files_to_remove {
    let _ = self.remove_file_internal(&self.root.join(file)).await;
  }

  // 清理 commit.lock
  self.lock_helper.update_commit_lock(None).await?;

  Ok(())
}
```

**恢复原则：**

- 移动操作是拷贝语义，可以重复执行
- 忽略文件移动/删除的错误
- 文件完整性检查推迟到 load() 时进行
- 如果文件丢失，load() 会删除整个 bucket 并返回 Err

## TODO

- [ ] 将进程冲突时的 panic 改为返回 Err，提供更友好的错误处理

## 总结

1. **加载一次**：启动时从磁盘读取所有数据
2. **在内存中修改**：Storage 跟踪修改，DB 不参与
3. **增量保存**：DB 读取现有数据，应用修改，原子写入
4. **无缓存**：所有操作都从磁盘读取
5. **原子提交**：Transaction 确保一致性
6. **部分保存**：仅保存修改的 scope
7. **两层锁**：内存锁（同进程）+ 文件锁（跨进程）
8. **竞态防护**：commit 时验证 state.lock，防止多进程竞争
