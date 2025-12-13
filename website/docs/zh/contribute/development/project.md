# 项目结构

这是一个包含 Rust crates 和 JavaScript 包的**单体仓库**：

## Rust crates (`crates/`)

### 核心 Crates

- **`rspack`**: 集成所有核心功能和插件的主 crate，提供完整的构建工具入口点
- **`rspack_core`**: 包含模块系统、依赖图、编译管道和其他核心功能的核心引擎
- **`rspack_binding_api`**: Node.js 绑定 API，将 Rust 核心功能桥接到 JavaScript/TypeScript 接口
- **`node_binding`**: Node.js 绑定实现，用于生成 Node.js 原生模块
- **`rspack_napi`**: NAPI (Node-API) 支持层，用于 Rust 和 Node.js 之间的互操作性
- **`rspack_allocator`**: 使用 mimalloc 的内存分配器，用于优化内存分配性能（Linux/macOS）

### 构建与绑定 Crates

- **`rspack_binding_build`**: 用于构建 Node.js 原生绑定的构建脚本
- **`rspack_binding_builder`**: 用于生成自定义 Rspack 绑定的绑定构建器
- **`rspack_binding_builder_macros`**: 绑定构建器宏，提供过程宏支持
- **`rspack_binding_builder_testing`**: 绑定构建器测试工具

### 工具类 Crates

- **`rspack_error`**: 错误处理和格式化，提供用户友好的错误消息
- **`rspack_fs`**: 文件系统抽象层，提供跨平台文件操作接口
- **`rspack_paths`**: 路径工具，用于路径规范化、解析和操作
- **`rspack_hash`**: 哈希算法实现，支持 MD4、SHA2、xxhash 等
- **`rspack_regex`**: 正则表达式工具，提供高性能的正则匹配
- **`rspack_location`**: 位置信息处理，用于源代码位置跟踪
- **`rspack_ids`**: ID 生成和管理，用于为模块、chunk 等创建唯一标识符
- **`rspack_collections`**: 集合数据结构，提供优化的 HashMap、HashSet 等
- **`rspack_util`**: 工具函数集合，包含各种辅助函数
- **`rspack_futures`**: 异步工具，提供异步编程支持
- **`rspack_workspace`**: 工作区支持，用于处理单体仓库场景

### 缓存与存储

- **`rspack_cacheable`**: 缓存系统，提供可缓存数据的序列化和反序列化
- **`rspack_cacheable_macros`**: 缓存系统宏，用于自动生成缓存相关代码
- **`rspack_cacheable_test`**: 缓存系统测试
- **`rspack_storage`**: 存储抽象层，提供持久化存储接口

### Hook 系统

- **`rspack_hook`**: Hook 系统，实现插件 hook 注册和调用机制
- **`rspack_macros`**: 过程宏，提供各种编译时代码生成功能
- **`rspack_macros_test`**: 宏系统测试

### 编译与转换

- **`rspack_javascript_compiler`**: JavaScript 编译器，用于处理 JS/TS 代码编译
- **`rspack_loader_runner`**: Loader 运行器，用于执行各种 loader 来处理资源
- **`rspack_loader_swc`**: SWC loader，使用 SWC 进行代码转换
- **`rspack_loader_lightningcss`**: Lightning CSS loader，用于处理 CSS 文件
- **`rspack_loader_react_refresh`**: React Fast Refresh loader，支持 React 热模块替换
- **`rspack_loader_preact_refresh`**: Preact Refresh loader，支持 Preact 热模块替换
- **`rspack_loader_testing`**: Loader 测试工具

### 插件系统

#### 核心插件

- **`rspack_plugin_javascript`**: JavaScript 插件，用于解析、转换和生成 JS/TS 模块代码
- **`rspack_plugin_runtime`**: Runtime 插件，生成 webpack 兼容的运行时代码
- **`rspack_plugin_entry`**: Entry 插件，用于处理入口点配置
- **`rspack_plugin_dynamic_entry`**: 动态入口插件，支持动态入口点

#### 资源与资源插件

- **`rspack_plugin_asset`**: Asset 插件，用于处理静态资源
- **`rspack_plugin_copy`**: Copy 插件，用于将文件复制到输出目录
- **`rspack_plugin_json`**: JSON 插件，用于处理 JSON 文件
- **`rspack_plugin_wasm`**: WebAssembly 插件，用于处理 WASM 模块
- **`rspack_plugin_html`**: HTML 插件，用于生成 HTML 文件

#### CSS 插件

- **`rspack_plugin_css`**: CSS 插件，用于处理 CSS 模块和样式
- **`rspack_plugin_extract_css`**: CSS 提取插件，用于将 CSS 提取到单独的文件
- **`rspack_plugin_css_chunking`**: CSS 代码分割插件
- **`rspack_plugin_lightning_css_minimizer`**: Lightning CSS 压缩插件

#### 优化插件

- **`rspack_plugin_swc_js_minimizer`**: SWC JavaScript 压缩插件
- **`rspack_plugin_split_chunks`**: 代码分割插件，实现 chunk 分割策略
- **`rspack_plugin_merge_duplicate_chunks`**: 合并重复 chunk 插件
- **`rspack_plugin_remove_empty_chunks`**: 移除空 chunk 插件
- **`rspack_plugin_remove_duplicate_modules`**: 移除重复模块插件
- **`rspack_plugin_limit_chunk_count`**: 限制 chunk 数量插件
- **`rspack_plugin_real_content_hash`**: 真实内容哈希插件，基于内容生成哈希

#### 开发插件

- **`rspack_plugin_hmr`**: 热模块替换 (HMR) 插件
- **`rspack_plugin_devtool`**: Source Map 插件，用于生成 source map
- **`rspack_plugin_progress`**: 进度显示插件
- **`rspack_plugin_lazy_compilation`**: 懒编译插件

#### 库与模块插件

- **`rspack_plugin_library`**: Library 插件，用于生成库文件
- **`rspack_plugin_esm_library`**: ESM 库插件，用于生成 ES 模块格式的库
- **`rspack_plugin_externals`**: Externals 插件，用于排除外部依赖
- **`rspack_plugin_module_replacement`**: 模块替换插件，支持模块别名
- **`rspack_plugin_ignore`**: Ignore 插件，用于忽略特定模块

#### 高级功能

- **`rspack_plugin_mf`**: 模块联邦插件，实现微前端模块联邦
- **`rspack_plugin_dll`**: DLL 插件，实现动态链接库功能
- **`rspack_plugin_worker`**: Web Worker 插件，用于处理 Worker 文件
- **`rspack_plugin_web_worker_template`**: Web Worker 模板插件
- **`rspack_plugin_schemes`**: 自定义 scheme 插件，支持自定义资源协议
- **`rspack_plugin_runtime_chunk`**: Runtime chunk 插件，用于分离运行时代码

#### 工具插件

- **`rspack_plugin_ensure_chunk_conditions`**: 确保 chunk 条件插件
- **`rspack_plugin_no_emit_on_errors`**: 错误时不输出插件
- **`rspack_plugin_circular_dependencies`**: 循环依赖检测插件
- **`rspack_plugin_banner`**: Banner 插件，用于添加文件头部注释
- **`rspack_plugin_size_limits`**: 大小限制插件，用于检查 bundle 大小
- **`rspack_plugin_sri`**: 子资源完整性 (SRI) 插件
- **`rspack_plugin_module_info_header`**: 模块信息头部插件
- **`rspack_plugin_warn_sensitive_module`**: 警告敏感模块插件

#### 调试与测试插件

- **`rspack_plugin_rsdoctor`**: RsDoctor 插件，提供调试和诊断功能
- **`rspack_plugin_rslib`**: RsLib 插件，用于库构建
- **`rspack_plugin_rstest`**: RsTest 插件，用于测试

### 浏览器与环境支持

- **`rspack_browser`**: 浏览器环境支持，提供浏览器端实现
- **`rspack_browserslist`**: Browserslist 支持，用于处理浏览器兼容性查询

### 监控与追踪

- **`rspack_tracing`**: 追踪系统，提供性能追踪功能
- **`rspack_tracing_perfetto`**: Perfetto 追踪支持，集成 Perfetto 性能分析工具
- **`rspack_watcher`**: 文件监听器，监控文件变化以触发重新构建
- **`rspack_tasks`**: 任务系统，用于管理构建任务

### SWC 插件

- **`swc_plugin_import`**: SWC import 插件，用于处理模块导入转换
- **`swc_plugin_ts_collector`**: SWC TypeScript 收集器插件，用于收集 TS 类型信息

## NPM 包 (`packages/`)

### 核心包

- **`@rspack/core`**: 主要的 JavaScript/TypeScript 包，提供 webpack 兼容的 API，包装 Rust 核心功能并暴露完整的构建工具接口供 Node.js 应用使用

### CLI 工具

- **`@rspack/cli`**: 命令行接口，提供 build、serve 和 preview 命令，用于从终端运行 Rspack
- **`create-rspack`**: 项目脚手架工具，用于创建新的 Rspack 项目，支持多种模板（vanilla、React、Vue），同时支持 JavaScript 和 TypeScript

### 浏览器支持

- **`@rspack/browser`**: Rspack 的浏览器兼容版本，可以在浏览器环境中使用 WebAssembly 运行，目前处于早期开发阶段

### 测试工具

- **`@rspack/test-tools`**: 测试工具和辅助函数，用于编写和运行 Rspack 测试，包括对 WebAssembly 测试执行的支持

## 测试用例 (`tests/`)

### 核心测试套件 (`rspack-test/`)

Rspack 核心功能的主要测试套件，包含各种模拟构建过程的测试类型：

#### 测试类型

- **`normalCases/`** (`Normal.test.js`, `Normal-dev.test.js`, `Normal-hot.test.js`, `Normal-prod.test.js`): 用于测试无需配置更改的核心构建过程的测试用例，当测试不需要 `rspack.config.js` 时使用
- **`configCases/`** (`Config.part1.test.js`, `Config.part2.test.js`, `Config.part3.test.js`): 用于测试构建配置选项的测试用例，允许通过 `rspack.config.js` 指定构建配置，通过 `test.config.js` 控制测试行为
- **`hotCases/`** (`HotNode.test.js`, `HotWeb.test.js`, `HotWorker.test.js`, `HotSnapshot.hottest.js`): 用于测试热模块替换 (HMR) 功能的测试用例，包括 HotNode (`target=async-node`)、HotWeb (`target=web`) 和 HotWorker (`target=webworker`)
- **`watchCases/`** (`Watch.part1.test.js`, `Watch.part2.test.js`, `Watch.part3.test.js`): 用于测试 Watch 模式下增量编译的测试用例，使用编号目录 (0, 1, 2...) 表示变更批次
- **`statsOutputCases/`** (`StatsOutput.test.js`): 用于测试构建完成后控制台输出日志的测试用例，快照存储在 `__snapshots__/StatsOutput.test.js.snap`
- **`statsAPICases/`** (`StatsAPI.test.js`): 用于测试构建完成后生成的 Stats 对象的测试用例，使用 `tests/rspack-test/fixtures` 作为源代码
- **`diagnosticsCases/`** (`Diagnostics.test.js`): 用于测试构建过程中格式化的警告/错误输出的测试用例，快照存储在 `stats.err` 文件中
- **`hashCases/`** (`Hash.test.js`): 用于测试哈希生成功能的测试用例，验证 Stats 对象中的哈希信息
- **`compilerCases/`** (`Compiler.test.js`): 用于测试 Compiler/Compilation 对象 API 的测试用例，使用 `tests/rspack-test/fixtures` 作为源代码
- **`defaultsCases/`** (`Defaults.test.js`): 用于测试配置选项交互的测试用例，生成构建配置并观察与默认配置的差异
- **`errorCases/`** (`Error.test.js`): 用于测试 `compilation.errors` 和 `compilation.warnings` 交互的测试用例
- **`hookCases/`** (`Hook.test.js`): 用于测试各种 hook 功能的测试用例，在快照中记录 hook 的输入/输出
- **`treeShakingCases/`** (`TreeShaking.test.js`): 用于测试 Tree Shaking 相关功能的测试用例，产品快照存储在 `__snapshots__/treeshaking.snap.txt`
- **`builtinCases/`** (`Builtin.test.js`): 用于测试具有内置原生实现的插件的测试用例，根据插件类型（CSS、CSS 模块、HTML、JavaScript）生成不同的快照
- **`cacheCases/`** (`Cache.test.js`): 用于测试缓存功能的测试用例，包括通用缓存、失效、快照、存储和可移植缓存场景
- **`serialCases/`** (`Serial.test.js`): 用于测试串行执行场景的测试用例
- **`exampleCases/`** (`Example.test.js`): 演示各种 Rspack 功能的示例测试用例
- **`esmOutputCases/`** (`EsmOutput.test.js`): 用于测试 ES 模块输出功能的测试用例，包括基本输出、去冲突、动态导入、外部依赖、互操作、命名空间、保留模块、重新导出和代码分割场景
- **`multiCompilerCases/`** (`MultiCompiler.test.js`): 用于测试多编译器场景的测试用例
- **增量测试** (`Incremental-node.test.js`, `Incremental-async-node.test.js`, `Incremental-web.test.js`, `Incremental-webworker.test.js`, `Incremental-watch.test.js`): 用于测试针对不同环境的增量编译的测试用例
- **原生监听器测试** (`NativeWatcher.part1.test.js`, `NativeWatcher.part2.test.js`, `NativeWatcher.part3.test.js`): 用于测试原生文件监听器功能的测试用例

#### 支持目录

- **`fixtures/`**: 通用测试文件和跨多个测试类型共享的 fixtures
- **`js/`**: 测试执行期间生成的构建产物和临时文件，按测试类型组织（例如，`js/normal`、`js/config`、`js/hot-{target}`）
- **`__snapshots__/`**: 各种测试类型的测试快照，包括 StatsOutput、HotSnapshot 和其他基于快照的测试

### E2E 测试 (`e2e/`)

Rspack 的端到端测试，涵盖真实场景和集成测试：

- **`cases/`**: 按功能区域组织的 E2E 测试用例（chunk、css、file、hooks、html、incremental、lazy-compilation、make、module-federation、persistent-cache、react、vue3、worker）
- **`fixtures/`**: E2E 测试的共享 fixtures 和工具
- **`utils/`**: E2E 测试执行的工具函数

### 基准测试 (`bench/`)

用于跟踪 Rspack JavaScript API 性能并防止性能退化的性能基准测试：

- **`fixtures/`**: 基准测试 fixtures（例如，用于基准测试的 `ts-react` 项目）
- 用于测量构建性能和 API 执行时间的基准测试文件
