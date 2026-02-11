# DB Hot/Cold 分离和 Index 机制

## Hot/Cold 分离策略

### 基本规则

```rust
// 检查是否需要冻结
if hot_pack_size > max_pack_size * 2 {
  // 需要冻结部分数据到 cold
}
```

### 边界情况

| 场景               | hot_pack_size          | 处理                |
| ------------------ | ---------------------- | ------------------- |
| 正常               | `<= max_pack_size * 2` | 保留所有数据在 hot  |
| 刚好临界           | `== max_pack_size * 2` | 保留，不冻结        |
| 超过临界           | `> max_pack_size * 2`  | 冻结                |
| 所有数据都是新数据 | 任意                   | 正常，cold 可以为空 |
| cold 为空          | -                      | 正常场景            |

### 冻结算法：按条目输出

**目标：**

- cold pack 达到 `0.8 * max_pack_size` 时输出
- 避免产生过小的 cold pack
- 支持超大 KV（单个 KV > max_pack_size）

```rust
fn freeze_hot_to_cold(
  hot_entries: Vec<(Vec<u8>, Vec<u8>)>,
  max_pack_size: usize,
) -> FreezeResult {
  let mut cold_packs = Vec::new();
  let mut current_cold = Vec::new();
  let mut current_size = 0;
  let mut remaining_hot = Vec::new();

  // 目标大小：0.8x
  let target_cold_size = (max_pack_size as f64 * 0.8) as usize;

  for (key, value) in hot_entries {
    let entry_size = key.len() + value.len();

    // 特殊情况：单个 KV 超过 max_pack_size
    if entry_size > max_pack_size {
      // 直接输出为单独的 cold pack（允许超标）
      let pack_name = format!("cold_{}.pack", generate_unique_id());
      cold_packs.push(ColdPack {
        name: pack_name,
        entries: vec![(key, value)],
        size: entry_size, // 超过 max_pack_size，但允许
      });
      continue;
    }

    // 正常情况：累积到 0.8x
    if current_size + entry_size > target_cold_size && !current_cold.is_empty() {
      // 当前 cold pack 达到目标大小，输出
      let pack_name = format!("cold_{}.pack", generate_unique_id());
      cold_packs.push(ColdPack {
        name: pack_name,
        entries: current_cold,
        size: current_size,
      });
      current_cold = Vec::new();
      current_size = 0;
    }

    current_cold.push((key, value));
    current_size += entry_size;
  }

  // 处理剩余数据
  if !current_cold.is_empty() {
    if current_size >= target_cold_size {
      // 剩余数据足够大（>= 0.8x），输出为 cold
      let pack_name = format!("cold_{}.pack", generate_unique_id());
      cold_packs.push(ColdPack {
        name: pack_name,
        entries: current_cold,
        size: current_size,
      });
    } else {
      // 剩余数据较小，保留在 hot
      remaining_hot = current_cold;
    }
  }

  FreezeResult {
    cold_packs,
    remaining_hot,
  }
}

struct FreezeResult {
  cold_packs: Vec<ColdPack>,
  remaining_hot: Vec<(Vec<u8>, Vec<u8>)>,
}

struct ColdPack {
  name: String,
  entries: Vec<(Vec<u8>, Vec<u8>)>,
  size: usize,
}
```

### 示例场景

#### 场景 1: 正常冻结

```
max_pack_size = 100KB
target_cold_size = 80KB
hot_pack_size = 250KB

输入 hot entries:
  entry1: 30KB
  entry2: 40KB
  entry3: 50KB
  entry4: 60KB
  entry5: 70KB

处理：
  current_cold = [entry1, entry2]  // 70KB
  + entry3 (50KB) → 120KB > 80KB  // 输出

  cold_1.pack = [entry1, entry2]  // 70KB < 80KB，但后续会超标

  current_cold = [entry3]  // 50KB
  + entry4 (60KB) → 110KB > 80KB  // 输出

  cold_2.pack = [entry3, entry4]  // 110KB > 80KB，输出

  current_cold = [entry5]  // 70KB < 80KB

  剩余数据 < 80KB → 保留在 hot
  remaining_hot = [entry5]  // 70KB

结果：
  cold_1.pack (70KB)
  cold_2.pack (110KB)
  hot.pack ([entry5], 70KB)
```

#### 场景 2: 超大 KV

```
max_pack_size = 100KB
entry1: 150KB  // 单个 KV 超过限制

处理：
  entry_size (150KB) > max_pack_size (100KB)
  → 直接输出为单独的 cold pack

结果：
  cold_xxx.pack = [entry1]  // 150KB（允许超标）
```

#### 场景 3: 所有数据都是新数据

```
max_pack_size = 100KB
hot 中只有新写入的数据：50KB

处理：
  hot_pack_size (50KB) <= max_pack_size * 2 (200KB)
  → 不需要冻结

结果：
  hot.pack (50KB)
  无 cold pack（正常）
```

### unique_id 生成

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

fn generate_unique_id() -> String {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let mut hasher = FxHasher::default();
  timestamp.hash(&mut hasher);
  format!("{:x}", hasher.finish())
}

// 示例输出：cold_a1b2c3d4e5f6.pack
```

## Index 机制

### Index 文件格式

```
hot.pack,18446744073709551615,a1b2c3d4e5f6
cold_abc.pack,281474976710655,f6e5d4c3b2a1
cold_xyz.pack,562949953421311,1a2b3c4d5e6f
```

**格式：** `pack_name,exist_check_hash,file_hash`

**字段说明：**

- `pack_name`：pack 文件名
- `exist_check_hash`：u64，用于快速判断 key 是否可能存在
- `file_hash`：字符串，文件内容哈希，用于防篡改

### exist_check_hash（原 content_hash）

**用途：** 快速判断某个 key 是否**可能**存在于此 pack

**算法：** 按位或（Bitwise OR）

```rust
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// 生成 pack 的 exist_check_hash
fn generate_exist_check_hash(keys: &[Vec<u8>]) -> u64 {
  let mut result = 0u64;

  for key in keys {
    let mut hasher = FxHasher::default();
    key.hash(&mut hasher);
    let key_hash = hasher.finish();

    result |= key_hash; // 按位或
  }

  result
}

/// 检查 key 是否可能存在于此 pack
fn may_contain(key: &[u8], exist_check_hash: u64) -> bool {
  let mut hasher = FxHasher::default();
  key.hash(&mut hasher);
  let key_hash = hasher.finish();

  (key_hash & exist_check_hash) == key_hash // 按位与
}
```

**示例：**

```rust
// Pack 中的 keys
keys = [
  b"key1",  // hash = 0b1010
  b"key2",  // hash = 0b1100
  b"key3",  // hash = 0b0011
]

// exist_check_hash
exist_check_hash = 0b1010 | 0b1100 | 0b0011 = 0b1111

// 查询
may_contain(b"key1", 0b1111)
  → key_hash = 0b1010
  → (0b1010 & 0b1111) == 0b1010 ✓ 可能存在

may_contain(b"key4", 0b1111)  // hash = 0b0101
  → key_hash = 0b0101
  → (0b0101 & 0b1111) == 0b0101 ✓ 可能存在（误判）

may_contain(b"key5", 0b1111)  // hash = 0b10000 (超出范围)
  → key_hash = 0b10000
  → (0b10000 & 0b1111) == 0b0000 ✗ 一定不存在
```

**特性：**

- **无漏判**：如果 key 在 pack 中，`may_contain` 一定返回 true
- **可能误判**：如果 key 不在 pack 中，`may_contain` 可能返回 true（需要实际读取验证）
- **简单高效**：单次按位与操作

### file_hash

**用途：** 防止文件被篡改或损坏

**算法：** FxHasher

```rust
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// 生成文件内容哈希
fn generate_file_hash(content: &[u8]) -> String {
  let mut hasher = FxHasher::default();
  content.hash(&mut hasher);
  format!("{:x}", hasher.finish())
}

/// 验证文件哈希
fn verify_file_hash(content: &[u8], expected_hash: &str) -> bool {
  let actual_hash = generate_file_hash(content);
  actual_hash == expected_hash
}
```

**使用场景：**

```rust
// 写入时
async fn write_pack(path: &Path, entries: &[(Vec<u8>, Vec<u8>)]) -> Result<String> {
  let content = serialize_pack(entries)?;
  fs::write(path, &content).await?;

  let file_hash = generate_file_hash(&content);
  Ok(file_hash)
}

// 读取时
async fn read_pack(path: &Path, expected_hash: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
  let content = fs::read(path).await?;

  // 验证哈希
  if !verify_file_hash(&content, expected_hash) {
    return Err(Error::from_reason(
      Some(ErrorType::Load),
      None,
      format!("Pack file corrupted: hash mismatch"),
    ));
  }

  deserialize_pack(&content)
}
```

### Index 的更新时机

**何时更新 index 文件：**

- 每次 save() 生成新的 pack 文件时
- 完全重写 index 文件，包含所有 pack（hot + cold）

**示例：**

```rust
// save() 流程
async fn save_page(page_id: usize, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<()> {
  // 1. 读取现有 pack 文件
  let existing_packs = read_existing_packs(page_id).await?;

  // 2. 合并并分离 Hot/Cold
  let (new_hot, new_cold_packs) = merge_and_freeze(existing_packs, entries)?;

  // 3. 写入新 pack 文件
  let mut index_entries = Vec::new();

  // 写入 hot.pack
  let hot_content = serialize_pack(&new_hot)?;
  tx.add_file("page_X/hot.pack", &hot_content).await?;

  let hot_keys: Vec<_> = new_hot.iter().map(|(k, _)| k.clone()).collect();
  index_entries.push(IndexEntry {
    pack_name: "hot.pack".to_string(),
    exist_check_hash: generate_exist_check_hash(&hot_keys),
    file_hash: generate_file_hash(&hot_content),
  });

  // 写入 cold packs
  for cold_pack in new_cold_packs {
    let cold_content = serialize_pack(&cold_pack.entries)?;
    tx.add_file(format!("page_X/{}", cold_pack.name), &cold_content).await?;

    let cold_keys: Vec<_> = cold_pack.entries.iter().map(|(k, _)| k.clone()).collect();
    index_entries.push(IndexEntry {
      pack_name: cold_pack.name.clone(),
      exist_check_hash: generate_exist_check_hash(&cold_keys),
      file_hash: generate_file_hash(&cold_content),
    });
  }

  // 保留未修改的 old cold packs
  for old_cold in existing_cold_packs_to_keep {
    index_entries.push(old_cold.index_entry);
  }

  // 4. 写入新的 index 文件（完全重写）
  let index_content = serialize_index(&index_entries)?;
  tx.add_file("page_X/index", &index_content).await?;

  Ok(())
}
```

### Index 的使用（未来优化）

**当前：** load() 不使用 index，直接读取所有 pack

**未来：** 实现 get(key) 时使用 index 优化

```rust
// 未来的 get() 实现
async fn get(&self, scope: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
  let page_id = allocate_page(key, self.options.page_count);
  let index = read_index(scope, page_id).await?;

  // 使用 exist_check_hash 过滤
  for entry in index.entries {
    if may_contain(key, entry.exist_check_hash) {
      // 可能存在，读取 pack
      let pack = read_pack(&entry.pack_name, &entry.file_hash).await?;
      if let Some(value) = pack.get(key) {
        return Ok(Some(value));
      }
    }
  }

  Ok(None)
}
```

## 性能考虑

### Hot/Cold 分离

**优点：**

- 减少写放大：cold pack 不会被重复写入
- 优化读取：未来 get() 可以跳过 cold pack

**开销：**

- 冻结操作：需要读取、拆分、写入多个文件
- 仅在 hot 超过 2x 时触发，频率低

### exist_check_hash

**优点：**

- 快速过滤：单次按位与操作
- 减少磁盘 I/O：避免读取不相关的 pack

**缺点：**

- 可能误判：需要实际读取验证
- 误判率取决于 hash 分布

### file_hash

**开销：**

- 写入时计算一次哈希
- 读取时验证一次哈希
- 开销极小（FxHasher 非常快）

## TODO

- [ ] 实现 get(key) API，利用 index 优化
- [ ] 收集 exist_check_hash 的误判率数据
- [ ] 考虑动态调整 target_cold_size（当前固定 0.8x）
