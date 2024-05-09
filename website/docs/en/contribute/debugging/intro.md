# Debugging

## Debugging with VSCode

1. Install `go install github.com/go-delve/delve/cmd/dlv@latest`
2. Install VSCode extension [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) and [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
3. build `@rspack/cli` and napi binding by run `pnpm install && pnpm -w build:cli:debug`
4. In VSCode's `Run and Debug` tab, select `debug-rspack` to start debugging the initial launch of `@rspack/cli`. This task can be configured in `.vscode/launch.json`, which launches the Node and Rust debugger together.

## Tracing

[`tracing`](https://crates.io/crates/tracing) is used to instrumenting Rspack.

The supported tracing levels for

* release builds are `INFO`, `WARN` and `ERROR`
* debug builds are `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`

Use the `RSPACK_PROFILE` environment variable for displaying trace information

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

### `oxc_resolver`

`oxc_resolver` emits some tracing information for debugging purposes.

```bash
RSPACK_PROFILE='TRACE=filter=oxc_resolver=trace&layer=logger' rspack build
```

## rust-lldb

`rust-lldb` can be used to get panic information from debug builds

```bash
rust-lldb -- node /path/to/rspack build
```

Once it launches, press `r` for running the program.

For example, `examples/arco-pro` crashes without any information before [this fix](https://github.com/web-infra-dev/rspack/pull/3195/files):

```
rspack/examples/arco-pro ❯ node ../../packages/rspack-cli/bin/rspack build
Rspack ██████████████████████░░░░░░░░░░░░░░░░░░ 56% building ./pages/welcome
zsh: bus error  node ../../packages/rspack-cli/bin/rspack build
```

Using `rust-lldb`

```bash
rspack/examples/arco-pro ❯ rust-lldb -- node ../../packages/rspack-cli/bin/rspack build
```

Press `r` and it prints:

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
