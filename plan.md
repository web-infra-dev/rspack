# JS Hook Register 感知真实 Tap 的执行计划

## 目标

当前 `JsHooksAdapterPlugin` 把每个 JS hook register 当作普通 `Interceptor` 安装到 Rust hook。这样即使 JS 侧没有真实 tap，`HookCommon::is_empty()` 也会因为存在 interceptor 返回 `false`，hook 调用仍会进入 tracing、register/附加 tap 分支，Rust 侧也无法单独判断 JS hook 是否被使用。

本次改造需要达到以下结果：

- JS register 不再占用通用 interceptor 槽位，而是通过 hook 的专用加载接口安装。
- 新增 `JsTapRegister`，用类型擦除的函数封装“按 Rust tap stages 获取 JS taps”的过程，并在 Rust 侧保存 JS register 最近一次上报的 tap 数量。
- `Hook::is_empty()` 同时考虑 Rust taps、普通 interceptors 和 JS 真实 taps；仅安装了一个当前为空的 JS register 时仍然返回 `true`。
- hook 对外提供 `has_js_taps()`，不触发 N-API/register 调用即可检查 JS 侧是否有 tap。
- 每种 hook 的 `call` 在创建 tracing span、调用 register 或准备 futures 之前先检查 `is_empty()`，为空时按该 hook 的返回语义直接结束。
- `define_hook!` 只生成 hook 参数签名和具体 `tap.run(...)` 所必需的代码；JS register 存储、空状态、执行路径选择、类型校验和 stage 准备下沉到 `rspack_hook` 的非泛型共享实现，避免每个 hook 重复展开以及引入新的泛型单态化膨胀。
- 保持现有 Rust/JS tap 的 stage 排序、register 缓存、动态 tap 状态更新以及普通 interceptor 行为不变。

## 当前执行流

涉及文件：

- `crates/rspack_hook/src/lib.rs`
- `crates/rspack_macros/src/hook.rs`
- `crates/rspack_binding_api/src/plugins/interceptor.rs`
- `crates/rspack_binding_api/src/plugins/js_hooks_plugin.rs`
- `packages/rspack/src/Compiler.ts`

```text
Compiler.ts
  根据 lite-tapable 的 isUsed() 更新 NonSkippableRegisters
       ↓
JsHooksAdapterPlugin::apply / compilation adapter
  将每个 Register* 克隆后通过 hook.intercept(...) 安装
       ↓
HookCommon
  interceptor_count > 0，因此 is_empty() 恒为 false
       ↓
Hook::call
  调用所有 interceptor
       ↓
RegisterJsTapsInner
  检查 NonSkippableRegisters，必要时调用 JS register
       ↓
将返回的 JS taps 与 Rust taps 按 stage 合并执行
```

问题的核心是：JS 侧已经能计算 hook 是否被使用、register 也能得到最终生成的 `JsTap[]`，但这些信息没有记录到具体 Rust hook。目标不是让 `is_empty()` 回调 JS 查询，而是由 JS 在既有注册流程中主动把数量同步到 Rust，之后热路径只读 Rust 本地状态。

## 目标设计

### 1. 在 `rspack_hook` 增加非泛型的 `JsTapRegister`

在 `crates/rspack_hook/src/lib.rs` 中增加单一的非泛型 `JsTapRegister`。不要定义 `JsTapRegister<T>`，也不要为每种 hook tap trait 单态化一套 register 容器。建议包含以下成员：

- `owner`：对 binding 提供的 `Arc<RegisterJsTapsInner>` 做所有权类型擦除后得到的单一 `Arc<dyn Any + Send + Sync>`（或等价 erased owner）。它与各 `Register*` 指向同一个 inner allocation，不再单独分配 `Arc<AtomicUsize>`。
- `load_tap_count`：非捕获 Rust function pointer/accessor，从 erased owner 中读取 inner 内嵌的 `AtomicUsize`；`is_empty()` 只经过这层 Rust 本地访问，不触发 JS/N-API 调度。
- `tap_type_id: TypeId`：记录 register 输出中承载的具体 hook tap trait-object 类型，在加载 register 时校验一次，避免错误 register 到执行期才静默失败。
- `function`：以 erased owner 和 Rust hook 已使用的 `Vec<i32>` 为输入，返回类型擦除的 tap payload，例如 `Vec<Box<dyn Any + Send + Sync>>`。用内部枚举保存以下两种已擦除函数：
  - async：非泛型 boxed function，返回类型擦除 tap vector 的 boxed future；
  - blocking：非泛型 boxed function，直接返回类型擦除 tap vector。

当前 binding 中的 JS hooks 都走 async register 构造；blocking 变体用于保证 hook 抽象完整，并允许在宏测试中覆盖 `Sync` hook。不要在同步 hook 中阻塞等待 async future。

为该类型提供最小接口：

- `new_async<T, O>(owner: Arc<O>, ...)` / `new_blocking<T, O>(...)`：泛型只出现在很薄的所有权/类型擦除边界，用来记录 `TypeId`，把 owner 和 `Vec<T>` 擦除；容器、存储和实际调度方法本身保持非泛型。
- `tap_count()` / `is_empty()`：通过 Rust accessor 对 inner 内嵌的 `AtomicUsize` 做 acquire load，供 hook 的 `is_empty()` 和 `has_js_taps()` 使用。
- `call_erased(used_stages)` / `call_erased_blocking(used_stages)`：共享、非泛型调度方法；仅在确认存在 JS taps 后调用匹配执行模式的 register 函数。

使用按值传入的 `Vec<i32>` 和 `'static` boxed future，避免让公共类型依赖具体 hook 类型或 binding 类型，也避免把 `rspack_binding_api` 的 N-API 类型泄漏到 `rspack_hook`。`JsTapRegister` 持有 erased owner，因此异步 future 可以安全借用或克隆同一个 inner owner；不使用裸指针指向内嵌 count。类型擦除带来的转换只发生在 JS register 边界，不改变 Rust 原生 taps 的存储和热执行路径。

### 2. 将 hook 共享状态和决策下沉到 `HookCommon`

扩展 `crates/rspack_hook/src/lib.rs` 的非泛型 `HookCommon`，集中保存和实现：

- `js_tap_register: Option<JsTapRegister>`，默认值为 `None`，每个 hook 最多加载一个 JS register。
- `load_js_tap_register(register, expected_tap_type_id)`：校验输出 tap 类型并设置 register，不增加 `interceptor_count`；重复加载必须显式报错或断言。
- `has_js_taps()`、`is_empty()`、`used_stages()` 等状态查询。
- 非泛型的 `HookCallMode`（例如 `Empty`、`RustOnly`、`WithAdditionalTaps`），让宏生成的 call 只做一次共享决策，不重复生成 interceptor/JS register 状态组合判断。
- JS register 的非泛型调用、计数读取以及 erased tap vector 的返回。

不要新增 `HookStorage<T>`、`HookCommon<T>` 或把整套执行器做成泛型 helper；这类抽象虽然减少 token 数，却仍会为每种 tap 类型单态化，不能解决最终二进制膨胀。

### 3. 让 `define_hook!` 只生成不可共享的薄胶水

修改 `crates/rspack_macros/src/hook.rs`，生成的具体 hook 继续只保存强类型 Rust taps 和普通 interceptors，但 JS register 留在 `HookCommon`。宏生成代码缩减为：

- hook trait 与参数/返回值签名。
- `tap()` 时把 stage 和强类型 Rust tap 写入现有存储。
- `Hook` trait 方法对 `HookCommon` 的薄委托，包括专用 `load_js_tap_register(...)`。
- `call` 最外层根据 `HookCommon::call_mode()` 处理空返回；需要 JS taps 时调用共享的 erased register，再通过一个很薄的 `unerase_taps::<Self::Tap>` 边界恢复本 hook 的 tap trait object。
- 各执行类型真正无法共享的 `tap.run(args...)`、bail/waterfall 返回传播和 parallel future 组装。

类型恢复 helper 允许有一个很小的泛型实例，但只负责 `TypeId` 校验后的 downcast/unbox；register 调度、状态机、计数、缓存入口和 stage 索引逻辑不能放进该泛型函数。这样把单态化范围限制在必要的 tap 类型转换上。

`Hook` trait 增加专用加载入口，使 binding adapter 可以统一安装 register；具体 hook 同时保留易于调用的 `has_js_taps()`/`is_empty()` 查询方法，但这些方法体只委托 `HookCommon`，不在每个宏展开中复制状态逻辑。

### 4. 在所有 hook call 的真正入口增加空路径快速返回

在宏生成 `call` 的最外层先执行 `self.is_empty()`，并且把检查放在 tracing span 和 register 调用之前。不同 hook 类型的空返回值如下：

| Hook 类型 | 空 hook 返回值 |
| --- | --- |
| `Series` / `Sync` / `Parallel` | `Ok(())` |
| `SeriesBail` | `Ok(None)` |
| `SeriesWaterfall` | `Ok(data)`，原样返回 waterfall 输入 |

通过空检查后，再区分两条执行路径：

- 没有普通 interceptor 且 `has_js_taps() == false`：直接执行已排序的 Rust taps，不构造额外 tap 容器。
- 存在普通 interceptor 或 JS taps：收集普通 interceptor 返回的 taps；仅当 `has_js_taps()` 为真时调用 JS register；最后复用 `merged_tap_indices_by_stage` 合并执行。

这样即使 hook 有 Rust taps、但 JS register 当前为空，也不会为 JS register 支付 N-API 或 future 分配成本。

### 5. 将 binding register 转换为 `JsTapRegister`

修改 `crates/rspack_binding_api/src/plugins/interceptor.rs`：

- 将所有 `Register*` 改为 `inner: Arc<RegisterJsTapsInner>`；`Clone` 只克隆这一层 owner，不再手工 clone inner 的 register、cache 和状态字段。
- `RegisterJsTapsInner` 直接内嵌 `tap_count: AtomicUsize`、`always_active`/bootstrap 标记和 register cache。`RegisterJsTapsCache::Cache` 改为直接持有 `RwLock<Option<RegisterFunctionOutput>>`，因为外层 inner 已经由 `Arc` 共享，不需要 `Arc<RwLock<...>>` 的第二层共享所有权。
- 在 inner 上集中提供 `tap_count()` 和 `set_tap_count(count)`；setter 对 `always_active` 统一执行 `max(count, 1)`，所有单个/批量更新都必须走该方法，避免某条路径把 bootstrap register 写成 0。
- `RegisterJsTapsInner::call_register` 改为直接接收 `Vec<i32>`（或 stages slice 后在边界复制），不再接收 `impl Hook`；排序/去重由 hook 的 `used_stages()` 保证。
- `RegisterJsTapsInner` 不再保存 `Option<NonSkippableRegisters>`，也不再另外分配 `Arc<AtomicUsize>`；`JsTapRegister` 类型擦除并持有同一个 `Arc<RegisterJsTapsInner>`，从 inner 内部读取 count。
- `skip = false` 的 bootstrap register 初始计数设为非零，避免首次 `CompilerThisCompilation` 被空路径跳过；`skip = true` 的 register 初始为 0，并由首次 bootstrap 状态同步写入 0/非 0。
- 调整 `define_register!`：删除各 `Register*` 对 `Interceptor<SpecificHook>` 的实现，改为生成非泛型 `JsTapRegister`。宏只在构造边界把具体 `$tap_name` trait object 包装成 erased tap payload，不为每个 hook 生成一套 register 调度实现。
- register function 通过 `JsTapRegister` 持有的 erased owner 取回 `RegisterJsTapsInner`，调用原有缓存逻辑，并继续把类型擦除的 `ThreadsafeJsTap` 转成对应的 `$tap_name` trait object，再统一擦除输出；function 本身不额外捕获另一份状态 `Arc`。
- 各 `Register*::clear_cache()` 直接操作 `self.inner.cache`，确保 adapter、hook 和 plugin cache cleanup 始终落在同一个 inner allocation。

新的所有权关系如下：

```text
Register* ───────────────┐
JsTapRegister(erased) ───┼─ Arc<RegisterJsTapsInner>
binding count setter ────┘      ├─ RegisterFunction
                                ├─ RwLock<Cache>
                                ├─ AtomicUsize tap_count
                                └─ bootstrap/always_active 标记
```

这里存在多个 `Arc` handle，但只有一层 `Arc` allocation，不是 `Arc<Inner>` 再嵌套 `Arc<AtomicUsize>`/`Arc<RwLock<...>>`。inner 不反向引用 hook 或 plugin，因此不会形成引用环。

### 6. 由 JS register 主动向 Rust 上报数量

修改 `crates/rspack_binding_api/src/lib.rs` 和 `crates/rspack_binding_api/src/plugins/js_hooks_plugin.rs`：

- 增加同步 N-API 方法 `setRegisterJsTapCount(kind, count)`；它只根据 `RegisterJsTapKind` 找到对应 register 并调用 `register.inner.set_tap_count(count)`，不执行 hook/register，也不反向调用 JS。
- 删除 `NonSkippableRegisters(Arc<RwLock<Vec<...>>>)`。`JsHooksAdapterPlugin` 已持有全部 `Register*`，因此 count 更新可以直接按 kind 路由到对应 inner。
- 现有 `setNonSkippableRegisters(kinds)` 暂时保留为 bootstrap/动态失效的批量 N-API 入口，但不再写全局集合：它遍历/路由各 `Register*`，把 kind 在集合中的 inner count 写成 1、其余写成 0；`always_active` 项保持至少为 1。
- 单个 count 更新和批量状态更新最终都写各自 `RegisterJsTapsInner` 内同一个 `AtomicUsize`，不维护额外的共享 map、count handle 或全局锁。

修改 `packages/rspack/src/Compiler.ts`：

- 在 `#createHookRegisterTaps` 和 `#createHookMapRegisterTaps` 生成的 JS register 中，完成 stage range 查询并得到最终 `jsTaps` 后，额外同步调用一次 `setRegisterJsTapCount(registerKind, jsTaps.length)`，再返回 `jsTaps`。
- 这个额外调用发生在 JS register 本来就已被 Rust 调度到 JS 的过程中，不由 `is_empty()` 发起；因此不会给每次空检查增加 Rust/JS 往返。
- 保留 `#updateNonSkippableRegisters()` 的职责，用于首次 `thisCompilation` bootstrap，以及 tap 在执行过程中新增/移除后、下一次 register 尚未运行前，批量同步 0/非 0 状态。register 真正运行后再用 `jsTaps.length` 覆盖为精确数量。
- `#decorateJsTaps` 仍在最后一个 tap 完成后刷新状态，避免 tap 自移除后 Rust 继续把该 hook 判断为非空。

计数的语义是“本次 register 生成的 JS stage-range tap 包装数量”。对于 `is_empty()`，只要求 `0` 与 `> 0` 准确；bootstrap 批量同步可以先写 0/1，register 执行后再写精确值。

### 7. 替换所有 JS adapter 的安装点

修改 `crates/rspack_binding_api/src/plugins/js_hooks_plugin.rs`，将所有 JS register 的：

```text
hook.intercept(register.clone())
```

统一替换为：

```text
hook.load_js_tap_register(register.clone().into_js_tap_register())
```

覆盖以下安装阶段，避免遗漏仅在 compilation 创建后才取得的插件 hooks：

- `JsHooksAdapterPlugin::apply` 中的 compiler、compilation、NormalModuleFactory、ContextModuleFactory hooks。
- `js_hooks_adapter_compilation` 中的 JavaScript modules hooks。
- `html_hooks_adapter_compilation` 中的 HtmlRspackPlugin hooks。
- `runtime_hooks_adapter_compilation` 中的 RuntimePlugin hooks。
- `real_content_hash_hooks_adapter_compilation` 中的 RealContentHashPlugin hooks。
- `rsdoctor_hooks_adapter_compilation` 中的 RsdoctorPlugin hooks。

普通 Rust interceptor 的 API 和已有测试保持不动；本次只迁移 JS register。

## 改造后的执行流

```text
Compiler.ts
  thisCompilation bootstrap / 动态变更
  根据 hook/hookMap.isUsed() 批量同步 0/非 0
       ↓
binding 的 Register*
  在薄构造边界生成非泛型 JsTapRegister
  └─ 类型擦除同一个 Arc<RegisterJsTapsInner>
       ├─ AtomicUsize tap_count
       ├─ RwLock cache
       └─ RegisterFunction
       ↓
hook.load_js_tap_register(...)
  HookCommon 校验 TypeId 后存入共享槽位
  不计入 interceptor_count
       ↓
Hook::call 最前面检查 is_empty()
  HookCommon::call_mode() 只读取 Rust 本地状态
  ├─ Rust tap / interceptor / JS tap 全部为空 → 按 hook 类型直接返回
  ├─ 仅 Rust taps → 直接执行强类型 taps
  └─ 需要 additional taps
       ↓
has_js_taps()
  ├─ false → 不调用 JS register，只执行 Rust taps/普通 interceptors
  └─ true  → HookCommon 调用非泛型 register，得到 erased JS taps
       ↓
JS register
  setRegisterJsTapCount(kind, jsTaps.length)
  同步把本次精确数量写回 Rust
       ↓
define_hook 薄胶水
  校验后恢复为当前 hook 的 tap trait object
       ↓
merged_tap_indices_by_stage
  合并 Rust taps、普通 interceptor taps 和 JS taps
       ↓
按 Series / Bail / Waterfall / Parallel 语义执行
```

## 顺序与兼容性约束

- Rust tap 在 `tap()` 时仍按 stage 稳定插入；JS register 返回的 tap stage 仍由 `Compiler.ts` 的 stage range 查询生成。
- 继续使用 `merged_tap_indices_by_stage`，确保 Rust/JS taps 的主排序规则不变；相同 stage 时保持当前 Rust base tap 优先于 additional/JS tap 的规则。
- JS register 是单独的 additional tap 来源；若同时存在普通 interceptors，应固定收集顺序并加测试，保证同 stage 的结果确定。建议普通 interceptors 保持注册顺序，JS taps 固定追加在其后。
- `has_js_taps()` 只表示 JS 侧状态，不把 Rust taps 或普通 interceptors 算进去；`is_empty()` 才表示整个 hook 是否为空。
- `JsTapRegister`、`HookCommon`、register function enum、erased tap 容器和 call-mode 状态机必须是非泛型具体类型；禁止把它们改成按 hook/tap 类型实例化的泛型 storage/executor。
- 宏展开中只保留强类型参数调用所必需的逻辑。共享状态判断、JS register 调度和类型检查不得以内联 token 的形式在每个 `define_hook!` 展开中复制。
- erased tap 恢复产生的额外分配/转换只允许出现在 JS register 路径；Rust-only fast path不能增加 `Any`、downcast、boxed future 或额外分配。
- `has_js_taps()` 和空路径检查不得调用类型擦除的 register，也不得触发 JS/N-API 调度；共享 helper 只能通过 Rust accessor 读取 `Arc<RegisterJsTapsInner>` 内嵌的 `AtomicUsize`。JS→Rust 的额外调用只能出现在 JS register 已经执行的路径中。
- count/cache/register 必须共享同一个 `Arc<RegisterJsTapsInner>` owner；禁止重新引入 `Arc<AtomicUsize>`、`Arc<RwLock<Cache>>` 或全局 `NonSkippableRegisters` 锁。
- JS 写入 count 后，后续 Rust `is_empty()` 必须直接看到更新；使用成对的 release store / acquire load（或更强顺序）明确跨线程可见性。
- `skip = false` 的 register 初始计数继续保持非零；这是现有 JS register 状态初始化的 bootstrap 约束，不能在首次 `thisCompilation` 前把它判断为空。
- 批量 0/1 更新与 register 的精确 count 更新可能先后发生，必须按调用时序写入同一原子值；不要让较旧的异步任务覆盖较新的状态。新增的 N-API setter 应为同步写入。
- register 抛错、JS tap 抛错、bail 和 waterfall 的错误/返回传播方式保持现状。

## 测试计划

### Rust 宏行为测试

在专用测试 crate 的 `crates/rspack_macros_test/tests/hook.rs` 中增加一个由 `Arc<MockRegisterInner>` 整体共享、count 内嵌为 `AtomicUsize` 的 mock `JsTapRegister`，覆盖：

1. 默认 hook：`is_empty() == true`、`has_js_taps() == false`，调用后直接返回。
2. 已加载但计数为 0 的 JS register：hook 仍为空，且 register 回调调用次数保持为 0。
3. Rust 计数从 0 切换为非零：`has_js_taps()` 和 `is_empty()` 立即反映变化，调用时执行 JS tap；断言空检查没有调用任何状态查询函数。
4. 存在 Rust tap、JS register 为空：只执行 Rust tap，不调用 JS register。
5. Rust tap、普通 interceptor tap、JS tap 同时存在：验证 stage 合并和同 stage 的确定性顺序。
6. 分别覆盖 `Series`、`Sync`、`SeriesBail`、`SeriesWaterfall`、`Parallel` 的空返回语义，尤其验证 waterfall 原样返回输入、bail 返回 `None`。
7. 定义至少两个 tap trait 不同的 hook，验证它们共用同一个非泛型 `JsTapRegister`/`HookCommon` 路径；错误的 tap `TypeId` 在加载时被拒绝，而不是执行时 panic。
8. 保留并运行现有 `AdditionalTaps` 测试，确认通用 interceptor 没有回归。

### 生成代码与单态化检查

- 检查 `define_hook!` 展开结果，确认 `is_empty`、`has_js_taps`、JS register 调度和 call-mode 分支只是调用 `HookCommon`，没有为每个 hook 复制完整实现。
- 对比改造前后的 release 构建符号/LLVM lines（优先使用仓库已有二进制体积流程；本地工具可用时使用 `cargo llvm-lines` 或等价工具），确认没有按每个 `Hook::Tap` 生成 `JsTapRegister<T>`、泛型 storage 或泛型 executor 实例。
- 记录允许保留的单态化边界：`new_async::<Tap>`/`new_blocking::<Tap>` 的薄擦除构造和 `unerase_taps::<Tap>` 的薄恢复函数；若这两部分体积明显增长，再改成宏内最小转换，不扩大共享执行器的泛型范围。

### Binding 与 JS 回归验证

- 通过编译保证所有 `Register*` 都能转换为目标 hook 的 trait object，所有 adapter 安装点均已迁移。
- 验证克隆 `Register*`、加载到 `HookCommon` 的 erased owner、调用 `clear_cache()` 和 N-API count setter 操作的是同一个 `RegisterJsTapsInner`；释放 plugin/hook 后没有 owner cycle。
- 验证 JS register 返回空数组和非空数组时，`setRegisterJsTapCount` 分别写入 0 和 `jsTaps.length`，且下一次 Rust hook call 按本地计数选择快速路径。
- 验证首次 compilation 仍由非零 bootstrap count 驱动，并能批量初始化其他 register；否则初始为 0 的 hook 会永远没有机会执行 register。
- 验证 JS tap 在执行期间自移除/新增后，`#decorateJsTaps` 的批量更新不会留下过期的非零/零计数。
- 运行已有 stage 与 hook 用例，例如 `configCases/hooks/stage-make`、`configCases/hooks/stage-compilation`、`hookCases/compilation#processAssets`，确认 JS tap 仍执行且顺序不变。
- 如需要新增集成覆盖，只在现有 `tests/rspack-test` runner 下增加 case，不创建新的顶层 `test.js` runner；case 应验证“无 JS tap 不影响构建”和“有 JS tap 仍被调用”，性能快速路径本身由 Rust mock 的调用计数断言。

### 建议验证命令

```bash
cargo test -p rspack_macros_test --test hook
pnpm run build:binding:dev
cd tests/rspack-test && pnpm run test -t "configCases/hooks/stage-make"
cd tests/rspack-test && pnpm run test -t "configCases/hooks/stage-compilation"
pnpm run test:rs
pnpm run lint:rs
cargo lint
```

按项目约定，不运行会在 sandbox 中卡住的 storage 和 native watcher 相关测试。

## 实施顺序

1. 在 `rspack_hook` 定义非泛型 `JsTapRegister`、erased tap payload、共享计数接口和 call-mode 枚举。
2. 扩展非泛型 `HookCommon`，把 JS register 槽位、类型校验、空状态、路径选择和 erased register 调度全部收进去。
3. 精简 `define_hook!`：只生成参数签名、强类型 Rust tap 存储、对 `HookCommon` 的薄委托，以及各 hook 类型不可共享的 `tap.run(...)`/返回传播逻辑。
4. 在 `rspack_macros_test` 用 mock register 固化空状态、动态状态、类型校验和各 hook 类型语义，并确认 Rust-only 路径不进入类型擦除逻辑。
5. 改造 binding 的 `RegisterJsTapsInner` 与 `define_register!`：用单层 `Arc<RegisterJsTapsInner>` 统一拥有 register、cache、count 和 bootstrap 状态，并从 `Interceptor` 实现切换为非泛型 `JsTapRegister` 构造。
6. 增加 `setRegisterJsTapCount`，再修改两个 JS register factory，在返回 `JsTap[]` 前同步上报数组长度。
7. 删除全局 `NonSkippableRegisters` 状态，让现有 `setNonSkippableRegisters` 批量入口直接更新各 register inner 的 count，并验证首次 bootstrap 和 tap 动态变化时的 0/非 0 更新。
8. 批量迁移 `JsHooksAdapterPlugin` 的全部安装点，并用搜索确认不再有 JS register 走 `.intercept(...)`。
9. 编译 binding，运行宏测试和现有 JS hook/stage 回归用例。
10. 检查宏展开和 release 单态化/体积变化，确认共享实现没有以泛型 helper 的形式重新膨胀。
11. 运行 Rust 测试与 lint，最后检查 diff 中没有改变 JS register 的缓存清理，并确认 `is_empty()` 路径没有任何 JS/N-API 调用。

## 实施结果

已按上述设计完成实现：

- `JsTapRegister`、`HookCommon` 和 `HookCallMode` 已集中在 `rspack_hook`；空状态只读取 Rust 本地 count，register 调度与 tracing 均位于空路径之后。
- 52 个 binding register 全部改为共享单层 `Arc<RegisterJsTapsInner>`，inner 直接内嵌 cache、`AtomicUsize` 和 `always_active`；旧 `NonSkippableRegisters` 全局锁已删除。
- 52 个 adapter 安装点全部从 `.intercept(...)` 迁移到 `.load_js_tap_register(...)`，普通 interceptor 保持原 API 和执行顺序。
- JS hook 与 HookMap register 都会把最终 `JsTap[]` 长度同步到新的 `setRegisterJsTapCount`；批量 bootstrap API 保留并直接写同一批 inner count。
- 宏专项测试覆盖空 register、动态 count、Rust-only 快路径、同步 register、错误 tap 类型、stage 合并，以及 Series/Sync/Bail/Waterfall 空返回；现有 interceptor 测试继续通过。仓库当前没有使用 `Parallel` hook，专项测试 crate 也没有宏展开所需的 `futures_concurrency` 依赖，因此没有为测试单独引入新的运行时依赖；Parallel 的空分支与 Series 一样生成 `Ok(())`。

已完成验证：

```text
pnpm run build:cli:dev
cargo test -p rspack_macros_test --test hook
cargo clippy -p rspack_hook -p rspack_macros -p rspack_binding_api -p rspack_macros_test --tests -- -D warnings
pnpm run lint:rs
pnpm run lint:js
pnpm run test -t "Hook"
pnpm run test -t "compilerCases/hooks-closure"
pnpm run test -t "configCases/hooks/stage-make"
pnpm run test -t "configCases/hooks/stage-compilation"
pnpm run test -t "hookCases/compilation#processAssets"
```

本地未安装 `cargo llvm-lines`/`cargo expand`，因此没有生成额外的 LLVM lines 数值对比；代码检查确认共享 storage、状态机和 erased 调度均为非泛型具体类型，只在 `new_async`/`new_blocking` 与 `unerase_js_taps` 保留计划允许的薄泛型边界。
