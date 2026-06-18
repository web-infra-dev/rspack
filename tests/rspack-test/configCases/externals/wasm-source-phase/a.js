import source wasmModule from "./static.wasm";

globalThis.staticModule = wasmModule;
globalThis.dynamicModule = import.source("./dynamic.wasm");
