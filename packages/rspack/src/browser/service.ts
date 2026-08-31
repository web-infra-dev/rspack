export enum RequestType {
  AddDependency = 'AddDependency',
  AddContextDependency = 'AddContextDependency',
  AddMissingDependency = 'AddMissingDependency',
  AddBuildDependency = 'AddBuildDependency',
  GetDependencies = 'GetDependencies',
  GetContextDependencies = 'GetContextDependencies',
  GetMissingDependencies = 'GetMissingDependencies',
  ClearDependencies = 'ClearDependencies',
  BeginDependencyChanges = 'BeginDependencyChanges',
  MergeDependencyChanges = 'MergeDependencyChanges',
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
  LoaderCacheGet = 'LoaderCacheGet',
  LoaderCacheStore = 'LoaderCacheStore',
  CompilationGetPath = 'CompilationGetPath',
  CompilationGetPathWithInfo = 'CompilationGetPathWithInfo',
  CompilationGetAssetPath = 'CompilationGetAssetPath',
  CompilationGetAssetPathWithInfo = 'CompilationGetAssetPathWithInfo',
}

export function run(): Promise<never> {
  return Promise.reject(new Error('Not support browser'));
}
