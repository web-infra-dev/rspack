# 使用 goexec 的原生编译器

可选的 `rspack/goexec` Cargo feature 使用已发布的 goexec 0.1.3 执行原生 Rust
编译任务，同时启用 `rspack_tasks`、`rspack_fs` 和 `rspack_plugin_schemes` 的对应后端。
默认仍使用 Tokio。Cargo.lock 固定依赖，无需 Git 或本地路径覆盖。

## 集成范围

`rspack_tasks::runtime` 统一选择 Runtime、JoinHandle、JoinError 和任务提交函数。
编译器上下文任务与 `rspack_parallel` 使用该接口。原生编译工作必须运行在所选执行器内；
`src/main.rs` 展示完整编译生命周期。丢弃任务句柄仍会让任务继续运行。

短文件系统调用在 goexec 跟踪的阻塞区间内执行；资源读取保留原有同步实现和错误上下文。
CPU 密集的延迟销毁作为普通任务执行，受 CPU 执行许可限制。Rayon 和 Tokio 的
任务局部状态、同步原语继续保留。

此 feature 仅覆盖原生 Rust Compiler API。Node/NAPI、JavaScript 插件/loader、watch
定时器、网络和持久缓存后台运行时仍依赖原有集成。不要在 Node binding 中全局开启。
基准程序禁用持久缓存。Cargo feature 合并会为整个依赖图选择同一个后端。

## 构建和比较

使用仓库固定的 Rust 工具链、Python 3 和 pnpm 10.11.0，在仓库根目录运行：

```sh
python3 xtask/goexec-compare/prepare-fixtures.py
cargo build --locked --profile executor-bench -p rspack_goexec_compare
cp target/executor-bench/rspack_goexec_compare target/goexec-compare/rspack-tokio
cargo build --locked --profile executor-bench -p rspack_goexec_compare --features goexec
cp target/executor-bench/rspack_goexec_compare target/goexec-compare/rspack-goexec
target/goexec-compare/rspack-tokio target/goexec-compare/fixtures threejs-10x development 12 2 3
target/goexec-compare/rspack-goexec target/goexec-compare/fixtures threejs-10x development 12 2 3
```

参数依次为 fixture 目录、项目、模式、线程数、预热次数和测量次数。项目支持 basic-react、
threejs、threejs-10x；模式支持 development、production-sourcemap（不压缩）和
production-minify（无 source map）。执行器与 Rayon 线程数相同。Tokio 允许额外 512 个
阻塞线程；goexec 的总工作线程上限为 WORKERS + 512，另有监视线程，阻塞移交观察延迟为
100 微秒。运行时默认线程数没有改为示例中的 12。

计时包含 `Compiler::run()` 及输出写入。每次新建 Compiler，禁用持久缓存，使用热 OS
文件缓存；构造、输出哈希验证、关闭和销毁在主计时区间外。两个执行器均将根任务提交到
工作线程，清理后等待 100 ms。每次验证编译错误、磁盘与内存输出、asset SHA-256 和关闭状态。
编译配置为 opt-level 3、LTO off、16 codegen units、mimalloc。

macOS 比较时给两个版本使用相同的 USER_INITIATED QoS shim，命令见[英文说明](README.md)。
它不设置 CPU 亲和性或隔离后台负载。完成全部编译后再计时；七组进程交替执行器顺序，
每个进程预热两次、保留三次，比较输出哈希、模块数和字节数。生成结果保存到 `target/`
或被忽略的 `results/` 目录。

## 先前验证

2026-09-23，在 Apple M3 Max（12P + 4E，48 GiB）上，Three.js-10x 的 12 线程测试：

| 模式 | Tokio 中位数 | goexec 中位数 | 配对耗时变化 [95% CI] |
|---|---:|---:|---:|
| 开发构建 | 143.06 ms | 116.89 ms | -18.82% [-20.74, -17.11] |
| Source map | 277.11 ms | 246.61 ms | -11.62% [-12.57, -10.63] |
| 压缩 | 1218.97 ms | 1187.06 ms | -2.89% [-3.85, -1.88] |

测量基于 upstream Rspack `cbf189da225ee607d9f66197f0ad8e50d8258c57` 和 goexec main
`f5d82e14d8b068ecd0894065d3457bcf4ce33c54`。已发布 crate 的 Rust 源码一致；这些历史数据
不是对后续 upstream 或本分支重新整理后的集成进行的性能测量。

每种条件、每个执行器有 21 次保留测量。变化率为七组配对进程中位数比值的几何平均，
95% 区间使用 10,000 次配对 bootstrap。没有删除异常值，也未进行多重比较校正。
两个版本均使用 QoS shim，以及 `CARGO_PROFILE_EXECUTOR_BENCH_DEBUG=1` 和
`CARGO_PROFILE_EXECUTOR_BENCH_STRIP=none`；复现时对两个构建命令都使用这些设置。

1/12/16 线程的 630 次构建及另外 18 次三项目冒烟构建全部输出一致，关闭成功，
没有线程创建失败或容量延迟。16 线程耗时减少 2.0–3.3%；单线程仍慢 3.5–10.8%。
这些共享主机的原生编译结果不代表所有工作负载，或 Node/NAPI、JS loader、watch/HMR、
持久缓存性能。
