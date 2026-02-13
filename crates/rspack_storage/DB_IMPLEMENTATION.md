# DB Storage Implementation Plan

## Overview

将现有的 Pack 存储重构为 DB 存储，对外保持 `Storage` trait 不变，内部使用 `DB` + `Bucket` 架构。

---

## 1. 文件组织结构

### 1.1 整体目录结构

```
root/
├── .temp/                  # 临时目录（事务使用）
│   ├── state.lock          # 进程锁
│   ├── commit.lock         # 提交锁
│   └── bucket1/            # 临时数据（与正式结构一致）
│       ├── 0.hot.pack
│       ├── 0.hot.index
│       └── bucket_meta.txt
├── bucket1/
│   ├── bucket_meta.txt     # Bucket 元数据
│   ├── 0.hot.pack          # 热数据块
│   ├── 0.hot.index         # 热数据索引
│   ├── 1.cold.pack         # 冷数据块
│   ├── 1.cold.index        # 冷数据索引
│   └── ...
└── bucket2/
    └── ...
```

### 1.2 文件说明

- **.temp/**: 临时目录，存储事务相关文件
  - **state.lock**: 进程锁，记录当前进程 PID
  - **commit.lock**: 提交锁，记录要添加/删除的文件列表
  - **bucket1/**: 临时数据，结构与正式目录一致
- **bucket_meta.txt**: Bucket 元数据（pages 列表，hot page ID）
- **N.hot.pack**: 热数据块，新写入和更新的数据
- **N.hot.index**: 热数据索引，存储 exist_check_hash
- **N.cold.pack**: 冷数据块，从 hot.pack 分裂而来
- **N.cold.index**: 冷数据索引

---

## 2. 配置参数

### 2.1 DBOptions

```rust
pub struct DBOptions {
    /// 每个 page 的条目数阈值，超过后分裂
    pub page_count: usize,

    /// 单个数据块的最大大小（字节）
    pub max_pack_size: usize,
}
```

### 2.2 参数作用说明

| 参数            | 作用                                   | 示例                |
| --------------- | -------------------------------------- | ------------------- |
| `page_count`    | Hot page 条目数阈值，超过后分裂成 Cold | 10 条，超过后分裂   |
| `max_pack_size` | 限制单个 pack 大小（当前未使用）       | max_pack_size=512KB |

---

## 3. 核心机制

### 3.1 热/冷数据块机制

#### 3.1.1 热数据块 (N.hot.pack)

**特性**:

- 每个 bucket 只有一个 hot page
- 条目数阈值：`page_count`
- 所有新写入和更新的数据先进入热数据块

**分裂条件**:

```
if hot page 条目数 > page_count:
    1. 取前 80% 条目作为 cold page
    2. 将 cold page 写入新的 N.cold.pack
    3. 后 20% 条目保留在新的 hot page
    4. hot page ID 递增
```

#### 3.1.2 冷数据块 (N.cold.pack)

**特性**:

- 每个 bucket 可以有多个 cold page
- 从 hot page 分裂而来，数据不可变
- 每个 cold page 对应一个 index 文件

### 3.2 索引机制

**位置**: 每个 pack 文件对应一个 `.index` 文件

**用途**:

- 存储该 pack 所有 key 的 exist_check_hash
- 用于快速判断 key 是否可能存在

**数据结构**:

```rust
// Index 文件只存储一个 u64 hash
pub struct Index {
  hash: u64,
}
```

**Hash 计算算法**:

```rust
// 生成 pack 的索引 hash
fn exist_check_hash(keys: &[&[u8]]) -> u64 {
  let mut hasher = DefaultHasher::new();
  for key in keys {
    key.hash(&mut hasher);
  }
  hasher.finish()
}
```

// 检查 key 是否可能在 pack 中
fn may_contain(key: &[u8], pack_hash: u64) -> bool {
let key_hash = hash(key);
(key_hash & pack_hash) == key_hash // 按位与操作
}

```

**查找流程**:

```

1. 计算 key 的 hash 值
2. 遍历 index 中的所有 pack:
   a. 如果 (key_hash & pack_hash) == key_hash:
   - 可能在该 pack 中，读取 pack 文件查找
   - 找到则返回 value
   - 未找到则继续下一个匹配的 pack
     b. 否则：一定不在该 pack 中，跳过
3. 所有 pack 都检查完毕，返回 None

```

**特性**:

- **快速过滤**: 按位与操作 O(1) 判断 key 是否可能在 pack 中
- **无假阴性**: 如果 key 在 pack 中，一定能匹配上
- **有假阳性**: 可能匹配上但 key 不在 pack 中（需要实际读取验证）
- **简单高效**: 每个 pack 只需一个 u64 值

---

## 4. 核心操作流程

### 4.1 Load All (全量读取 Bucket)

**用途**: 加载整个 bucket 的所有数据到内存（对应 Storage::load）

**流程**:

```

1. 读取 db_meta
2. 读取 bucket_meta，获取 page_count
3. 对每个 page（并行）:
   a. 读取 hot.pack，解析所有 key-value pairs
   b. 读取所有 cold pack，解析所有 key-value pairs
   c. 合并到内存 HashMap
4. 返回所有 key-value pairs

```

**注**: 全量读取不需要 index，直接读取所有 pack 文件

### 4.2 Get (读取单个 Key)

**用途**: 查找某个 key 的 value（未来可能的优化）

**流程**:

```

1. 计算 key 的 hash 值
2. 计算 key 所属的 page_index
3. 读取该 page 的 index 文件
4. 遍历 index 中的所有 pack:
   a. 如果 (key_hash & pack_hash) == key_hash:
   - 读取该 pack 文件
   - 查找 key，找到则返回 value
   - 未找到则继续下一个匹配的 pack
     b. 否则：跳过该 pack
5. 所有匹配的 pack 都未找到，返回 None

```

**注**: 当前 Storage trait 没有单独的 get 方法，此流程为未来优化预留

### 4.3 Set (写入/更新数据)

**内存操作** (立即执行):

```

1. 计算 key 所属的 page_index
2. 更新内存中的 data[page_index][key] = value
3. 标记 dirty = true

```

**持久化** (在 Save 时执行):

```

1. 对每个 dirty page:
   a. 将 data[page_index] 添加到 hot.pack 队尾
   b. 如果 hot.pack > max_pack_size \* 2:
   - 从队头取出 max_pack_size 数据
   - 写入新的 cold_N.pack
     c. 更新 index 文件（索引 hash）

```

### 4.4 Remove (删除数据)

**删除分两种情况**:

**情况 1: 删除在 hot.pack 中的 key**

```

1. 从 hot.pack 数据中移除该 key-value
2. 重新生成 hot.pack 文件
3. 更新 index

```

**情况 2: 删除在 cold_N.pack 中的 key**

```

1. 读取该 cold_N.pack 的所有数据到内存
2. 读取 hot.pack 的所有数据到内存
3. 从 cold_N.pack 数据中删除该 key
4. 合并: 将 cold_N.pack 剩余数据放到 hot.pack 队头
5. 删除原 cold_N.pack 文件
6. 重新生成 hot.pack 文件
7. 更新 index

```

### 4.5 Save (持久化)

**两阶段锁定策略**：

```

Phase 1: Read Lock - 数据准备阶段

1. 持有读锁（允许并发读取）
2. 对每个 scope:
   a. 加载现有数据
   b. 合并变更
   c. 调用 bucket.prepare_save() 准备新数据
   - 计算 hot/cold 分裂
   - 生成所有 pack/index 文件内容
   - 返回相对路径（如 "bucket1/0.hot.pack"）
3. 收集所有待添加/删除文件列表

Phase 2: Write Lock - 事务提交阶段

1. 释放读锁，获取写锁（独占访问）
2. 创建 Transaction
3. tx.begin():
   - 检查 .temp/state.lock，处理崩溃恢复
   - 创建新的 .temp/state.lock（记录当前 PID）
   - 清空 .temp 目录
4. 添加文件到事务:
   - tx.add_file("bucket1/0.hot.pack", content)
     → 写入到 .temp/bucket1/0.hot.pack
   - tx.add_file("bucket1/0.hot.index", content)
     → 写入到 .temp/bucket1/0.hot.index
5. tx.commit():
   - 验证 state.lock 的 PID 匹配
   - 写入 .temp/commit.lock（记录所有操作）
   - 移动文件: .temp/bucket1/0.hot.pack → bucket1/0.hot.pack
   - 删除旧文件
   - 删除 .temp/commit.lock
   - 删除 .temp/state.lock

```

**设计优势**：

- **最小化写锁时间**: 数据准备（计算、序列化）在读锁下完成
- **提高并发性**: 准备阶段允许其他读操作并发执行
- **原子性**: 写锁阶段只做文件移动，保证事务完整性
- **崩溃恢复**: 通过 state.lock 和 commit.lock 实现自动恢复


---

## 5. 数据文件格式

### 5.1 Pack 文件格式 (hot.pack / cold_N.pack)

**格式说明**:

```

第 1 行: key_size_0 key_size_1 key_size_2 ... (空格分隔)
第 2 行: value_size_0 value_size_1 value_size_2 ... (空格分隔)
第 3 行开始: 连续的二进制数据

- 先是所有 keys 的二进制数据（按顺序）
  - key_0 的字节 (key_size_0 字节)
  - key_1 的字节 (key_size_1 字节)
  - ...
- 然后是所有 values 的二进制数据（按顺序）
  - value_0 的字节 (value_size_0 字节)
  - value_1 的字节 (value_size_1 字节)
  - ...

```

**示例**:

假设有 2 个 key-value 对：

- key_0 = "abc" (3 字节), value_0 = "hello" (5 字节)
- key_1 = "xy" (2 字节), value_1 = "world!" (6 字节)

文件内容：

```

3 2
5 6
abcxyhelloworld!

````

**读取流程**:

```rust
// 1. 读取第 1 行，解析 key sizes
let key_sizes = read_line().split(' ').map(|s| s.parse::<usize>()).collect(); // [3, 2]

// 2. 读取第 2 行，解析 value sizes
let value_sizes = read_line().split(' ').map(|s| s.parse::<usize>()).collect(); // [5, 6]

// 3. 读取所有 keys
let mut keys = vec![];
for size in key_sizes {
    keys.push(read(size));  // "abc", "xy"
}

// 4. 读取所有 values
let mut values = vec![];
for size in value_sizes {
    values.push(read(size));  // "hello", "world!"
}

// 5. 组合成 key-value pairs
let pairs = keys.into_iter().zip(values).collect();
````

**写入流程**:

```rust
// 1. 写入第 1 行：所有 key 的大小
write_line(keys.iter().map(|k| k.len()).join(" "));  // "3 2"

// 2. 写入第 2 行：所有 value 的大小
write_line(values.iter().map(|v| v.len()).join(" "));  // "5 6"

// 3. 写入所有 keys 的二进制数据
for key in keys {
    write(key);  // "abc", "xy"
}

// 4. 写入所有 values 的二进制数据
for value in values {
    write(value);  // "hello", "world!"
}

flush();
```

**特点**:

- ✅ 前两行是文本格式（便于调试和检查）
- ✅ 数据部分是二进制格式（紧凑存储）
- ✅ 沿用现有 Pack 实现的格式（不含 generation）
- ✅ Keys 和 Values 分别连续存储，便于跳过读取

### 5.2 Index 文件格式

**格式说明**:

```
每行一条记录，格式为：pack_file_name,content_hash,file_hash
使用逗号分隔
```

**字段说明**:

- `pack_file_name`: pack 文件名（如 hot.pack 或 cold_abc.pack）
- `content_hash`: 该 pack 所有 key 的索引 hash（按位或生成，用于快速查找）
- `file_hash`: 该 pack 文件的内容 hash（用于防篡改验证）

**示例**:

```
hot.pack,18446744073709551615,a1b2c3d4e5f6
cold_abc123.pack,281474976710655,f6e5d4c3b2a1
cold_def456.pack,562949953421311,123456789abc
```

**读取流程**:

```rust
// 读取整个 index 文件
let lines = read_all_lines();

// 解析每一行
let index: Vec<PackIndexEntry> = lines
    .iter()
    .map(|line| {
        let parts: Vec<&str> = line.split(',').collect();
        PackIndexEntry {
            pack_name: parts[0].to_string(),
            content_hash: parts[1].parse::<u64>().unwrap(),
            file_hash: parts[2].to_string(),
        }
    })
    .collect();
```

**写入流程**:

```rust
// 对每个 pack 文件写入一行
for entry in index {
    write_line(format!("{},{},{}", entry.pack_name, entry.content_hash, entry.file_hash));
}
flush();
```

**特点**:

- ✅ 纯文本格式，便于调试和人工检查
- ✅ 每行一条记录，简单清晰
- ✅ content_hash 用于快速查找（按位或/与操作）
- ✅ file_hash 用于防篡改验证（类似现有 Pack 的实现）

### 5.3 Bucket Meta 格式

**格式说明**: 存储 bucket 的配置信息（尽量少更新或不更新）

```
第 1 行: page_count max_pack_size (空格分隔)
```

**示例**:

```
10 524288
```

**解释**:

- `page_count=10`: 分页数量
- `max_pack_size=524288`: 单个 pack 最大大小

**注**:

- Bucket Meta 只存储配置，不存储 pack 文件列表
- Pack 文件列表由各 page 的 index 文件管理
- 目的：减少 meta 文件的更新频率

### 5.4 DB Meta 格式

**格式说明**: 存储 DB 的全局信息

```
第 1 行: version
第 2 行: bucket 列表（逗号分隔）
```

**示例**:

```
1.0.0
snapshot,module_graph,meta
```

第 1 行: page_count max_pack_size (空格分隔)
第 2+ 行: 每个 page 的 pack 文件列表（每行一个 page）

```

**示例**:

```

10 524288
hot.pack,abc123,1024 cold_1.pack,def456,2048
hot.pack,789012,512

```

**解释**:

- 行 1: `page_count=10, max_pack_size=524288`
- 行 2: page_0 的 pack 列表
  - `hot.pack,abc123,1024` (文件名,hash,大小)
  - `cold_1.pack,def456,2048`
- 行 3: page_1 的 pack 列表
  - `hot.pack,789012,512`

**注**:

- DB 实现中不需要 generation 字段
- 每个 pack 的格式：`name,hash,size`（逗号分隔）
- 同一行的多个 pack 用空格分隔

### 5.4 DB Meta 格式

**格式说明**: 沿用现有实现的格式

```

第 1 行: version
第 2 行: bucket 列表（逗号分隔）

```

**示例**:

```

1.0.0
snapshot,module_graph,meta

````

---

## 6. 实现阶段

### Phase 1: 基础结构

- [ ] `db/mod.rs` - DB 核心结构
- [ ] `db/bucket.rs` - Bucket 结构
- [ ] `db/page.rs` - Page 结构
- [ ] `db/options.rs` - DBOptions 定义
- [ ] `db/meta.rs` - Meta 文件读写

### Phase 2: 数据读写

- [ ] `db/pack/reader.rs` - Pack 文件读取
- [ ] `db/pack/writer.rs` - Pack 文件写入
- [ ] `db/pack/format.rs` - Pack 格式定义

### Phase 3: 索引机制

- [ ] `db/index/mod.rs` - Index 索引 hash 实现
- [ ] `db/index/reader.rs` - Index 文件读取
- [ ] `db/index/writer.rs` - Index 文件写入

### Phase 4: 冷热分离

- [ ] `db/hot_cold.rs` - 热/冷数据块管理
- [ ] 实现热区块冻结逻辑
- [ ] 实现冷区块回收逻辑

### Phase 5: 事务集成

- [ ] 集成现有的 `Transaction`
- [ ] 实现 Save 流程
- [ ] 实现错误恢复

### Phase 6: Storage 适配

- [ ] 实现 `Storage` trait for `DB`
- [ ] 保持 API 兼容

### Phase 7: 优化

- [ ] 并行加载优化
- [ ] 并行保存优化
- [ ] 内存优化

---

## 7. 关键设计决策

### 7.1 为什么需要分页（Page）？

**问题**: 单个 bucket 可能有 1GB+ 数据
**方案**: 分成 10 个 page，每个约 100MB
**优势**:

- 降低单次 I/O 数据量
- 支持并行读写
- 减少锁竞争

### 7.2 为什么需要热/冷分离？

**问题**: 频繁写入会导致反复重写大文件
**方案**: 新数据写入 hot.pack，满了再冻结为 cold.pack
**优势**:

- 减少写放大（Write Amplification）
- hot.pack 小，重写开销低
- cold.pack 不变，读取稳定

### 7.3 为什么 hot.pack = 2x max_pack_size？

**原因**:

1. 接收新写入数据的缓冲空间
2. 接收冷区块迁移的数据（删除/更新场景）
3. 避免频繁触发冻结

**示例**:

- max_pack_size = 50MB
- hot.pack 容量 = 100MB
- 当 hot.pack > 100MB 时，取 50MB 冻结为 cold.pack

### 7.4 为什么需要索引 Hash？

**问题**: 查找一个 key 需要遍历所有 pack 文件
**方案**: 使用按位或/与操作的简化 Bloom Filter
**优势**:

- O(1) 按位与操作判断 key 是否可能存在
- 每个 pack 只需 8 字节（u64）
- 无假阴性，有假阳性（需实际读取验证）
- 避免无效的文件读取

### 7.5 删除时为什么要回收冷区块？

**问题**: 删除/更新冷区块中的 key 会产生空洞
**方案**: 将整个 cold.pack 回收到 hot.pack
**优势**:

- 避免碎片化
- 保持文件紧凑
- 简化实现（不需要 compaction）

---

## 8. 性能考虑

### 8.1 读性能

- **分页**: 并行加载多个 page
- **索引**: 索引 Hash 快速定位
- **缓存**: 内存中保持加载的数据

### 8.2 写性能

- **批量写入**: 所有修改先在内存，Save 时批量持久化
- **热区块**: 只重写小的 hot.pack
- **并行**: 多个 page 并行写入

### 8.3 空间效率

- **紧凑格式**: 最小化元数据开销
- **碎片控制**: 冷区块回收机制

---

## 9. 待定问题

### 9.1 Hash 函数选择

- **候选算法**: FxHash, XxHash, SipHash 等
- **需考虑**: 速度 vs 分布均匀性

### 9.2 Pack 文件命名策略

- **hot.pack**: 固定名称
- **cold pack**: 当前使用内容 hash 生成，沿用现有实现
- **注**: 文档中使用 `cold_N.pack` 便于理解，实际实现可能不同

### 9.3 并发控制细节

- **当前**: 使用 Transaction 的 state.lock 单进程锁
- **未来**: 是否需要支持多进程并发？

---

## 10. 实现优先级

### P0 (核心功能)

- 基础 DB / Bucket / Page 结构
- Pack 文件读写
- Load / Set / Remove / Save 基本流程
- Transaction 集成

### P1 (性能优化)

- 索引 Hash 机制
- 热/冷分离机制
- 并行加载/保存

### P2 (高级特性)

- 统计信息（bucket 大小、pack 数量等）
- 性能监控

---

## 11. 测试计划

### 11.1 单元测试

- Pack 文件读写正确性
- 索引 Hash 准确性（按位或/与操作）
- 热/冷分离逻辑

### 11.2 集成测试

- 完整的 Load → Set → Save 流程
- Transaction 恢复逻辑
- 多 bucket 并发

### 11.3 性能测试

- 大数据量加载性能（1GB+）
- 频繁写入性能
- 查询性能（有/无索引 Hash）

---

## 12. 迁移计划

### 12.1 兼容性

- **API**: Storage trait 保持不变
- **数据**: 新旧格式不兼容，需要迁移工具或重新构建缓存

### 12.2 迁移策略

- **方案 1**: 清空旧缓存，使用新 DB（推荐，简单）
- **方案 2**: 提供迁移工具，从 Pack 格式转换为 DB 格式

---

## 附录A：术语表

| 术语        | 说明                                   |
| ----------- | -------------------------------------- |
| DB          | 数据库实例，管理所有 bucket            |
| Bucket      | 独立的键值空间，对应原来的 Scope       |
| Page        | Bucket 内的数据分页，降低单文件大小    |
| Hot Pack    | 热数据块，接收新写入和更新             |
| Cold Pack   | 冷数据块，从 hot.pack 冻结而来         |
| Index Hash  | 简化的 Bloom Filter，基于按位或/与操作 |
| Transaction | 事务，保证文件操作的原子性             |

---

## 附录B：操作对比

### Load All vs Get

| 操作         | 用途            | 是否使用 Index | 读取范围         | 适用场景             |
| ------------ | --------------- | -------------- | ---------------- | -------------------- |
| **Load All** | 加载整个 bucket | ❌ 不使用      | 所有 pack 文件   | 批量操作、初始化缓存 |
| **Get**      | 查找单个 key    | ✅ 使用        | 匹配的 pack 文件 | 点查询、按需加载     |

### Storage trait 映射

```rust
trait Storage {
    // 当前实现：对应 Load All 操作
    async fn load(&self, scope: &'static str) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>>;

    // 未来可能添加：对应 Get 操作
    // async fn get(&self, scope: &'static str, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
````

### 为什么 Load All 不需要 Index？

1. **全量读取**: 需要读取所有 pack 文件，index 无法减少 I/O
2. **简化流程**: 避免先读 index 再读 pack 的两次 I/O
3. **批量操作**: 适合一次性加载所有数据的场景

### 为什么需要 Get 操作（未来优化）？

1. **按需加载**: 只需要少量 key 时，避免全量读取
2. **减少 I/O**: 通过 index 快速定位，只读取必要的 pack
3. **性能优化**: 大 bucket 场景下，点查询更高效

---

**文档版本**: v0.2  
**更新时间**: 2026-02-11  
**状态**: Draft - 待审核
