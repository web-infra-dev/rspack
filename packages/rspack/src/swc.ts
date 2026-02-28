import type { TransformOutput } from '@rspack/binding';
import binding from '@rspack/binding';
import type { JsMinifyOptions, Options as TransformOptions } from '@swc/types';

export type { TransformOutput, TransformOptions, JsMinifyOptions };

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
  const _options = JSON.stringify(options || {});

  return binding.transform(source, _options);
}

export function transformSync(
  source: string,
  options?: TransformOptions,
): TransformOutput {
  const _options = JSON.stringify(options || {});
  return binding.transformSync(source, _options);
}
