# Bench & Diagnostics

this repo is used to bench against esbuild and webpack, and can also be used to track the tracing of rspack and diagnostic perf problem.

## tracing

```sh
TRACE=TRACE cargo run --release -F tracing # generated tracing-xxx.json

npm install -g speedscope # if not install speedscope

speedscope trace-xxx.json # generate flamegraph
```
