# Debugging

## 通过 VSCode 调试

1. 安装 `go install github.com/go-delve/delve/cmd/dlv@latest`
2. 安装 VSCode 扩展 [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 和 [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
3. 通过执行 `pnpm install && pnpm -w build:cli:debug` 构建 `@rspack/cli` 和 napi binding
4. 在 VSCode 的 `Run and Debug` 栏中, 选择 `debug-rspack` 开始调试`@rspack/cli` 的启动过程。 该任务可以在 `.vscode/launch.json` 中配置，会同时启动 Node 和 Rust 的调试器。

## Tracing

[`tracing`](https://crates.io/crates/tracing) 被用于度量（instrumenting） Rspack。

被支持 tracing 等级有：

- release 版本是 `INFO`, `WARN` and `ERROR`
- debug 版本是 `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`

使用 `RSPACK_PROFILE` 环境变量来展示 trace 信息。

```bash
RSPACK_PROFILE=TRACE=layer=logger rspack build
# filter for an event
RSPACK_PROFILE='TRACE=layer=logger&filter=rspack_core::compiler::compilation' rspack build
# with logger level
RSPACK_PROFILE='TRACE=layer=logger&filter=rspack_core::compiler::compilation=info' rspack build
# filter logs across multiple modules
RSPACK_PROFILE='TRACE=layer=logger&filter=rspack_core::compiler::compilation,rspack_core::build_chunk_graph::code_splitter' rspack build
# [fn_name] will show:
# - all functions calls to `fn_name`
# - the arguments(except for these in the `skip` list)
# - everything until this function returns
RSPACK_PROFILE='TRACE=layer=logger&filter=[build_chunk_graph]' rspack build
# compilation::chunk_asset is a custom instrument name
RSPACK_PROFILE='TRACE=layer=logger&filter=[compilation:chunk_asset]' rspack build
# log a specific function by their arguments
RSPACK_PROFILE='TRACE=layer=logger&filter=[compilation:chunk_asset{filename="main\.js"}]' rspack build
# It support regexp expression
RSPACK_PROFILE='TRACE=layer=logger&filter=[compilation:chunk_asset{filename=".*\.js"}]' rspack build
# disable ansi color escape codes
NO_COLOR=1 RSPACK_PROFILE=TRACE=layer=logger rspack build
```

### Resolver

`oxc_resolver` emits some tracing information for debugging purposes.

```bash
RSPACK_PROFILE='TRACE=filter=oxc_resolver=trace&layer=logger' rspack build
```

## rust-lldb

`rust-lldb` 可用于从 debug 版本中获取 panic 信息

```bash
rust-lldb -- node /path/to/rspack build
```

启动后，按住 `r` 来执行程序。

例如，`examples/arco-pro` 崩溃了并且没有任何信息在[这个修复](https://github.com/web-infra-dev/rspack/pull/3195/files) 之前:

```
rspack/examples/arco-pro ❯ node ../../packages/rspack-cli/bin/rspack build
Rspack ██████████████████████░░░░░░░░░░░░░░░░░░ 56% building ./pages/welcome
zsh: bus error  node ../../packages/rspack-cli/bin/rspack build
```

使用 `rust-lldb`

```bash
rspack/examples/arco-pro ❯ rust-lldb -- node ../../packages/rspack-cli/bin/rspack build
```

按下 `r` 然后会打印:

```
Process 23110 stopped
* thread #10, name = 'tokio-runtime-worker', stop reason = EXC_BAD_ACCESS (code=2, address=0x70000cc66560)
    frame #0: 0x0000000140d0db4b rspack.darwin-x64.node`swc_ecma_parser::parser::expr::ops::_$LT$impl$u20$swc_ecma_parser..parser..Parser$LT$I$GT$$GT$::parse_unary_expr::h29f49330a806839c(self=0x0000000000000000) at ops.rs:244
   241 	    /// Parse unary expression and update expression.
   242 	    ///
   243 	    /// spec: 'UnaryExpression'
-> 244 	    pub(in crate::parser) fn parse_unary_expr(&mut self) -> PResult<Box<Expr>> {
   245 	        trace_cur!(self, parse_unary_expr);
   246 	        let start = cur_pos!(self);
   247
Target 0: (node) stopped.
```

## 混合调试

本节旨在说明 JavaScript 和 Rust 混合调试的方法。

### 准备工作

为了说明这个过程，我将使用一个例子。首先介绍一下我使用的环境和例子。

- System: macos
- IDE: vscode
- Debugging target: `rspack build ${projectRoot}/basic`

首先，您需要在调试模式下构建 rspack。为此，请在项目的根目录中执行以下命令：

```bash
npm run build:binding:debug
npm run build:js
```

### 在 VSCode 中设置 `launch.json`

需要在 `.vscode/launch.json` 中配置两个调试配置。

- 给 node 添加 attach:

```jsonc
{
  "name": "attach:node",
  "request": "attach", // refer: https://code.visualstudio.com/docs/editor/debugging#_launch-versus-attach-configurations
  "type": "node",
  // `9229` is the default port of message
  "port": 9229,
}
```

- 和 lldb 的 launch 配置

```jsonc
{
  "name": "launch:rust-from-node",
  "request": "launch",
  "type": "lldb", // it means we use `lldb` to launch the binary file of `node`
  "program": "node",
  "args": [
    "--inspect",
    "--enable-source-maps",
    "${workspaceFolder}/packages/rspack-cli/bin/rspack",
    "build",
    "-c",
    "${workspaceFolder}/examples/basic/rspack.config.js",
  ],
  // `cwd` is just for repack find the correctly entry.
  "cwd": "${workspaceFolder}/examples/basic/",
}
```

Next, we can utilize [compounds](https://code.visualstudio.com/docs/editor/debugging#_compound-launch-configurations) to amalgamate the two commands:

```json
{
  "name": "mix-debug",
  "configurations": ["attach:node", "launch:rust-from-node"]
}
```

Finally, your `﻿launch.json` should appear as follows:

```json
{
  "configurations": [
    {
      "name": "attach:node",
      "request": "attach",
      "type": "node",
      "port": 9229
    },
    {
      "name": "launch:rust-from-node",
      "request": "launch",
      "type": "lldb",
      "program": "node",
      "args": [
        "--inspect",
        "--enable-source-maps",
        "${workspaceFolder}/packages/rspack-cli/bin/rspack",
        "build",
        "-c",
        "${workspaceFolder}/examples/basic/rspack.config.js"
      ],
      "cwd": "${workspaceFolder}/examples/basic/"
    }
  ],
  "compounds": [
    {
      "name": "mix-debug",
      "configurations": ["attach:node", "launch:rust-from-node"]
    }
  ]
}
```

### Debugging Attempt

Next, we can introduce some breakpoints and commence debugging.

The result appears as follows:

<video width="640" height="480" controls>
  <source src="https://github.com/web-infra-dev/rspack/assets/30187863/106983f7-a59e-4d9e-9001-552f4441d88b" type="video/mp4">
  Your browser does not support the video tag.
</video>
