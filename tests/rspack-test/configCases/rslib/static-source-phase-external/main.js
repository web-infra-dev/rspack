import source wasmSource from "./add.wasm";

const instance = new WebAssembly.Instance(wasmSource);

export const add = (a, b) => instance.exports.add(a, b);
