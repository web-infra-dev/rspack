import type { JsLoaderContext } from '@rspack/binding';

export enum RequestType {
  AddDependency = 'AddDependency',
  AddContextDependency = 'AddContextDependency',
  AddMissingDependency = 'AddMissingDependency',
  AddBuildDependency = 'AddBuildDependency',
  GetDependencies = 'GetDependencies',
  GetContextDependencies = 'GetContextDependencies',
  GetMissingDependencies = 'GetMissingDependencies',
  ClearDependencies = 'ClearDependencies',
  Resolve = 'Resolve',
  GetResolve = 'GetResolve',
  GetLogger = 'GetLogger',
  EmitError = 'EmitError',
  EmitWarning = 'EmitWarning',
  EmitFile = 'EmitFile',
  EmitDiagnostic = 'EmitDiagnostic',
  SetCacheable = 'SetCacheable',
  ImportModule = 'ImportModule',
  UpdateLoaderObjects = 'UpdateLoaderObjects',
  UpdateBuildInfo = 'UpdateBuildInfo',
  CompilationGetPath = 'CompilationGetPath',
  CompilationGetPathWithInfo = 'CompilationGetPathWithInfo',
  CompilationGetAssetPath = 'CompilationGetAssetPath',
  CompilationGetAssetPathWithInfo = 'CompilationGetAssetPathWithInfo',
}

export function readAdditionalData(context: JsLoaderContext): unknown {
  return context.additionalData;
}

export function writeAdditionalData(
  context: JsLoaderContext,
  value: unknown,
): void {
  context.additionalData =
    value === null || value === undefined ? undefined : value;
  context.additionalDataSerialized = undefined;
}

export function run(): Promise<never> {
  return Promise.reject(new Error('Not support browser'));
}
