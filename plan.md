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
       ↓ Arc 共享同一个 inner
JsTapRegister
  trait 只暴露 is_empty()
       ↓ 仅 RegisterJsTapsInner 实现
Hook
  泛型 load_js_tap_register<RegisterJsTapsInner>
  ├─ Arc 擦除为非泛型 JsTapRegister 状态字段
  └─ Arc clone 放进原 interceptors 字段
       ↓
hook.call
  empty -> tracing/register 前直接返回
  rust only -> 直接执行 Rust taps
  additional -> 执行原 interceptors，按 stage 合并 taps
```

## 1. `rspack_hook`：轻量 `JsTapRegister` trait

文件：`crates/rspack_hook/src/lib.rs`

- `JsTapRegister` 是只包含 `is_empty()` 的非泛型 trait，不保存 executor、owner 或 tap payload。
- 为 `Arc<T>` 复用 `Interceptor<H>` 调用；生产代码中的 `T` 只有 `RegisterJsTapsInner`。
- 删除 erased owner/taps、闭包、`Any`、downcast 和 async/blocking register function 等第二套执行抽象。
- `HookCommon` 只保存所有 hook 共享的 metadata、Rust tap stages 和普通 interceptor 数量。
- `HookCommon::call_mode(has_js_taps)` 根据本地布尔状态返回 `Empty`、`RustTaps` 或 `AdditionalTaps`。

## 2. `define_hook!`：泛型加载，原 interceptor 路径执行

文件：`crates/rspack_macros/src/hook.rs`

- 每个 hook 增加 `js_tap_register: Option<Arc<dyn JsTapRegister>>`，只用于保存 inner 所有权并查询状态。
- `load_js_tap_register<R>` 要求 `R: JsTapRegister + Interceptor<Self>`；生产侧唯一 concrete `R` 是 `RegisterJsTapsInner`。
- 加载时 clone 同一个 inner `Arc` 到现有 `interceptors`，但不增加普通 interceptor count；另一个 Arc handle 擦除为非泛型状态字段，不增加 inner allocation。
- `has_js_taps()` 只调用 Rust trait 的 `is_empty()`。
- `call` 在 tracing span 和 interceptor/register 调用前计算 call mode；空 hook 直接返回对应空值。
- additional taps 完全走现有 interceptor 循环和 stage merge，不生成 erased JS taps 的加载/还原代码。

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

## 4. `define_register!`：复用原 interceptor

文件：`crates/rspack_binding_api/src/plugins/interceptor.rs`

- 只为 `RegisterJsTapsInner` 实现一次 `JsTapRegister`。
- 宏为同一个 `RegisterJsTapsInner` 生成各具体 Hook 必需的 `Interceptor<具体 Hook>` 实现。
- interceptor 首先读取 `tap_count`；0 时返回空 taps，否则调用已有 register/cache 逻辑并转换具体 tap。
- adapter 调用点直接把各 `Register*` 内的 `Arc<RegisterJsTapsInner>` 传给 hook；删除 `into_js_tap_register`。
- 不生成新的 async/blocking executor、owner/downcast、whole-vector downcast或逐 tap downcast。

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
  - 同步 hook 也能通过同一泛型加载接口运行 register interceptor。
- 运行 `cargo test -p rspack_macros_test --test hook`。
- 运行 binding/macros test crate 的 clippy。
- 运行 JS formatter 检查。
- 使用 CI profile 构建并 strip native binding，与 PR merge-base 精确比较二进制大小。

## 当前进度

- [x] JS 侧向 Rust 同步 tap 状态。
- [x] `RegisterJsTapsInner` 改为单层 `Arc`，内嵌 cache 和 count。
- [x] 删除 `NonSkippableRegisters`。
- [x] hook 空路径在 tracing/register 前返回。
- [x] `JsTapRegister` 收敛为仅含本地状态检查的 trait，且仅由 `RegisterJsTapsInner` 实现。
- [x] `define_register!` 复用原 `Interceptor` 执行路径。
- [x] 完成最终测试和 CI profile 体积比较（stripped native binding `+20,472 bytes`，低于 50 KiB 阈值）。
- [x] 更新 draft PR。
