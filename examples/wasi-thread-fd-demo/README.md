# WASI Threaded FD Namespace Demo

This demo isolates the runtime mismatch behind the rspack wasm flaky:

- one shared `WebAssembly.Memory`
- two JavaScript threads
- each thread creates its own `node:wasi` `WASI` instance
- both instantiate the same `wasm32-wasip1-threads` module on that shared memory
- both open the same file and keep the file handle alive

If the WASI host state were process-global, the second open would allocate a
different file descriptor while the first one is still alive. In practice the
worker also gets `fd=4`, which shows that each `WASI` instance has its own host
fd namespace even though the wasm program is sharing one process image.

This mirrors the current rspack wasm runtime shape closely enough for the root
cause:

- main thread: one `WASI` instance
- worker thread: another `WASI` instance
- shared wasm memory across both threads
- filesystem syscalls executed from both threads

That combination is enough to reproduce the fd-namespace mismatch without any
rspack-specific compilation logic.

## Run

```bash
cd examples/wasi-thread-fd-demo
cargo build --manifest-path rust-demo/Cargo.toml --target wasm32-wasip1-threads --profile ci
node run.mjs
```

Expected output contains:

```text
main fd: 4
worker fd: 4
```

That is the core mismatch that later blows up in threaded WASI filesystem code
such as `std::sys::fs::wasi::open_parent`.
