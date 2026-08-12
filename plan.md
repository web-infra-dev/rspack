# JS hook register 感知真实 tap 的执行计划

## 目标

JS hook register 不再作为普通 interceptor 计入 hook 的非空状态。JS 侧主动把真实 tap 数量同步到 Rust；hook 的空路径只读取 Rust 本地状态，不调用 JS。register 真正执行时继续复用现有 `Interceptor<Hook>`，不增加第二套执行和 tap 类型擦除逻辑。

## 执行流

```text
JS hook 注册/移除 tap
  读取 lite-tapable isUsed()/tap 数量
       ↓ 一次 JS -> Rust 同步
RegisterJsTapsInner
  AtomicUsize 保存 tap_count
       ↓ Register* 方法隐藏 Arc clone
JsTapRegister
  非泛型 trait 只暴露 is_empty
       ↓ Arc<dyn JsTapRegister> 存入非泛型 HookCommon
load_js_tap_register<R>
  在具体 Hook 边界要求 R: Interceptor<Self>
  同一个 Arc<R> 延迟转换为当前 Hook 的 interceptor
       ↓
Hook.call
  通过原 interceptors 路径调用 register
       ↓
  empty -> tracing/register 前直接返回
  rust only -> 直接执行 Rust taps
  additional -> 执行普通 interceptors 和 JS interceptor，按 stage 合并 taps
```

## 1. `rspack_hook`：非泛型 `JsTapRegister` trait

文件：`crates/rspack_hook/src/lib.rs`

- `JsTapRegister` 是 object-safe 的非泛型 trait，只暴露本地 `is_empty()`。
- 非泛型 `HookCommon` 保存 `Arc<dyn JsTapRegister>`，统一管理重复加载、`has_js_taps()`、`is_empty()` 和 call mode。
- 不使用闭包、`Any`、downcast 或手写函数指针表。
- `HookCommon::call_mode()` 根据 Rust taps、普通 interceptors 和本地 JS tap 状态返回 `Empty`、`RustTaps` 或 `AdditionalTaps`。

## 2. `define_hook!`：在具体 Hook 边界延迟转换

文件：`crates/rspack_macros/src/hook.rs`

- 生成的 hook 不再增加独立的 JS register 字段；非泛型状态 Arc 由 `HookCommon` 保存。
- `load_js_tap_register<R>` 在实际加载到具体 Hook 时要求 `R: JsTapRegister + Interceptor<Self>`。
- 同一个 `Arc<R>` 一份擦除为非泛型状态 trait object，另一份在该 Hook 边界转为原 `Interceptor<Self>` trait object；不增加 inner allocation。
- additional 路径完全复用原 interceptor 循环。
- `has_js_taps()` 只读取 Rust 本地状态。
- `call` 在 tracing span 和 interceptor/register 调用前计算 call mode；空 hook 直接返回对应空值。
- JS taps 仍作为当前 hook 的具体 tap 返回，与普通 interceptor taps 走同一个 stage merge。

## 3. `RegisterJsTapsInner`：单层共享状态

文件：`crates/rspack_binding_api/src/plugins/interceptor.rs`

- 所有 `Register*` 使用 `Arc<RegisterJsTapsInner>`。
- inner 直接内嵌：
  - compiler-scoped register callback；
  - register cache；
  - `AtomicUsize tap_count`；
  - `always_active` bootstrap 标记。
- cache 直接保存 `RwLock<Option<...>>`，不增加第二层 `Arc`。
- `set_tap_count` 是所有 JS 状态更新的唯一入口；`always_active` register 至少保持为 1。
- 删除全局 `NonSkippableRegisters` 和独立的 `Arc<AtomicUsize>`。

## 4. `define_register!`：状态非泛型，执行延迟绑定 Hook

文件：`crates/rspack_binding_api/src/plugins/interceptor.rs`

- `RegisterJsTapsInner` 只实现一次非泛型 `JsTapRegister` 状态 trait。
- 宏为 inner 生成各具体 Hook 必需的 `Interceptor<具体 Hook>`；转换只发生在 `load_js_tap_register<R>` 的具体 Hook 边界。
- interceptor 首先读取 `tap_count`；0 时返回空 taps，否则调用已有 register/cache 逻辑并转换具体 tap。
- `Register*::js_tap_register()` 只 clone 已有 inner Arc，隐藏 adapter 调用点的 `inner.clone()`，不创建新的 owner allocation。
- adapter 只把 `js_tap_register()` 的结果传给 hook，不接触内部 Arc。
- 不进行运行时 downcast，也不擦除或 downcast 每个 tap。

## 5. JS 状态同步

文件：

- `packages/rspack/src/Compiler.ts`
- `packages/rspack/src/lite-tapable/index.ts`
- `crates/node_binding/src/lib.rs`
- `crates/rspack_binding_api/src/plugins/js_hooks_plugin.rs`

- JS register 每次实际运行后，把 `jsTaps.length` 同步给 Rust。
- tap 新增、移除或 bootstrap 状态变化时，继续通过批量入口同步是否非空，避免下一次 register 执行前状态过期。
- `is_empty()`、`has_js_taps()` 不进行 JS/N-API 调用。

## 6. 验证

- Rust hook 测试覆盖：
  - JS tap 数量为 0 时，空 hook 跳过 register；
  - JS tap 从 0 变为非 0 后，hook 能立即感知；
  - Rust taps、普通 interceptors、JS taps 按 stage 合并；
  - 只有 Rust taps 且 JS register 为空时走快速路径；
  - 同步 hook 也能通过同一延迟转换接口运行 register interceptor。
- 运行 `cargo test -p rspack_macros_test --test hook`。
- 运行 binding/macros test crate 的 clippy。
- 运行 JS formatter 检查。
- 使用 CI profile 构建并 strip native binding，与 PR merge-base 精确比较二进制大小。

## 当前进度

- [x] JS 侧向 Rust 同步 tap 状态。
- [x] `RegisterJsTapsInner` 改为单层 `Arc`，内嵌 cache 和 count。
- [x] 删除 `NonSkippableRegisters`。
- [x] hook 空路径在 tracing/register 前返回。
- [x] `JsTapRegister` 和 `HookCommon` 保持非泛型，`HookCommon` 保存 `Arc<dyn JsTapRegister>`。
- [x] 只在具体 Hook 的加载边界把同一个 inner Arc 转为 `Interceptor<Self>`。
- [x] adapter 通过生成对象的方法隐藏 `inner.clone()`。
- [x] 当前延迟转换实现通过 hook 测试、`-D warnings` clippy 和完整 CI profile binding 构建。
- [x] 当前未提交实现 stripped native binding 为 `70,694,224` bytes：相对 merge-base `-28,680 bytes`，相对上一版 `-49,152 bytes`。
- [ ] 用户确认后再提交、推送并更新 draft PR。
