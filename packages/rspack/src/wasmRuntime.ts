import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

type BindingWithWasmRuntime = {
  setWasmRuntimeLibraryPath?: (path: string) => void;
};

type WasmRuntimePackage = {
  resolveWasmRuntime: () => string;
};

let registered = false;

export function hasSwcWasmPlugins(options: unknown): boolean {
  if (!options || typeof options !== 'object') {
    return false;
  }

  const plugins = (
    options as { jsc?: { experimental?: { plugins?: unknown } } }
  ).jsc?.experimental?.plugins;
  return Array.isArray(plugins) && plugins.length > 0;
}

export function registerWasmRuntime(): void {
  if (registered) {
    return;
  }

  const wasmRuntime = require('@rspack/wasm-runtime') as WasmRuntimePackage;
  const binding = require('@rspack/binding') as BindingWithWasmRuntime;
  const setWasmRuntimeLibraryPath = binding.setWasmRuntimeLibraryPath;

  if (typeof setWasmRuntimeLibraryPath !== 'function') {
    throw new Error(
      '@rspack/binding does not support dynamic @rspack/wasm-runtime loading',
    );
  }

  setWasmRuntimeLibraryPath(wasmRuntime.resolveWasmRuntime());
  registered = true;
}

export function tryRegisterWasmRuntime(): void {
  try {
    registerWasmRuntime();
  } catch {
    return;
  }
}
