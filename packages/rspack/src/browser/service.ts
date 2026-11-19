import type { JsLoaderContext } from '@rspack/binding';
import type { Compiler } from '../Compiler';

export function readAdditionalData(context: JsLoaderContext): unknown {
  return context.additionalData;
}

export function writeAdditionalData(
  context: JsLoaderContext,
  value: unknown,
  strict: boolean,
): void {
  context.additionalData =
    strict || value === null || value === undefined ? undefined : value;
  context.additionalDataSerialized = undefined;
}

export function getCompilerHandle(_compiler: Compiler): number {
  return 0;
}
export function registerParallelLoader(..._args: unknown[]): void {}

export function beginWorkerImport(): () => void {
  return () => {};
}
