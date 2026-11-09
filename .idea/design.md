# JS 任务队列与订阅生命周期设计

本文记录当前执行设计。按最新讨论收敛为：**dispatcher 是进程级单例；每个 JS 线程有一个 subscription；subscription 创建时启动 Tokio 接收循环，Drop 时通过 CancellationToken 结束循环。**

队列放在独立 crate `rspack_js_tasks`，不放 core、rspack_parallel 或 binding。`rspack_loader_runner::LoaderContext::loader_yield` 直接构造任务、投递队列、等待结果并恢复 context；删除 LoaderRunnerPlugin::start_yielding、core NormalModuleLoaderStartYielding / loader_yield hook 和 binding scheduler hook。

```text
NormalModule::build
  crates/rspack_core/src/normal_module.rs
       ↓
run_loaders → LoaderContext::loader_yield
  crates/rspack_loader_runner/src/{runner,javascript}.rs
  读取 JsLoaderRuntime，跳过无 pitch 的链，切分 main/parallel 连续段
  take_owned，原 Box<LoaderContext> 移入任务
       ↓
js_task_dispatcher().dispatch_main / dispatch_worker
  crates/rspack_js_tasks/src/{dispatcher,task}.rs
  私有 async-channel；检查 main ready/cancel 或 worker 是否可用
       ↓
JsTaskSubscription / TSFN
  crates/rspack_binding_api/src/js_tasks/{subscription,tsfn}.rs
       ↓
JsLoaderTask.takeContext → 统一 runLoaders → complete / fail
  binding 转换 JS context，管理 Module scope、worker permit、对象桥 handle
       ↓
loader runner 恢复原 Box<LoaderContext>，继续 pitch / normal / builtin
```

依赖方向为 binding → core → loader runner → rspack_js_tasks，binding 也直接订阅 rspack_js_tasks。新 crate 不依赖 core、loader runner 或 N-API。任务中的 context 在队列内部保存为 `Box<dyn Any + Send>`，由 binding 恢复为 `LoaderContext<RunnerContext>`，返回后 runner 恢复自己的 `LoaderContext<Context>`；只擦除类型，不复制、序列化或重新构造 native state。公共 channel 类型和 close 操作均不暴露。

每个 compiler 的 `JsLoaderRuntime` 保存 main object handle、parallel references 和无 pitch 缓存。binding 的 JsLoader 通过 compiler ID 获取该元数据，返回 Arc；不执行异步 hook，不把当前 compiler 的 handle 持久化进可缓存的 JsLoader。thisCompilation 复用原有 hook 登记 Weak 路由；JsCompiler finalize 删除路由，包括 unsafeFastDrop。

队列另有 main 的 `MainTaskRegistration`：由主订阅强持有，dispatcher 只持 Weak，Tokio 接收循环不持有。它保存 ready/cancel 和入队许可；stop 与 enqueue 串行，防止订阅排空后又收到无消费者任务。worker 可用 ID 集合与入队/末个 worker 退出排空也在同一锁下处理。这些仅是原有消费者检查的下沉，不增加 dispatcher generation、task envelope 或 compiler 强引用。

## 对象与命名

| 名称                                        | 职责与生命周期                                                                           |
| ------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `JsTaskDispatcher`                          | 独立 crate 中的进程级单例；封装私有 main/worker channel、main Weak 注册和 worker 可用 ID |
| `JS_TASK_DISPATCHER`                        | 新 crate 私有的 `LazyLock<JsTaskDispatcher>`，通过 js_task_dispatcher() 访问             |
| `JsTaskSubscription`                        | 一个 JS 线程的 TSFN、receiver 与取消信号；构造时启动一次接收循环                         |
| `MAIN_TASK_SUBSCRIPTION`                    | 主线程订阅的全局 Weak 入口；各 JsCompiler 持有实例的 Arc，实例拥有 worker 订阅表         |
| `worker_subscriptions`                      | 主线程 subscription 上的 DashMap，按 Node thread ID 强引用 worker 订阅                   |
| `get_js_task_subscription(is_worker: bool)` | Rust 统一导出；false 获取主线程订阅，true 获取并注册当前 worker 订阅                     |
| `JsTask`                                    | `Loader`、`WorkerFunction` 等任务枚举                                                    |

“只创建一次”应用于 dispatcher。主线程 subscription 沿用最初的需求：最后一个 JsCompiler 释放后可销毁，之后创建 compiler 时可重新建立。若改成 `static Arc<JsTaskSubscription>`，主线程订阅就会常驻，不能再期待 compiler GC 触发它的 Drop；这两种语义需要明确区分。

删除上一版的 `JsTaskDispatcherId`、`SubscriptionKey`、dispatcher generation、dispatcher lifecycle、subscription 指回 dispatcher 的 Weak，以及通用的 `JsTaskEnvelope`。当前只有一个固定 dispatcher 和一个 main env，没有必要预置这些协议。已有 loader 的 task ID、完成 cell 和在途任务记录仍按实际用途保留在任务实现中。

## 最小结构

以下为订阅所有权示意，省略队列封装、MainTaskRegistration 和初始化错误处理；dispatcher 位于独立 crate，subscription 位于 binding：

```rust
static JS_TASK_DISPATCHER: LazyLock<JsTaskDispatcher> =
  LazyLock::new(JsTaskDispatcher::new);

static MAIN_TASK_SUBSCRIPTION: Mutex<Weak<JsTaskSubscription>> =
  Mutex::new(Weak::new());

struct JsTaskDispatcher {
  main_sender: Sender<JsTask>,
  main_receiver: Receiver<JsTask>,
  worker_sender: Sender<JsTask>,
  worker_receiver: Receiver<JsTask>,
}

struct JsTaskSubscription {
  tsfn: JsTaskTsfn,
  receiver: Receiver<JsTask>,
  cancel: CancellationToken,
  // 仅 main subscription 使用；worker subscription 中保持为空。
  worker_subscriptions: DashMap<NodeThreadId, Arc<JsTaskSubscription>>,
}

struct JsCompiler {
  task_subscription: Arc<JsTaskSubscription>, // 主线程共享实例
  // 原有字段
}
```

两个 channel 只创建一次。main subscription clone `main_receiver`；每个 worker subscription clone 同一个 `worker_receiver`。接收循环不需要知道 compiler、任务类型或 dispatcher 身份。只有 main subscription 管理 worker 订阅，worker subscription 的表为空；暂不为此引入额外角色类型。

如果 `tsfn` 和 `receiver` 构造后只由接收循环使用，还可以直接 move 进 Tokio task，让 worker subscription 的有效状态只保留 `cancel`，main subscription 另外保留 worker 表。保留这两个字段仅在其他方法确实需要访问时有价值；无论哪种形式，都不让后台循环拥有 subscription 本身。

## Rust 导出与 JsCompiler 参数

Rust 统一导出 `get_js_task_subscription(is_worker: bool)`，JS 侧按现有 N-API 命名约定调用 `getJsTaskSubscription(isWorker)`。语义上的返回值为 `Arc<JsTaskSubscription>`，通过 binding 的不透明持有对象跨 JS 边界传递，不暴露内部 TSFN、receiver 或 token，也不序列化 Arc。

```rust
// 接口语义示意；实际 N-API 签名包含当前 Env 和不透明返回值转换。
pub fn get_js_task_subscription(
  is_worker: bool,
) -> napi::Result<Arc<JsTaskSubscription>>;

impl JsCompiler {
  // 示意新增参数；其余构造参数保持现有定义。
  fn new(/* 原有参数 */, task_subscription: Arc<JsTaskSubscription>) -> Self;
}
```

N-API 边界具体使用可取出的 holder，`Env` 由 binding 注入，JS 调用者只传 `isWorker`：

```rust
#[napi(ts_return_type = "ExternalObject<'JsTaskSubscription'>")]
pub fn get_js_task_subscription(
  env: Env,
  is_worker: bool,
) -> napi::Result<External<Option<Arc<JsTaskSubscription>>>>;
```

主线程返回 `External::new(Some(main_subscription))`；`JsCompiler` 构造函数接收 holder，校验其角色和所属 env 后通过 `take()` 取得 Arc，保存在自己的正常字段中。holder 为空或已被消费时返回明确错误。多个 compiler 各自调用一次获取接口，复用底层订阅；不能把同一个已消费的 holder 再传给另一个 compiler。

- `is_worker == false`：在主线程 upgrade `MAIN_TASK_SUBSCRIPTION`，存在则 clone Arc；不存在则使用 main receiver 和当前 env 的 TSFN 创建订阅、启动接收循环，再保存 Weak。返回主订阅 Arc，作为 binding `JsCompiler` 的构造参数。
- `is_worker == true`：在当前 worker env 中取得所属 main subscription，按当前 Node thread ID 查它的 `worker_subscriptions`；存在则复用，不存在则用共享 worker receiver 和当前 env 的 TSFN 创建并注册。表持有 Arc，返回 `External::new(None)`，不把强引用交给 JS；worker 环境和 cleanup 仍只长期保存 Weak。
- `is_worker` 用于选择订阅角色，不允许把 worker subscription 传给主线程 `JsCompiler`。初始化时核对调用环境及订阅归属；worker 在 main 尚不存在时返回错误。

`JsCompiler` 直接保存传入的 Arc，不在构造函数内再次获取或创建订阅。这样订阅的创建先于 compiler，也便于多个 compiler 复用同一个实例。改动只涉及 binding `JsCompiler`，不向 `rspack_core::Compiler::new` 增加参数。

```text
packages/rspack/src/Compiler.ts
  创建 native compiler 前调用 getJsTaskSubscription(false)
       ↓
crates/rspack_binding_api/src/js_tasks/mod.rs
  get_js_task_subscription(false)
  upgrade / 创建 main subscription，返回 Arc 的不透明持有对象
       ↓
new binding.JsCompiler(..., taskSubscription)
  crates/rspack_binding_api/src/lib.rs
  取出 Arc，保存在 JsCompiler.task_subscription
       ↓
compiler 执行 / 释放
  共享订阅执行任务；最后一个持有者释放时取消整组订阅
```

JS 临时持有对象应在构造参数转换时把 Arc 移交给 `JsCompiler`，避免除了 compiler 字段之外又留下一个依赖 JS GC 才释放的强引用。构造失败时释放已取出的 Arc。不要把返回值保存在 JS 模块级变量中。worker 分支的返回 holder 为空，由 main 的表继续拥有订阅。

TSFN 仍在所属 env 创建，并使用不捕获 compiler 的统一任务入口；loader 任务执行时按 CompilerId 找 compiler 再显式传入。该导出及参数转换需要同步更新 binding 导出和生成类型，TypeScript 侧仅把订阅视为不透明参数。

## 强弱引用与释放条件

```text
进程级 JS_TASK_DISPATCHER
  main channel + worker channel
  不持有 subscription

MAIN_TASK_SUBSCRIPTION -- Weak --> main subscription
                                     ^
                                     | Arc
                              每个 binding JsCompiler

main subscription
  worker_subscriptions[thread_id] -- Arc --> worker subscription

worker 的 env cleanup / 本地缓存 -- Weak --> 所属 main / worker subscription

subscription
  cancel token
       |
       +-- clone --> Tokio 接收循环
                     receiver + TSFN + cancel token
                     不持有 Arc<subscription>
```

`worker_subscriptions` 放在 **MAIN_TASK_SUBSCRIPTION 指向的主线程实例上**，不放在永久 static dispatcher 上。全局变量本身仍是 `Mutex<Weak<JsTaskSubscription>>`；销毁的是它指向的实例和实例中的表，static Weak 入口保留以便后续重建。

这里推荐表存 **Arc**：main subscription 是 worker 订阅的所有者。最后一个 compiler 和临时调用方释放 main Arc 后，main Drop 取消自身及表中的 worker 订阅，随后释放整张表的强引用。worker 没有其他长期强引用时进入 Drop，接收循环通过各自 token 退出。若表只存 Weak，表的销毁只会删除索引，不能保证 worker 订阅随之释放。

具体规则：

- main：各 `JsCompiler` 持有 Arc，全局只存 Weak。关闭某一个 compiler 不取消其他 compiler 使用的订阅。最后一个真正持有者销毁后触发 Drop。
- worker：main 的表持有长期 Arc；worker 环境的缓存和 cleanup 回调只存 Weak。worker subscription 不反向强引用 main，TSFN/后台循环也不捕获 main Arc，避免阻止 main Drop 或形成引用环。
- `get_js_task_subscription(true)` 短暂 upgrade 所属 main，按 thread ID 查找或创建 worker 订阅并插入表，随后释放 main Arc。查表结果的 Arc 仅供短暂调用使用，不由 JS 包装对象、环境状态或永久接收循环长期持有。没有存活 main 时注册失败，不在 worker env 中创建 main subscription。
- worker env 提前退出时，cleanup upgrade 它保存的 worker Weak 并 cancel，再从它所属 main 的表中移除对应项。cleanup 捕获所属 main 的 Weak，不查询可能已经重建的全局实例；移除时核对 Arc 指针相等，避免删除同 thread ID 下的替代实例。env teardown 必须主动取消，不能等待临时 Arc 全部消失。
- main Drop 也显式取消表中所有 worker：即使存在短暂的 worker Arc，循环仍会停止。随后表字段析构释放所有 Arc，worker 对象在最后一个临时引用释放后销毁。
- `unsafe_fast_drop` 会跳过 core compiler 的析构，所以主订阅 Arc 放在 binding `JsCompiler` 的正常字段中，不放进 core compiler/plugin。
- 独立 WorkerFunction 的外部调用方在调用期间也持有 main subscription，保活其拥有的 worker 订阅表，即使该函数不需要 main RPC。任务队列本身不充当 subscription 的长期所有者。

释放顺序：

```text
最后一个 binding JsCompiler / 外部调用方释放 main Arc
       ↓
main subscription::drop
  cancel main + 表内所有 worker token
       ↓
worker_subscriptions 字段析构
  释放表内 Arc，worker subscription 随最后一个引用释放而 Drop
       ↓
各 Tokio 接收循环的 select 被唤醒
  退出并释放 receiver / TSFN
       ↓
MAIN_TASK_SUBSCRIPTION 保留失效 Weak
  后续 compiler 创建新的 main subscription 和空 worker 表
```

不再引入一个额外的 `JsTaskClient` 类型，直到有实际调用方需要这层封装。

## 构造即启动，Drop 即取消

`new` 在 TSFN 所属的 JS 环境创建订阅，然后用 receiver、TSFN、token 的 clone 启动 Tokio task。task 不能捕获 `Arc<Self>`：否则 task 保活 subscription，Drop 永远不会执行，也就永远不会发出取消信号。

```rust
impl JsTaskSubscription {
  fn new(tsfn: JsTaskTsfn, receiver: Receiver<JsTask>) -> Self {
    let cancel = CancellationToken::new();

    // 使用 binding 的 Tokio runtime 入口，不要求当前 JS 线程处在 Tokio 上下文中。
    rspack_napi::runtime::spawn(receive_tasks(
      tsfn.clone(),
      receiver.clone(),
      cancel.clone(),
    ));

    Self {
      tsfn,
      receiver,
      cancel,
      worker_subscriptions: DashMap::new(),
    }
  }
}

impl Drop for JsTaskSubscription {
  fn drop(&mut self) {
    self.cancel.cancel();
    // worker 的表为空；main 在释放表内 Arc 前主动停止所有 worker 循环。
    for worker in self.worker_subscriptions.iter() {
      worker.value().cancel.cancel();
    }
  }
}
```

构造前通过现有 `rspack_napi::runtime::ensure_runtime(env)` 确保运行时和环境 cleanup 已注册。构造失败应在 spawn 前返回；同一线程的重复 `get_js_task_subscription(is_worker)` 调用复用已有对象，不重复启动循环。

接收循环示意，`post_task` 是需要实现的拥有任务所有权的投递封装：

```rust
async fn receive_tasks(
  tsfn: JsTaskTsfn,
  receiver: Receiver<JsTask>,
  cancel: CancellationToken,
) {
  loop {
    let task = tokio::select! {
      biased;
      _ = cancel.cancelled() => break,
      received = receiver.recv() => match received {
        Ok(task) => task,
        Err(_) => break,
      },
    };

    // recv 和取消可能同时发生；取出来的任务不能直接丢弃。
    if cancel.is_cancelled() {
      task.fail_before_start(SubscriptionClosed);
      break;
    }

    // 非阻塞安排 JS 回调；不在这里等待 loader/function 执行结束。
    // 投递失败必须能取回 owned task，由统一完成协议归还 native state。
    if let Err(task) = post_task(&tsfn, task, cancel.clone()) {
      task.fail_before_start(SubscriptionClosed);
      break;
    }
  }
  // future 结束，释放它持有的 receiver、TSFN 和 token。
}
```

该示意只展示订阅和取消机制，具体任务类型提供自己的失败完成方法。Token 的 clone 共享取消状态；`cancel()` 唤醒 `cancelled()` 等待者，参见 [CancellationToken 文档](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)。

需要保留三个准确的语义：

1. subscription Drop **发出取消请求**；后台 future 下一次获得执行机会才退出并释放它的资源，并非 Drop 返回时全部资源已同步释放。不能在 Drop 中阻塞等待它。
2. 丢弃普通 token 自身不是取消协议，必须在 subscription Drop 中调用 `cancel()`，或明确使用 cancellation drop guard。
3. 已经交给 TSFN/JS 的任务不再归接收循环管理。投递句柄携带取消信号，在 JS 开始访问 context 前再检查；已开始执行的任务则由自己的完成/失败协议负责，不能因 subscription Drop 就提前释放 JS 正在使用的 native state。

env cleanup 需要确认循环退出时，可使用退出通知或接收循环的 JoinHandle 进行协调；这是 cleanup 的资源管理需求，不需要因此给每个任务增加代次协议。要配合当前 binding runtime 的 cleanup 顺序，确保退出时不会再向失效的 env 投递。

## 任务内容与执行流

任务本身使用枚举即可，不重复在每项里放 dispatcher ID 或目标队列字段。调用 `dispatch_main(task)` / `dispatch_worker(task)` 时已选定队列。

```rust
// rspack_js_tasks，不依赖 loader runner 或 core。
enum JsTask {
  Loader(LoaderTask),
  // WorkerFunction 可在有调用方时增加自己的 payload / completion。
}

struct LoaderTask {
  context: Option<Box<dyn Any + Send>>,
  main_object_handle: u32,
  start: u32,
  end: u32,
  result: Option<oneshot::Sender<LoaderTaskResult>>,
}

// take_owned 得到的同一个 Box<LoaderContext>，这里只做类型擦除。
type LoaderTaskResult = (Box<dyn Any + Send>, Vec<String>, Result<()>);
```

Loader 沿用连续链执行；名称可从 context 的 loader items 得到，不把单个 `loader_name` 强设为调度必需字段。compiler ID 同样从 context 取得。未来 WorkerFunction 的名称可在 worker 注册表中解析，options/args 使用可跨线程的 Rust 数据或对象 handle，不包含原始 main-env N-API value；它定义自己的返回协议。

```text
loader runner 的 loader_yield / WorkerFunction 调用方
  创建 JsTask，选择 main 或 worker 队列
       ↓
全局 JsTaskDispatcher
  async-channel 入队
       ↓
对应线程的 subscription 接收循环
  select { cancelled, receiver.recv() }
       ↓
该线程自己的 TSFN
  只安排 JS 执行，不捕获 compiler
       ↓
Loader：用 CompilerId 找到高层 JS Compiler
        runLoaders(compiler, context)
WorkerFunction：按 function_name 查函数
                invoke(options, args)
       ↓
任务自身 complete / fail
       ↓
Loader 返回原 boxed native state；函数任务返回结果或错误
```

主线程 TSFN 继续遵循已确定的约束：移除 `runLoaders.bind(null, compiler)`；按 `CompilerId` 查找高层 `packages/rspack/src/Compiler.ts` 对象，再作为参数传入。现有 `COMPILER_REFERENCES` 指向 binding `JsCompiler`，且在 `run` 时登记；需要补上到高层 JS owner 的弱引用映射，不能把 binding wrapper 直接当成高层 Compiler。

compiler close 后撤销它自己的查找路由；共享 TSFN 和接收循环不关闭。共享 callback 不捕获 compiler，因此也不交给某个单独 compiler 的 `CompilerScopedTsFnManager` 管理。

## Loader context 的构造与对象访问

Rust 队列传递 `Box<LoaderContext<RunnerContext>>`，在实际消费任务的 JS 线程边界构造 `JsLoaderContext`，再由统一的 `runLoaders` 构造 loader 可见的 `loaderContext`。main 和 worker 使用同一套 context 与 Module binding；按最新约定，worker 对 loader 暴露受限的 Module 包装，内部持有当前任务的真实 native Module。compiler 和 compilation 保留现有主线程代理实现。

| 内容                                                                    | worker 中的来源与访问方式                                                                                            |
| ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| resource、content、sourceMap、loaderItems、loaderIndex、dependencies 等 | 从 Rust task 的 native context 转换，执行结果 merge 回同一个 boxed state                                             |
| `loaderContext._compiler`                                               | 沿用 `createWorkerCompiler` 和现有对象桥，调用通过 RPC 在主线程执行                                                  |
| `loaderContext._compilation`                                            | 沿用 compiler 的 `_lastCompilation` 代理，在主线程访问真实 compilation；同步调用复用 MessagePort + Atomics           |
| `loaderContext._module`                                                 | 使用受限的 worker Module 包装，内部访问当前线程从 Rust context 得到的真实 Module；不再替换成 main module 的 RPC 代理 |
| loader options、`additionalData`                                        | 继续使用 `MainObjectHandle(u32)` 与对象桥缓存；loader resolve 和加载缓存保持                                         |
| 主线程 loader hook 及自定义 context 扩展                                | 保留 prepare/hooks-only 阶段及扩展桥接，worker 执行阶段使用本线程构造的 context                                      |

loader 可见的包装保留既有受限 API；内部“和主线程一样的 module 对象”指同一套 `ModuleObject` / Module 类及其 API，指向当前任务对应的 Rust module。每个 env 创建自己的 JS wrapper，不跨 isolate 传递主线程的 JS 对象或 N-API 引用。复用 `module.rs` 的线程本地实例缓存，并在对应 worker env 中完成清理。

```text
Rust loader runner
  crates/rspack_loader_runner/src/javascript.rs
  take_owned，Box<LoaderContext> 放入 JsTask::Loader
       ↓
worker subscription / TSFN
  js_tasks/subscription.rs
  在当前 worker env 交付任务
       ↓
task.takeContext()
  plugins/js_loader/worker.rs → context.rs
  从 native state 构造 JsLoaderContext
  保留 ModuleObject::with_ptr 得到的 _module
       ↓
统一 runLoaders(compilerProxy, context, extensions)
  packages/rspack/src/loader-runner/index.ts
  _compiler / _compilation → 现有主线程 RPC
  _module → context._module，本线程的 native Module wrapper
       ↓
task.complete(result) / task.fail(error)
  merge 执行状态并返回原 boxed state
       ↓
Rust runner restore_owned，继续 mixed loader chain
```

具体修改点：

1. `crates/rspack_binding_api/src/plugins/js_loader/worker.rs`：删除 `take_context` 中的 `context.module = None`，保留 `context.rs::JsLoaderContext::from_native` 构造的 `ModuleObject`。main/worker 均在自己的 JS 边界完成 N-API 转换。
2. `packages/rspack/src/loader-runner/worker.ts`：删除保存 `nativeModule`、赋值 `context._module = main.module`、完成前恢复 `result._module` 的逻辑；把 Rust 返回的 context 保持原样交给 `runLoaders`，额外传入受限的 worker Module 包装，仅用于 loaderContext._module。
3. `packages/rspack/src/loader-runner/service.ts`：从 `WorkerLoaderContext` 和 `registerLoaderContext` 的桥接载荷移除专供 `_module` 替换使用的 `module` 字段、参数和 `remoteRoots.add(module)`。用 buildInfo 字段保留自定义值（含函数）及 assets 查询的对象桥；保留 compiler、compilation、文件系统、options 和 hook extensions 的桥接；主线程 hook 自身仍接收主线程的 module wrapper。
4. `packages/rspack/src/loader-runner/index.ts`：保留 `const module = context._module` 供 runner 内部访问；worker 的 `loaderContext._module` 指向受限包装，main 则直接使用 native Module；调整 `registerLoaderContext` 调用。extensions 合并后也必须保持 `_module` 指向本线程受限包装，不让主线程返回的扩展重新覆盖成代理。`emitFile` 等通过 `_module` 的操作使用同一套 Module binding。
5. `crates/rspack_binding_api/src/module.rs`：适配 Module 在 worker loader 执行窗口内的访问。当前 `with_ref` 会先调用 `with_compilation`，后者依赖线程本地 `COMPILER_REFERENCES`；`original_source` 也直接查询该表。worker 没有主线程的 compiler 引用，因此仅保留 `_module` 字段还不够。实现时将仅需当前 module 的操作改为使用任务提供的有效 native module；需要 compilation 信息的操作沿用主线程访问路径或取得所需的拥有所有权的数据，不把主线程 compiler 的 N-API 引用塞进 worker 的表，也不以裸指针绕过 compilation 并发访问约束。该适配留在 binding，compiler/compilation 的现有代理模型不变。

worker 包装公开原有的 type、identifier、matchResource、request、userRequest、rawRequest、buildInfo，并支持不依赖 compilation 图的 resource/context/layer、source map 标记、factoryMeta、readableIdentifier、nameForCondition、originalSource、size、libIdent 和 emitFile。blocks、dependencies、error 等需要 compilation 的 API 不暴露。

assets 保持现有可用语义：buildInfo（含 assets 查询）通过主线程对象桥访问，避免在 worker 读取 BindingCell 中已缓存的 main-env JS 引用；worker 的 emitFile 直接写当前任务的 native module，转换 source 和 assetInfo 后不调用 reflector.set_jsobject 缓存 worker-env 对象。main 后续访问 assetInfo 时在自己的 env 中创建 wrapper。无需修改 core BindingCell 的全局缓存模型。

| assets 操作                                          | 执行位置与对象归属                                                                                            |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `loaderContext.emitFile` / `_module.emitFile`        | worker 本地 Module binding；source、source map、assetInfo 转成 Rust 数据，写入当前 module 的 buildInfo.assets |
| `_module.buildInfo.assets` 查询                      | 已有主线程 buildInfo 对象桥；worker 得到 RPC 结果或缓存代理，不读取 main 的原始 N-API value                   |
| compilation 的 getAsset / emitAsset / updateAsset 等 | 沿用 compilation 代理，在主线程调用真实实现，不在 worker 新建 compilation                                     |
| 主线程后续读取 worker emit 的 assetInfo              | 在 main env 创建 JS wrapper；worker 传入对象的 JS 身份不跨 env 保留，支持的元数据值保留                       |

```text
worker loaderContext.emitFile(name, content, sourceMap, assetInfo)
  packages/rspack/src/loader-runner/index.ts
       ↓
当前 worker 的 native Module.emitFile
  crates/rspack_binding_api/src/module.rs
  先完成 source / assetInfo 转换，再借用 module 写 assets
  不把 worker JS object 写进 BindingCell reflector
       ↓
task.complete / fail
  revoke Module 访问窗口，归还原 Box<LoaderContext>
       ↓
Rust normal module build / compilation 收集 assets
       ↓
main compilation.getAsset / processAssets
  在 main env materialize source / assetInfo
```

转换顺序需要避免持有可变 module 借用时执行 JS getter 或对象桥 RPC。保留的 worker facade 在任务结束后不可继续调用 native module；主线程 hook 的 extensions 也不能替换这个 facade。验证覆盖 emit 后通过 buildInfo.assets 查询、processAssets 中读取内容与 assetInfo、factoryMeta/matchResource 修改，以及旧 facade 在下一任务中访问失败。

Module 的 native 访问窗口由当前 task 的执行与完成协议保证：JS loader 尚未完成时保持 native state 有效；完成或失败前不得 restore 并让 Rust 同时继续操作该 module。主线程 prepare/hooks-only 与 worker 执行按现有顺序交接，避免两边同时修改 module。worker 缓存中的 wrapper 不延长 native 指针的有效窗口；订阅退出、compiler close 和 rebuild 时按对应 env 清理缓存与失效访问路径。

## 简化后仍需处理的行为

### 并发、重入与任务归属

main 接收循环不能逐个 `await runLoaders` 后才收下一项：一个 loader 的 `importModule` 可能需要本队列中的后续任务。JS 回调启动异步执行后返回，结果由 task 独立报告。

worker 需要避免把共享队列全部预取到单个繁忙线程的 TSFN 队列。现有“一般单任务、等待 import 时可重入”的策略保留在 worker 消费/执行逻辑中；如果接收循环等待 worker 可用信号，该等待也要同时 select 取消。这里需要实际的接收节流，但不预先把它扩展成通用 subscription 的一组生命周期/permit 协议字段。

投递失败、JS 抛错、Promise reject、compiler 已失效、N-API 转换失败都要一次性结束 task。Loader 失败仍要把原 native context 返回 loader runner，不仅仅丢弃 oneshot sender。worker 崩溃时沿用在途任务回收机制；必须等确认 JS 已无法继续访问 native state 后才能归还，不能把“取消接收”当作“终止正在执行的 JS”。

### 全局 channel 不随 subscription 关闭

subscription Drop 取消本循环；main 还取消其拥有的 worker 循环并释放订阅表。各循环退出时释放自己的 receiver clone，不调用 `receiver.close()`。对共享 receiver 调用 close 会关闭整个 channel，影响其他 worker，而且关闭后已排队的消息仍可被取走，见 [async-channel Receiver 文档](https://docs.rs/async-channel/latest/async_channel/struct.Receiver.html#method.close)。

因为 static dispatcher 始终持有 receiver，所有订阅退出后 channel 仍然是 open。因而 **send 成功不代表存在消费者**：

- main loader 通过其正在运行的 compiler 保活主订阅；其他 main 调用也必须有明确的订阅所有者，不能仅向 static sender 盲发任务。
- worker 路径继续由现有 worker pool 的 ready/失败/退出通知决定是否可接收新任务；main 实例上的订阅表查找或数量不能代替 ready 状态。
- 最后一个可用消费者退出时，对受影响的排队任务和在途任务显式失败归还。检查消费者状态与入队/停止的边界要同步，不能出现“检查成功、消费者退出、成功入队后永久无人处理”的窗口。
- main 订阅销毁再创建时，旧循环即使已收到取消也可能尚未退出。新 main/worker 接收循环启动前要完成对应旧循环的退出交接及旧任务处理；单靠 Weak upgrade 失败不足以证明这一步已完成。可用独立退出通知串联启动，不需要 dispatcher generation 字段。

新旧接收循环的交接不在全局 Mutex/DashMap guard 内等待，也不能在 JS 主线程上同步 block_on。全局队列不会被 close，因此重建 subscription 时可复用同一队列。

### 环境边界与对象桥

每个 TSFN 在所属 JS 线程创建。DashMap 使用 Node `worker_threads.threadId`，不使用 Tokio 线程 ID。当前计划针对一个 main env；在另一个 Node worker 中创建独立 compiler 时，不能误用原 main TSFN，多 main env 支持另行设计，不提前给当前结构增加环境路由字段。

空闲 TSFN unref 只决定它是否保活事件循环，不会让 JS callback 变成弱引用，也不等于关闭。强制 env teardown 还需专门的 native 清理路径，见 [Node-API TSFN 生命周期文档](https://nodejs.org/api/n-api.html#asynchronous-thread-safe-function-calls)。

`additionalData`/options 继续使用 `MainObjectHandle(u32)`。当前 `bridge.rs` 中的 `RELEASE: OnceLock<Arc<TSFN>>` 属于环境资源，不能因 dispatcher 只初始化一次就让它永远绑定首个主订阅。主订阅重建时，要让新对象使用新 release 路由，旧对象只使用自己的有效路由；可以保留进程内单调分配、不复用的对象 ID，并使用所属订阅的弱路由，避免额外代次字段。

worker options cache、loader references、MessagePort、初始化 Promise 仍按实际 compiler/worker 生命周期清理；dispatcher 常驻不代表这些 JS 对象也常驻。main 订阅销毁会取消其拥有的 worker 接收循环，但不自动终止 OS worker；worker pool 的销毁由它自己的退出协议负责。若复用仍存活的 OS worker，新的 main subscription 创建后必须让各 worker 在自己的 env 中重新注册 TSFN/订阅并报告 ready，不能沿用已取消的订阅。MessagePort + Atomics 同步 RPC 的重入能力保持，不放到一个必须等待当前 loader 返回才能处理的循环后面。

## 执行计划

| 步骤                             | 文件/范围                                                                                                            | 完成条件                                                                                                                                                        |
| -------------------------------- | -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. 提取通用队列                  | 独立 `rspack_js_tasks/src/{lib,dispatcher,task}.rs`                                                                  | 一个 static dispatcher；两条 async-channel；`JsTask` enum；不引入 dispatcher ID/envelope/generation                                                             |
| 2. 实现订阅循环                  | `subscription.rs`、`crates/rspack_napi/src/runtime.rs` 的现有入口                                                    | 构造后自动 spawn；只捕获独立资源；Drop cancel；验证循环和 TSFN 的释放                                                                                           |
| 3. 导出订阅并注入 compiler       | `js_tasks/mod.rs`、binding `lib.rs`、`Compiler.ts`、worker 初始化/cleanup、binding 导出与生成类型                    | 导出 `get_js_task_subscription(is_worker)`；main 返回 Arc 经不透明参数移交 JsCompiler；全局 main Weak；main 表持有 worker Arc；worker env 只存 Weak；同线程复用 |
| 4. 移除 bound runner             | `Compiler.ts`、`JsLoaderRspackPlugin.ts`、`raw_builtins/mod.rs`、loader plugin `mod.rs`                              | TSFN 不捕获 compiler；通过 CompilerId 查高层 JS owner，再显式传参                                                                                               |
| 5. main 与 worker 统一入队       | `rspack_loader_runner/src/javascript.rs`；独立任务队列；binding `worker.rs`                                          | runner 直接投递两类任务；删除所有 yield hook 转发；main 可异步并发；worker import 重入及节流保持                                                                |
| 6. 统一 loader context 与 Module | loader plugin `context.rs`、`worker.rs`，binding `module.rs`；JS loader-runner `worker.ts`、`service.ts`、`index.ts` | worker context 保留 native `_module`，loader 使用受限包装；移除专用 module 代理替换；适配 Module 的线程本地依赖；compiler/compilation 沿用主线程 RPC            |
| 7. 处理退出及对象路由            | worker pool、task 完成表、`bridge.rs`、`service.ts`                                                                  | 无消费者时失败归还；订阅交接无遗漏；旧 release 回调不影响新对象；close 一个 compiler 不影响其他 compiler                                                        |
| 8. 构建与回归                    | binding/JS 构建、现有测试 runner、生成类型                                                                           | 满足下方验证矩阵                                                                                                                                                |

队列及原生完成协议放在 `crates/rspack_js_tasks/`；binding `js_tasks/` 只保留 subscription、capacity 和 tsfn；JS context merge 位于 `plugins/js_loader/result.rs`。runner 自己调用 `take_owned/restore_owned`，不修改 `rspack_core::Compiler::new` 的公共构造参数，也不同时搬迁所有 compilation RPC。

具体 worker pool 的迁移与此订阅模型解耦。落实前文直接使用 worker_threads 的目标时，保持默认 `max(CPU - 2, 1)` 和 make 等待 ready 的约定。WASI 另核对其环境/内存边界，不能认为 native static 自动跨 WASM instance 共享。

## 验证矩阵

先构建对应 binding/JS 产物，再复用现有 configCases/compilerCases 测试 runner。新增 Rust 行为测试如确有需要放专用测试 crate，不新增 crate-local 测试；按项目约定跳过 storage 和 native watcher 测试。

| 场景                                                 | 必须验证                                                                                                                                                                     |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 同线程多次获取订阅                                   | `get_js_task_subscription(false/true)` 在对应线程复用一个实例、一个基础接收循环                                                                                              |
| main 返回值作为 JsCompiler 参数                      | 构造函数持有所传 Arc，不再次取订阅；临时 opaque holder 移交后无多余强引用；构造失败释放 Arc                                                                                  |
| worker 注册与参数校验                                | main 表拥有注册结果；worker 不长期保留返回值 Arc；缺少 main 或向 JsCompiler 传 worker 订阅时明确失败                                                                         |
| 接收循环永久等待，释放最后一个 subscription Arc      | Drop 可执行；cancel 唤醒 select；future、receiver clone、TSFN 最终释放                                                                                                       |
| 循环在等待 worker 可用时释放订阅                     | 同样可取消，不仅 recv 路径能退出                                                                                                                                             |
| cancel 与 recv/TSFN 投递并发                         | 已取出的 task 不丢失，native state 恰好返回一次                                                                                                                              |
| 两个 compiler 共用 main subscription                 | TSFN 按 CompilerId 取得正确 compiler；关闭/回收 A 不影响 B                                                                                                                   |
| 最后一个 compiler GC，再创建新 compiler              | dispatcher/channel 不重建；旧 main 和 worker 表释放、循环退出；新 main 接管，worker 重新注册                                                                                 |
| worker env 退出                                      | cleanup 取消并从所属 main 表移除 Arc；不误删替代实例；其他 worker 的 channel 仍可用                                                                                          |
| 最后一个 compiler GC，worker env 仍存活              | main 表和 worker 订阅可释放；临时 worker Arc 不妨碍取消；后台循环不保活 main                                                                                                 |
| 最后一个 worker 退出或启动失败                       | 排队/在途任务结束，无悬挂的 oneshot                                                                                                                                          |
| main/单 worker 递归 importModule                     | 不被串行接收或错误限流阻塞                                                                                                                                                   |
| unsafeFastDrop、构造失败、env 强制退出               | 无 subscription 强引用泄漏，无失效 env 的后续投递                                                                                                                            |
| worker loaderContext 对象来源                        | `_module` 使用本 env 的 Module 类与实例缓存；无 main.module 替换；`_compiler`/`_compilation` 仍通过现有 RPC 访问 main                                                        |
| worker Module API 与生命周期                         | 覆盖 identifier、readableIdentifier、originalSource、buildInfo、emitFile；不因 worker 缺少 COMPILER_REFERENCES 失败；结果保留到 Rust 后续阶段；重建/退出无缓存或失效指针残留 |
| main hook 与 worker context 交接                     | hook 按原顺序执行；extensions 不把 `_module` 覆盖成主线程代理；main/worker 不并发修改同一 module                                                                             |
| additionalData/options 释放与主订阅重建              | 无旧 release TSFN 错路由，无对象 ID 误释放                                                                                                                                   |
| TSFN 回调、消息端口和 compiler GC                    | 共享回调不保活 compiler，空闲时不额外阻止进程退出                                                                                                                            |
| 混合 loader/builtin/pitch 与 WorkerFunction 扩展示例 | 两类完成协议独立；Loader 仍归还同一个 boxed state                                                                                                                            |
| main-only 与混合任务性能                             | 测量 channel/TSFN 开销与 worker 预取分布，不以统一队列推断性能收益                                                                                                           |

完成相关 Rust tests、clippy、JS 类型与 lint 检查。通用 WorkerFunction 可后续实现，当前先验证 Loader 在同一订阅模型下的正确性。

## V8 普通数据传输实验（2026-09-20）

本节覆盖前文 options / additionalData 一律使用 MainObjectHandle 的描述。仅这两类普通数据改为 V8 字节；compiler、compilation、module buildInfo 等活对象仍按现有桥接实现。

```text
packages/rspack/src/loader-runner/service.ts::registerParallelLoader
  node:v8.serialize({ options, parallel })，注册时执行一次
       ↓
crates/rspack_binding_api/src/plugins/js_loader/bridge.rs
  LoaderReferences.serialized_options 保存 Rust Vec<u8>，按 compiler handle / ident 索引
       ↓
worker createWorkerCompiler / references.get
  getSerializedLoaderOptions 直接读取 Rust 字节 → node:v8.deserialize → worker 本地缓存

loader callback 的 additionalData
  当前线程 node:v8.serialize
       ↓
result.rs::merge_loader_context
  SerializedLoaderData(Vec<u8>) 保存到 native LoaderContext.additional_data
       ↓
MPMC 队列移动 Box<LoaderContext>，中间 builtin 可以原样传递 additionalData
       ↓
下一段 JS loader 所在线程 task.takeContext
  context.rs 构造 Buffer → node:v8.deserialize → 执行 loader
```

主线程 prepare hook 使用 hooks_only context，不构造 additionalData Buffer。options 与 compiler 引用表一起清理；additionalData 与 LoaderContext 一起清理，没有新的 JS 对象保活注册表。当前实现跨 N-API 边界复制字节，不声称零拷贝。

worker options 缓存沿用 referenceVersion 失效机制；注册新值时替换 Rust 缓存，不能序列化时移除旧字节。序列化是注册时的值快照，后续主线程原对象原地修改不会自动同步。数据图内部循环与共享引用保留；跨线程原对象身份、任意自定义原型和访问器不属于 V8 数据序列化保证。函数值不纳入本次实验；序列化失败、additionalData 根对象为自定义类时保留既有对象桥。浏览器/WASM 沿用原有 handle 路径。

验证使用 serialized-data 配置用例，让 options getter 在 make 开始后抛错，从而检测 worker 是否仍回主线程读取 options；同一用例验证 additionalData 在 main → worker → builtin → worker → main 往返后的值和图内引用。性能复测沿用 10,000 个大 JS 模块各自导入 CSS Module 的 Babel + CSS extract 场景。该场景主要测 options 读取，不能据此推断大体积 additionalData 的收益。

## Parallel loader 跳过 loader hook（2026-09-20）

worker 的 pitch / normal task 不再执行 NormalModule.loader hook。main loader 段仍按原实现执行 hook；compiler.options.loader 配置、compiler / compilation API 和 buildInfo 桥接继续可用。hook 注入的自定义 loaderContext 属性不再提供给 worker。

```text
Rust MPMC → worker.execute
       ↓
prepareWorkerTask → main
  prepareLoaderTask 仅取得 native context 的 main wrapper
  注册 compiler / buildInfo 桥接对象（不调用 runLoaders / loader hook）
  绑定 handle 的原有 task 生命周期
  单次响应直接带回桥接数据
       ↓
worker task.takeContext → runLoaders（跳过 hook）→ complete
```

本次消除主线程 JS loaderContext 的构造与 loader hook 调用，并合并原本的 getMainObject 同步往返。仍有一次 main 准备请求，用于 compiler / compilation / buildInfo 桥接；Rust hooksOnly snapshot 的转换仍保留，不能将这个实验描述为完全没有 main 参与。logger 等其他 RPC 和 V8 数据路径保持不变。
