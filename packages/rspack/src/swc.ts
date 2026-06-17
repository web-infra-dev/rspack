import type { TransformOutput } from '@rspack/binding';
import binding from '@rspack/binding';
import type { JsMinifyOptions, Options as TransformOptions } from '@swc/types';
import { hasSwcWasmPlugins, registerWasmRuntime } from './wasmRuntime';

export type { JsMinifyOptions, TransformOptions, TransformOutput };

export async function minify(
  source: string,
  options?: JsMinifyOptions,
): Promise<TransformOutput> {
  const _options = JSON.stringify(options || {});

  return binding.minify(source, _options);
}

export function minifySync(
  source: string,
  options?: JsMinifyOptions,
): TransformOutput {
  const _options = JSON.stringify(options || {});
  return binding.minifySync(source, _options);
}

export async function transform(
  source: string,
  options?: TransformOptions,
): Promise<TransformOutput> {
  if (hasSwcWasmPlugins(options)) {
    registerWasmRuntime();
  }
  const _options = JSON.stringify(options || {});

  return binding.transform(source, _options);
}

export function transformSync(
  source: string,
  options?: TransformOptions,
): TransformOutput {
  if (hasSwcWasmPlugins(options)) {
    registerWasmRuntime();
  }
  const _options = JSON.stringify(options || {});
  return binding.transformSync(source, _options);
}
