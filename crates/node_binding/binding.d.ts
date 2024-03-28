/* auto-generated by NAPI-RS */
/* eslint-disable */


export class ExternalObject<T> {
  readonly '': {
    readonly '': unique symbol
    [K: symbol]: T
  }
}
export class JsCompilation {
  updateAsset(filename: string, newSourceOrFunction: JsCompatSource | ((source: JsCompatSource) => JsCompatSource), assetInfoUpdateOrFunction?: JsAssetInfo | ((assetInfo: JsAssetInfo) => JsAssetInfo)): void
  getAssets(): Readonly<JsAsset>[]
  getAsset(name: string): JsAsset | null
  getAssetSource(name: string): JsCompatSource | null
  getModules(): Array<JsModule>
  getOptimizationBailout(): Array<JsStatsOptimizationBailout>
  getChunks(): Array<JsChunk>
  getNamedChunk(name: string): JsChunk | null
  setAssetSource(name: string, source: JsCompatSource): void
  deleteAssetSource(name: string): void
  getAssetFilenames(): Array<string>
  hasAsset(name: string): boolean
  emitAsset(filename: string, source: JsCompatSource, assetInfo: JsAssetInfo): void
  deleteAsset(filename: string): void
  renameAsset(filename: string, newName: string): void
  get entrypoints(): Record<string, JsChunkGroup>
  get hash(): string | null
  getFileDependencies(): Array<string>
  getContextDependencies(): Array<string>
  getMissingDependencies(): Array<string>
  getBuildDependencies(): Array<string>
  pushDiagnostic(severity: "error" | "warning", title: string, message: string): void
  pushNativeDiagnostics(diagnostics: ExternalObject<'Diagnostic[]'>): void
  getStats(): JsStats
  getAssetPath(filename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string), data: PathData): string
  getAssetPathWithInfo(filename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string), data: PathData): PathWithInfo
  getPath(filename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string), data: PathData): string
  getPathWithInfo(filename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string), data: PathData): PathWithInfo
  addFileDependencies(deps: Array<string>): void
  addContextDependencies(deps: Array<string>): void
  addMissingDependencies(deps: Array<string>): void
  addBuildDependencies(deps: Array<string>): void
  rebuildModule(moduleIdentifiers: Array<string>, f: (...args: any[]) => any): void
  importModule(request: string, publicPath: string | undefined | null, baseUri: string | undefined | null, originalModule: string | undefined | null, originalModuleContext: string | undefined | null, callback: (...args: any[]) => any): void
}

export class JsStats {
  getAssets(): JsStatsGetAssets
  getModules(reasons: boolean, moduleAssets: boolean, nestedModules: boolean, source: boolean, usedExports: boolean, providedExports: boolean): Array<JsStatsModule>
  getChunks(chunkModules: boolean, chunksRelations: boolean, reasons: boolean, moduleAssets: boolean, nestedModules: boolean, source: boolean, usedExports: boolean, providedExports: boolean): Array<JsStatsChunk>
  getEntrypoints(): Array<JsStatsChunkGroup>
  getNamedChunkGroups(): Array<JsStatsChunkGroup>
  getErrors(): Array<JsStatsError>
  getWarnings(): Array<JsStatsWarning>
  getLogging(acceptedTypes: number): Array<JsStatsLogging>
  getHash(): string | null
}

export class Rspack {
  constructor(options: RawOptions, builtinPlugins: Array<BuiltinPlugin>, jsHooks: JsHooks, registerJsTaps: RegisterJsTaps, outputFilesystem: ThreadsafeNodeFS)
  setDisabledHooks(hooks: Array<string>): void
  setNonSkippableRegisters(kinds: Array<RegisterJsTapKind>): void
  /** Build with the given option passed to the constructor */
  build(callback: (err: null | Error) => void): void
  /** Rebuild with the given option passed to the constructor */
  rebuild(changed_files: string[], removed_files: string[], callback: (err: null | Error) => void): void
}

export function __chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable(jsChunkUkey: number, compilation: JsCompilation): Array<JsChunk>

export function __chunk_graph_inner_get_chunk_entry_modules(jsChunkUkey: number, compilation: JsCompilation): Array<JsModule>

export function __chunk_graph_inner_get_chunk_modules(jsChunkUkey: number, compilation: JsCompilation): Array<JsModule>

export function __chunk_graph_inner_get_chunk_modules_iterable_by_source_type(jsChunkUkey: number, sourceType: string, compilation: JsCompilation): Array<JsModule>

export function __chunk_group_inner_get_chunk_group(ukey: number, compilation: JsCompilation): JsChunkGroup

export function __chunk_inner_can_be_initial(jsChunkUkey: number, compilation: JsCompilation): boolean

export function __chunk_inner_get_all_async_chunks(jsChunkUkey: number, compilation: JsCompilation): Array<JsChunk>

export function __chunk_inner_get_all_initial_chunks(jsChunkUkey: number, compilation: JsCompilation): Array<JsChunk>

export function __chunk_inner_get_all_referenced_chunks(jsChunkUkey: number, compilation: JsCompilation): Array<JsChunk>

export function __chunk_inner_has_runtime(jsChunkUkey: number, compilation: JsCompilation): boolean

export function __chunk_inner_is_only_initial(jsChunkUkey: number, compilation: JsCompilation): boolean

export function __entrypoint_inner_get_runtime_chunk(ukey: number, compilation: JsCompilation): JsChunk

export interface BuiltinPlugin {
  name: BuiltinPluginName
  options: unknown
  canInherentFromParent?: boolean
}

export enum BuiltinPluginName {
  DefinePlugin = 'DefinePlugin',
  ProvidePlugin = 'ProvidePlugin',
  BannerPlugin = 'BannerPlugin',
  ProgressPlugin = 'ProgressPlugin',
  EntryPlugin = 'EntryPlugin',
  ExternalsPlugin = 'ExternalsPlugin',
  NodeTargetPlugin = 'NodeTargetPlugin',
  ElectronTargetPlugin = 'ElectronTargetPlugin',
  EnableChunkLoadingPlugin = 'EnableChunkLoadingPlugin',
  EnableLibraryPlugin = 'EnableLibraryPlugin',
  EnableWasmLoadingPlugin = 'EnableWasmLoadingPlugin',
  ChunkPrefetchPreloadPlugin = 'ChunkPrefetchPreloadPlugin',
  CommonJsChunkFormatPlugin = 'CommonJsChunkFormatPlugin',
  ArrayPushCallbackChunkFormatPlugin = 'ArrayPushCallbackChunkFormatPlugin',
  ModuleChunkFormatPlugin = 'ModuleChunkFormatPlugin',
  HotModuleReplacementPlugin = 'HotModuleReplacementPlugin',
  LimitChunkCountPlugin = 'LimitChunkCountPlugin',
  WorkerPlugin = 'WorkerPlugin',
  WebWorkerTemplatePlugin = 'WebWorkerTemplatePlugin',
  MergeDuplicateChunksPlugin = 'MergeDuplicateChunksPlugin',
  SplitChunksPlugin = 'SplitChunksPlugin',
  ShareRuntimePlugin = 'ShareRuntimePlugin',
  ContainerPlugin = 'ContainerPlugin',
  ContainerReferencePlugin = 'ContainerReferencePlugin',
  ProvideSharedPlugin = 'ProvideSharedPlugin',
  ConsumeSharedPlugin = 'ConsumeSharedPlugin',
  NamedModuleIdsPlugin = 'NamedModuleIdsPlugin',
  DeterministicModuleIdsPlugin = 'DeterministicModuleIdsPlugin',
  NamedChunkIdsPlugin = 'NamedChunkIdsPlugin',
  DeterministicChunkIdsPlugin = 'DeterministicChunkIdsPlugin',
  RealContentHashPlugin = 'RealContentHashPlugin',
  RemoveEmptyChunksPlugin = 'RemoveEmptyChunksPlugin',
  EnsureChunkConditionsPlugin = 'EnsureChunkConditionsPlugin',
  WarnCaseSensitiveModulesPlugin = 'WarnCaseSensitiveModulesPlugin',
  DataUriPlugin = 'DataUriPlugin',
  FileUriPlugin = 'FileUriPlugin',
  RuntimePlugin = 'RuntimePlugin',
  JsonModulesPlugin = 'JsonModulesPlugin',
  InferAsyncModulesPlugin = 'InferAsyncModulesPlugin',
  JavascriptModulesPlugin = 'JavascriptModulesPlugin',
  AsyncWebAssemblyModulesPlugin = 'AsyncWebAssemblyModulesPlugin',
  AssetModulesPlugin = 'AssetModulesPlugin',
  SourceMapDevToolPlugin = 'SourceMapDevToolPlugin',
  EvalSourceMapDevToolPlugin = 'EvalSourceMapDevToolPlugin',
  EvalDevToolModulePlugin = 'EvalDevToolModulePlugin',
  SideEffectsFlagPlugin = 'SideEffectsFlagPlugin',
  FlagDependencyExportsPlugin = 'FlagDependencyExportsPlugin',
  FlagDependencyUsagePlugin = 'FlagDependencyUsagePlugin',
  MangleExportsPlugin = 'MangleExportsPlugin',
  ModuleConcatenationPlugin = 'ModuleConcatenationPlugin',
  HttpExternalsRspackPlugin = 'HttpExternalsRspackPlugin',
  CopyRspackPlugin = 'CopyRspackPlugin',
  HtmlRspackPlugin = 'HtmlRspackPlugin',
  SwcJsMinimizerRspackPlugin = 'SwcJsMinimizerRspackPlugin',
  SwcCssMinimizerRspackPlugin = 'SwcCssMinimizerRspackPlugin',
  BundlerInfoRspackPlugin = 'BundlerInfoRspackPlugin',
  JsLoaderRspackPlugin = 'JsLoaderRspackPlugin'
}

export function cleanupGlobalTrace(): void

export interface JsAfterResolveData {
  request: string
  context: string
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  createData?: JsCreateData
}

export interface JsAsset {
  name: string
  info: JsAssetInfo
}

export interface JsAssetEmittedArgs {
  filename: string
  outputPath: string
  targetPath: string
}

export interface JsAssetInfo {
  /** if the asset can be long term cached forever (contains a hash) */
  immutable: boolean
  /** whether the asset is minimized */
  minimized: boolean
  /**
   * the value(s) of the full hash used for this asset
   * the value(s) of the chunk hash used for this asset
   */
  chunkHash: Array<string>
  /**
   * the value(s) of the module hash used for this asset
   * the value(s) of the content hash used for this asset
   */
  contentHash: Array<string>
  sourceFilename?: string
  /**
   * size in bytes, only set after asset has been emitted
   * when asset is only used for development and doesn't count towards user-facing assets
   */
  development: boolean
  /** when asset ships data for updating an existing application (HMR) */
  hotModuleReplacement: boolean
  /** when asset is javascript and an ESM */
  javascriptModule?: boolean
  /** related object to other assets, keyed by type of relation (only points from parent to child) */
  related: JsAssetInfoRelated
}

export interface JsAssetInfoRelated {
  sourceMap?: string
}

export interface JsBeforeResolveArgs {
  request: string
  context: string
}

export interface JsBuildTimeExecutionOption {
  publicPath?: string
  baseUri?: string
}

export interface JsChunk {
  __inner_ukey: number
  __inner_groups: Array<number>
  name?: string
  id?: string
  ids: Array<string>
  idNameHints: Array<string>
  filenameTemplate?: string
  cssFilenameTemplate?: string
  files: Array<string>
  runtime: Array<string>
  hash?: string
  contentHash: Record<string, string>
  renderedHash?: string
  chunkReasons: Array<string>
  auxiliaryFiles: Array<string>
}

export interface JsChunkAssetArgs {
  chunk: JsChunk
  filename: string
}

export interface JsChunkGroup {
  __inner_parents: Array<number>
  __inner_ukey: number
  chunks: Array<JsChunk>
  index?: number
  name?: string
}

export interface JsChunkPathData {
  id?: string
  name?: string
  hash?: string
  contentHash?: Record<string, string>
}

export interface JsCodegenerationResult {
  sources: Record<string, string>
}

export interface JsCodegenerationResults {
  map: Record<string, Record<string, JsCodegenerationResult>>
}

export interface JsCompatSource {
  /** Whether the underlying data structure is a `RawSource` */
  isRaw: boolean
  /** Whether the underlying value is a buffer or string */
  isBuffer: boolean
  source: Buffer
  map?: Buffer
}

export interface JsCreateData {
  request: string
  userRequest: string
  resource: string
}

export interface JsExecuteModuleArg {
  entry: string
  runtimeModules: Array<string>
  codegenResults: JsCodegenerationResults
  id: number
}

export interface JsExecuteModuleResult {
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  buildDependencies: Array<string>
  missingDependencies: Array<string>
  assets: Array<string>
  id: number
}

export interface JsHooks {
  normalModuleFactoryResolveForScheme: (data: JsResolveForSchemeInput) => Promise<JsResolveForSchemeResult>
}

export interface JsLoaderContext {
  /** Content maybe empty in pitching stage */
  content: null | Buffer
  additionalData?: Buffer
  sourceMap?: Buffer
  resource: string
  resourcePath: string
  resourceQuery?: string
  resourceFragment?: string
  cacheable: boolean
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  buildDependencies: Array<string>
  assetFilenames: Array<string>
  currentLoader: string
  isPitching: boolean
  /**
   * Loader index from JS.
   * If loaders are dispatched by JS loader runner,
   * then, this field is correspondence with loader index in JS side.
   * It is useful when loader dispatched on JS side has an builtin loader, for example: builtin:swc-loader,
   * Then this field will be used as an hack to test whether it should return an AST or string.
   */
  loaderIndexFromJs?: number
  /**
   * Internal additional data, contains more than `String`
   * @internal
   */
  additionalDataExternal: ExternalObject<'AdditionalData'>
  /**
   * Internal loader context
   * @internal
   */
  contextExternal: ExternalObject<'LoaderRunnerContext'>
  /**
   * Internal loader diagnostic
   * @internal
   */
  diagnosticsExternal: ExternalObject<'Diagnostic[]'>
  _moduleIdentifier: string
  hot: boolean
}

/** Only for dts generation */
export interface JsLoaderResult {
  /** Content in pitching stage can be empty */
  content?: Buffer
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  buildDependencies: Array<string>
  assetFilenames: Array<string>
  sourceMap?: Buffer
  additionalData?: Buffer
  additionalDataExternal: ExternalObject<'AdditionalData'>
  cacheable: boolean
  /** Used to instruct how rust loaders should execute */
  isPitching: boolean
}

export interface JsModule {
  context?: string
  originalSource?: JsCompatSource
  resource?: string
  moduleIdentifier: string
  nameForCondition?: string
}

export interface JsNormalModuleFactoryCreateModuleArgs {
  dependencyType: string
  rawRequest: string
  resourceResolveData: JsResourceData
  context: string
  matchResource?: string
}

export interface JsResolveForSchemeInput {
  resourceData: JsResourceData
  scheme: string
}

export interface JsResolveForSchemeResult {
  resourceData: JsResourceData
  stop: boolean
}

export interface JsResourceData {
  /** Resource with absolute path, query and fragment */
  resource: string
  /** Absolute resource path only */
  path: string
  /** Resource query with `?` prefix */
  query?: string
  /** Resource fragment with `#` prefix */
  fragment?: string
}

export interface JsRuntimeModule {
  source?: JsCompatSource
  moduleIdentifier: string
  constructorName: string
  name: string
}

export interface JsRuntimeModuleArg {
  module: JsRuntimeModule
  chunk: JsChunk
}

export interface JsStatsAsset {
  type: string
  name: string
  size: number
  chunks: Array<string | undefined | null>
  chunkNames: Array<string>
  info: JsStatsAssetInfo
  emitted: boolean
}

export interface JsStatsAssetInfo {
  development: boolean
  hotModuleReplacement: boolean
  sourceFilename?: string
}

export interface JsStatsAssetsByChunkName {
  name: string
  files: Array<string>
}

export interface JsStatsChunk {
  type: string
  files: Array<string>
  auxiliaryFiles: Array<string>
  id?: string
  entry: boolean
  initial: boolean
  names: Array<string>
  size: number
  modules?: Array<JsStatsModule>
  parents?: Array<string>
  children?: Array<string>
  siblings?: Array<string>
}

export interface JsStatsChunkGroup {
  name: string
  assets: Array<JsStatsChunkGroupAsset>
  chunks: Array<string | undefined | null>
  assetsSize: number
}

export interface JsStatsChunkGroupAsset {
  name: string
  size: number
}

export interface JsStatsError {
  message: string
  formatted: string
  moduleIdentifier?: string
  moduleName?: string
  moduleId?: string
}

export interface JsStatsGetAssets {
  assets: Array<JsStatsAsset>
  assetsByChunkName: Array<JsStatsAssetsByChunkName>
}

export interface JsStatsLogging {
  name: string
  type: string
  args?: Array<string>
  trace?: Array<string>
}

export interface JsStatsMillisecond {
  secs: number
  subsecMillis: number
}

export interface JsStatsModule {
  type: string
  moduleType: string
  identifier: string
  name: string
  id?: string
  chunks: Array<string | undefined | null>
  size: number
  issuer?: string
  issuerName?: string
  issuerId?: string
  issuerPath: Array<JsStatsModuleIssuer>
  nameForCondition?: string
  reasons?: Array<JsStatsModuleReason>
  assets?: Array<string>
  source?: string | Buffer
  profile?: JsStatsModuleProfile
  orphan: boolean
  providedExports?: Array<string>
  usedExports?: string | Array<string>
  optimizationBailout?: Array<string>
}

export interface JsStatsModuleIssuer {
  identifier: string
  name: string
  id?: string
}

export interface JsStatsModuleProfile {
  factory: JsStatsMillisecond
  integration: JsStatsMillisecond
  building: JsStatsMillisecond
}

export interface JsStatsModuleReason {
  moduleIdentifier?: string
  moduleName?: string
  moduleId?: string
  type?: string
  userRequest?: string
}

export interface JsStatsOptimizationBailout {
  inner: string
}

export interface JsStatsWarning {
  message: string
  formatted: string
  moduleIdentifier?: string
  moduleName?: string
  moduleId?: string
}

export interface JsTap {
  function: (...args: any[]) => any
  stage: number
}

export interface NodeFS {
  writeFile: (...args: any[]) => any
  removeFile: (...args: any[]) => any
  mkdir: (...args: any[]) => any
  mkdirp: (...args: any[]) => any
}

export interface PathData {
  filename?: string
  hash?: string
  contentHash?: string
  runtime?: string
  url?: string
  id?: string
  chunk?: JsChunkPathData
}

export interface PathWithInfo {
  path: string
  info: JsAssetInfo
}

export interface RawAliasOptionItem {
  path: string
  redirect: Array<string | false>
}

export interface RawAssetGeneratorDataUrlFnArgs {
  filename: string
  content: string
}

export interface RawAssetGeneratorDataUrlOptions {
  encoding?: "base64" | "false" | undefined
  mimetype?: string
}

export interface RawAssetGeneratorOptions {
  filename?: string
  publicPath?: string
  dataUrl?: RawAssetGeneratorDataUrlOptions | ((arg: RawAssetGeneratorDataUrlFnArgs) => string)
}

export interface RawAssetInlineGeneratorOptions {
  dataUrl?: RawAssetGeneratorDataUrlOptions | ((arg: RawAssetGeneratorDataUrlFnArgs) => string)
}

export interface RawAssetParserDataUrl {
  type: "options"
  options?: RawAssetParserDataUrlOptions
}

export interface RawAssetParserDataUrlOptions {
  maxSize?: number
}

export interface RawAssetParserOptions {
  dataUrlCondition?: RawAssetParserDataUrl
}

export interface RawAssetResourceGeneratorOptions {
  filename?: string
  publicPath?: string
}

export interface RawBannerContentFnCtx {
  hash: string
  chunk: JsChunk
  filename: string
}

export interface RawBannerPluginOptions {
  banner: string | ((...args: any[]) => any)
  entryOnly?: boolean
  footer?: boolean
  raw?: boolean
  test?: string | RegExp | (string | RegExp)[]
  include?: string | RegExp | (string | RegExp)[]
  exclude?: string | RegExp | (string | RegExp)[]
}

export interface RawBuiltins {
  css?: RawCssPluginConfig
  treeShaking: string
}

export interface RawBundlerInfoPluginOptions {
  version: string
  force: boolean | string[]
}

export interface RawCacheGroupOptions {
  key: string
  priority?: number
  test?: RegExp | string | Function
  filename?: string
  idHint?: string
  /** What kind of chunks should be selected. */
  chunks?: RegExp | 'async' | 'initial' | 'all'
  type?: RegExp | string
  automaticNameDelimiter?: string
  minChunks?: number
  minSize?: number
  maxSize?: number
  maxAsyncSize?: number
  maxInitialSize?: number
  name?: string | false | Function
  reuseExistingChunk?: boolean
  enforce?: boolean
}

export interface RawCacheGroupTestCtx {
  module: JsModule
}

export interface RawCacheOptions {
  type: string
  maxGenerations: number
  maxAge: number
  profile: boolean
  buildDependencies: Array<string>
  cacheDirectory: string
  cacheLocation: string
  name: string
  version: string
}

export interface RawChunkOptionNameCtx {
  module: JsModule
}

export interface RawConsumeOptions {
  key: string
  import?: string
  importResolved?: string
  shareKey: string
  shareScope: string
  requiredVersion?: string | false | undefined
  packageName?: string
  strictVersion: boolean
  singleton: boolean
  eager: boolean
}

export interface RawConsumeSharedPluginOptions {
  consumes: Array<RawConsumeOptions>
  enhanced: boolean
}

export interface RawContainerPluginOptions {
  name: string
  shareScope: string
  library: RawLibraryOptions
  runtime?: string
  filename?: string
  exposes: Array<RawExposeOptions>
  enhanced: boolean
}

export interface RawContainerReferencePluginOptions {
  remoteType: string
  remotes: Array<RawRemoteOptions>
  shareScope?: string
  enhanced: boolean
}

export interface RawCopyGlobOptions {
  caseSensitiveMatch?: boolean
  dot?: boolean
  ignore?: Array<string>
}

export interface RawCopyPattern {
  from: string
  to?: string
  context?: string
  toType?: string
  noErrorOnMissing: boolean
  force: boolean
  priority: number
  globOptions: RawCopyGlobOptions
  info?: RawInfo
}

export interface RawCopyRspackPluginOptions {
  patterns: Array<RawCopyPattern>
}

export interface RawCrossOriginLoading {
  type: "bool" | "string"
  stringPayload?: string
  boolPayload?: boolean
}

export interface RawCssModulesConfig {
  localsConvention: "asIs" | "camelCase" | "camelCaseOnly" | "dashes" | "dashesOnly"
  localIdentName: string
  exportsOnly: boolean
}

export interface RawCssPluginConfig {
  modules: RawCssModulesConfig
  namedExports?: boolean
}

export interface RawEntryOptions {
  name?: string
  runtime?: string
  chunkLoading?: string
  asyncChunks?: boolean
  publicPath?: string
  baseUri?: string
  filename?: string
  library?: RawLibraryOptions
}

export interface RawEntryPluginOptions {
  context: string
  entry: string
  options: RawEntryOptions
}

export interface RawEvalDevToolModulePluginOptions {
  namespace?: string
  moduleFilenameTemplate?: string | ((info: RawModuleFilenameTemplateFnCtx) => string)
  sourceUrlComment?: string
}

export interface RawExperiments {
  newSplitChunks: boolean
  topLevelAwait: boolean
  rspackFuture: RawRspackFuture
}

export interface RawExposeOptions {
  key: string
  name?: string
  import: Array<string>
}

export interface RawExternalItemFnCtx {
  request: string
  context: string
  dependencyType: string
}

export interface RawExternalItemFnResult {
  externalType?: string
  result?: string | boolean | string[] | Record<string, string[]>
}

export interface RawExternalsPluginOptions {
  type: string
  externals: (string | RegExp | Record<string, string | boolean | string[] | Record<string, string[]>> | ((...args: any[]) => any))[]
}

export interface RawExternalsPresets {
  node: boolean
  web: boolean
  electron: boolean
  electronMain: boolean
  electronPreload: boolean
  electronRenderer: boolean
}

export interface RawExtractComments {
  banner?: string | boolean
  condition?: string
}

export interface RawFallbackCacheGroupOptions {
  chunks?: RegExp | 'async' | 'initial' | 'all'
  minSize?: number
  maxSize?: number
  maxAsyncSize?: number
  maxInitialSize?: number
  automaticNameDelimiter?: string
}

export interface RawFuncUseCtx {
  resource?: string
  realResource?: string
  resourceQuery?: string
  issuer?: string
}

export interface RawGeneratorOptions {
  type: "asset" | "asset/inline" | "asset/resource" | "unknown"
  asset?: RawAssetGeneratorOptions
  assetInline?: RawAssetInlineGeneratorOptions
  assetResource?: RawAssetResourceGeneratorOptions
}

export interface RawHtmlRspackPluginOptions {
  /** emitted file name in output path */
  filename?: string
  /** template html file */
  template?: string
  templateContent?: string
  templateParameters?: Record<string, string>
  /** "head", "body" or "false" */
  inject: "head" | "body" | "false"
  /** path or `auto` */
  publicPath?: string
  /** `blocking`, `defer`, or `module` */
  scriptLoading: "blocking" | "defer" | "module"
  /** entry_chunk_name (only entry chunks are supported) */
  chunks?: Array<string>
  excludedChunks?: Array<string>
  sri?: "sha256" | "sha384" | "sha512"
  minify?: boolean
  title?: string
  favicon?: string
  meta?: Record<string, Record<string, string>>
}

export interface RawHttpExternalsRspackPluginOptions {
  css: boolean
  webAsync: boolean
}

export interface RawInfo {
  immutable?: boolean
  minimized?: boolean
  chunkHash?: Array<string>
  contentHash?: Array<string>
  development?: boolean
  hotModuleReplacement?: boolean
  related?: RawRelated
  version?: string
}

export interface RawJavascriptParserOptions {
  dynamicImportMode: string
  dynamicImportPreload: string
  dynamicImportPrefetch: string
  url: string
}

export interface RawLibraryAuxiliaryComment {
  root?: string
  commonjs?: string
  commonjs2?: string
  amd?: string
}

export interface RawLibraryCustomUmdObject {
  amd?: string
  commonjs?: string
  root?: Array<string>
}

export interface RawLibraryName {
  type: "string" | "array" | "umdObject"
  stringPayload?: string
  arrayPayload?: Array<string>
  umdObjectPayload?: RawLibraryCustomUmdObject
}

export interface RawLibraryOptions {
  name?: RawLibraryName
  export?: Array<string>
  libraryType: string
  umdNamedDefine?: boolean
  auxiliaryComment?: RawLibraryAuxiliaryComment
  amdContainer?: string
}

export interface RawLimitChunkCountPluginOptions {
  chunkOverhead?: number
  entryChunkMultiplicator?: number
  maxChunks: number
}

export interface RawModuleFilenameTemplateFnCtx {
  identifier: string
  shortIdentifier: string
  resource: string
  resourcePath: string
  absoluteResourcePath: string
  loaders: string
  allLoaders: string
  query: string
  moduleId: string
  hash: string
  namespace: string
}

export interface RawModuleOptions {
  rules: Array<RawModuleRule>
  parser?: Record<string, RawParserOptions>
  generator?: Record<string, RawGeneratorOptions>
  noParse?: string | RegExp | ((request: string) => boolean) | (string | RegExp | ((request: string) => boolean))[]
}

export interface RawModuleRule {
  /**
   * A conditional match matching an absolute path + query + fragment.
   * Note:
   *   This is a custom matching rule not initially designed by webpack.
   *   Only for single-threaded environment interoperation purpose.
   */
  rspackResource?: RawRuleSetCondition
  /** A condition matcher matching an absolute path. */
  test?: RawRuleSetCondition
  include?: RawRuleSetCondition
  exclude?: RawRuleSetCondition
  /** A condition matcher matching an absolute path. */
  resource?: RawRuleSetCondition
  /** A condition matcher against the resource query. */
  resourceQuery?: RawRuleSetCondition
  resourceFragment?: RawRuleSetCondition
  descriptionData?: Record<string, RawRuleSetCondition>
  sideEffects?: boolean
  use?: RawModuleRuleUses
  type?: string
  parser?: RawParserOptions
  generator?: RawGeneratorOptions
  resolve?: RawResolveOptions
  issuer?: RawRuleSetCondition
  dependency?: RawRuleSetCondition
  scheme?: RawRuleSetCondition
  mimetype?: RawRuleSetCondition
  oneOf?: Array<RawModuleRule>
  rules?: Array<RawModuleRule>
  /** Specifies the category of the loader. No value means normal loader. */
  enforce?: 'pre' | 'post'
}

/**
 * `loader` is for both JS and Rust loaders.
 * `options` is
 *   - a `None` on rust side and handled by js side `getOptions` when
 * using with `loader`.
 *   - a `Some(string)` on rust side, deserialized by `serde_json::from_str`
 * and passed to rust side loader in [get_builtin_loader] when using with
 * `builtin_loader`.
 */
export interface RawModuleRuleUse {
  loader: string
  options?: string
}

export interface RawModuleRuleUses {
  type: "array" | "function"
  arrayUse?: Array<RawModuleRuleUse>
  funcUse?: (arg: RawFuncUseCtx) => RawModuleRuleUse[]
}

export interface RawNodeOption {
  dirname: string
  filename: string
  global: string
}

export interface RawOptimizationOptions {
  removeAvailableModules: boolean
  sideEffects: string
  usedExports: string
  providedExports: boolean
  innerGraph: boolean
  mangleExports: string
  concatenateModules: boolean
}

export interface RawOptions {
  mode?: undefined | 'production' | 'development' | 'none'
  target: Array<string>
  context: string
  output: RawOutputOptions
  resolve: RawResolveOptions
  resolveLoader: RawResolveOptions
  module: RawModuleOptions
  devtool: string
  optimization: RawOptimizationOptions
  stats: RawStatsOptions
  snapshot: RawSnapshotOptions
  cache: RawCacheOptions
  experiments: RawExperiments
  node?: RawNodeOption
  profile: boolean
  bail: boolean
  builtins: RawBuiltins
}

export interface RawOutputOptions {
  path: string
  clean: boolean
  publicPath: string
  assetModuleFilename: string
  wasmLoading: string
  enabledWasmLoadingTypes: Array<string>
  webassemblyModuleFilename: string
  filename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string)
  chunkFilename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string)
  crossOriginLoading: RawCrossOriginLoading
  cssFilename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string)
  cssChunkFilename: string | ((pathData: PathData, assetInfo?: JsAssetInfo) => string)
  hotUpdateMainFilename: string
  hotUpdateChunkFilename: string
  hotUpdateGlobal: string
  uniqueName: string
  chunkLoadingGlobal: string
  library?: RawLibraryOptions
  strictModuleErrorHandling: boolean
  enabledLibraryTypes?: Array<string>
  globalObject: string
  importFunctionName: string
  iife: boolean
  module: boolean
  chunkLoading: string
  enabledChunkLoadingTypes?: Array<string>
  trustedTypes?: RawTrustedTypes
  sourceMapFilename: string
  hashFunction: string
  hashDigest: string
  hashDigestLength: number
  hashSalt?: string
  asyncChunks: boolean
  workerChunkLoading: string
  workerWasmLoading: string
  workerPublicPath: string
  scriptType: "module" | "text/javascript" | "false"
}

export interface RawParserOptions {
  type: "asset" | "javascript" | "unknown"
  asset?: RawAssetParserOptions
  javascript?: RawJavascriptParserOptions
}

export interface RawPathData {
  filename?: string
  contentHash?: string
  url?: string
}

export interface RawPluginImportConfig {
  libraryName: string
  libraryDirectory?: string
  customName?: string
  customStyleName?: string
  style?: RawStyleConfig
  camelToDashComponentName?: boolean
  transformToDefaultImport?: boolean
  ignoreEsComponent?: Array<string>
  ignoreStyleComponent?: Array<string>
}

export interface RawProgressPluginOptions {
  prefix: string
  profile: boolean
}

export interface RawProvideOptions {
  key: string
  shareKey: string
  shareScope: string
  version?: string | false | undefined
  eager: boolean
}

export interface RawReactOptions {
  runtime?: "automatic" | "classic"
  importSource?: string
  pragma?: string
  pragmaFrag?: string
  throwIfNamespace?: boolean
  development?: boolean
  useBuiltins?: boolean
  useSpread?: boolean
  refresh?: boolean
}

export interface RawRegexMatcher {
  source: string
  flags: string
}

export interface RawRelated {
  sourceMap?: string
}

export interface RawRelayConfig {
  artifactDirectory?: string
  language: 'javascript' | 'typescript' | 'flow'
}

export interface RawRemoteOptions {
  key: string
  external: Array<string>
  shareScope: string
}

export interface RawResolveOptions {
  preferRelative?: boolean
  preferAbsolute?: boolean
  extensions?: Array<string>
  mainFiles?: Array<string>
  mainFields?: Array<string>
  conditionNames?: Array<string>
  alias?: Array<RawAliasOptionItem>
  fallback?: Array<RawAliasOptionItem>
  symlinks?: boolean
  tsconfig?: RawResolveTsconfigOptions
  modules?: Array<string>
  byDependency?: Record<string, RawResolveOptions>
  fullySpecified?: boolean
  exportsFields?: Array<string>
  extensionAlias?: Record<string, Array<string>>
  aliasFields?: Array<string>
  restrictions?: Array<string>
  roots?: Array<string>
}

export interface RawResolveTsconfigOptions {
  configFile: string
  referencesType: "auto" | "manual" | "disabled"
  references?: Array<string>
}

export interface RawRspackFuture {
  newTreeshaking: boolean
}

export interface RawRuleSetCondition {
  type: "string" | "regexp" | "logical" | "array" | "function"
  stringMatcher?: string
  regexpMatcher?: RawRegexMatcher
  logicalMatcher?: Array<RawRuleSetLogicalConditions>
  arrayMatcher?: Array<RawRuleSetCondition>
  funcMatcher?: (value: string) => boolean
}

export interface RawRuleSetLogicalConditions {
  and?: Array<RawRuleSetCondition>
  or?: Array<RawRuleSetCondition>
  not?: RawRuleSetCondition
}

export interface RawSnapshotOptions {
  resolve: RawSnapshotStrategy
  module: RawSnapshotStrategy
}

export interface RawSnapshotStrategy {
  hash: boolean
  timestamp: boolean
}

export interface RawSourceMapDevToolPluginOptions {
  append?: (false | null) | string | Function
  columns?: boolean
  fallbackModuleFilenameTemplate?: string | ((info: RawModuleFilenameTemplateFnCtx) => string)
  fileContext?: string
  filename?: (false | null) | string
  module?: boolean
  moduleFilenameTemplate?: string | ((info: RawModuleFilenameTemplateFnCtx) => string)
  namespace?: string
  noSources?: boolean
  publicPath?: string
  sourceRoot?: string
  test?: (text: string) => boolean
}

export interface RawSplitChunksOptions {
  fallbackCacheGroup?: RawFallbackCacheGroupOptions
  name?: string | false | Function
  cacheGroups?: Array<RawCacheGroupOptions>
  /** What kind of chunks should be selected. */
  chunks?: RegExp | 'async' | 'initial' | 'all' | Function
  automaticNameDelimiter?: string
  maxAsyncRequests?: number
  maxInitialRequests?: number
  minChunks?: number
  hidePathInfo?: boolean
  minSize?: number
  enforceSizeThreshold?: number
  minRemainingSize?: number
  maxSize?: number
  maxAsyncSize?: number
  maxInitialSize?: number
}

export interface RawStatsOptions {
  colors: boolean
}

export interface RawStyleConfig {
  styleLibraryDirectory?: string
  custom?: string
  css?: string
  bool?: boolean
}

export interface RawSwcJsMinimizerRspackPluginOptions {
  extractComments?: RawExtractComments
  compress: boolean | string
  mangle: boolean | string
  format: string
  module?: boolean
  test?: string | RegExp | (string | RegExp)[]
  include?: string | RegExp | (string | RegExp)[]
  exclude?: string | RegExp | (string | RegExp)[]
}

export interface RawTrustedTypes {
  policyName?: string
}

/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/d1d0607158ab40463d1b123fed52cc526eba8385/bindings/binding_core_node/src/util.rs#L29-L58
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
export function registerGlobalTrace(filter: string, layer: "chrome" | "logger", output: string): void

export enum RegisterJsTapKind {
  CompilerThisCompilation = 0,
  CompilerCompilation = 1,
  CompilerMake = 2,
  CompilerFinishMake = 3,
  CompilerShouldEmit = 4,
  CompilerEmit = 5,
  CompilerAfterEmit = 6,
  CompilerAssetEmitted = 7,
  CompilationBuildModule = 8,
  CompilationStillValidModule = 9,
  CompilationSucceedModule = 10,
  CompilationExecuteModule = 11,
  CompilationFinishModules = 12,
  CompilationOptimizeModules = 13,
  CompilationAfterOptimizeModules = 14,
  CompilationOptimizeTree = 15,
  CompilationOptimizeChunkModules = 16,
  CompilationRuntimeModule = 17,
  CompilationChunkAsset = 18,
  CompilationProcessAssets = 19,
  CompilationAfterProcessAssets = 20,
  NormalModuleFactoryBeforeResolve = 21,
  NormalModuleFactoryAfterResolve = 22,
  NormalModuleFactoryCreateModule = 23,
  ContextModuleFactoryBeforeResolve = 24,
  ContextModuleFactoryAfterResolve = 25
}

export interface RegisterJsTaps {
  registerCompilerThisCompilationTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>
  registerCompilerCompilationTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>
  registerCompilerMakeTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>
  registerCompilerFinishMakeTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>
  registerCompilerShouldEmitTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => boolean | undefined); stage: number; }>
  registerCompilerEmitTaps: (stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>
  registerCompilerAfterEmitTaps: (stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>
  registerCompilerAssetEmittedTaps: (stages: Array<number>) => Array<{ function: ((arg: JsAssetEmittedArgs) => Promise<void>); stage: number; }>
  registerCompilationBuildModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>
  registerCompilationStillValidModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>
  registerCompilationSucceedModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsModule) => void); stage: number; }>
  registerCompilationExecuteModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsExecuteModuleArg) => void); stage: number; }>
  registerCompilationRuntimeModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsRuntimeModuleArg) => JsRuntimeModule | undefined); stage: number; }>
  registerCompilationFinishModulesTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>
  registerCompilationOptimizeModulesTaps: (stages: Array<number>) => Array<{ function: (() => boolean | undefined); stage: number; }>
  registerCompilationAfterOptimizeModulesTaps: (stages: Array<number>) => Array<{ function: (() => void); stage: number; }>
  registerCompilationOptimizeTreeTaps: (stages: Array<number>) => Array<{ function: (() => Promise<void>); stage: number; }>
  registerCompilationOptimizeChunkModulesTaps: (stages: Array<number>) => Array<{ function: (() => Promise<boolean | undefined>); stage: number; }>
  registerCompilationChunkAssetTaps: (stages: Array<number>) => Array<{ function: ((arg: JsChunkAssetArgs) => void); stage: number; }>
  registerCompilationProcessAssetsTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => Promise<void>); stage: number; }>
  registerCompilationAfterProcessAssetsTaps: (stages: Array<number>) => Array<{ function: ((arg: JsCompilation) => void); stage: number; }>
  registerNormalModuleFactoryBeforeResolveTaps: (stages: Array<number>) => Array<{ function: ((arg: JsBeforeResolveArgs) => Promise<[boolean | undefined, JsBeforeResolveArgs]>); stage: number; }>
  registerNormalModuleFactoryAfterResolveTaps: (stages: Array<number>) => Array<{ function: ((arg: JsAfterResolveData) => Promise<[boolean | undefined, JsCreateData | undefined]>); stage: number; }>
  registerNormalModuleFactoryCreateModuleTaps: (stages: Array<number>) => Array<{ function: ((arg: JsNormalModuleFactoryCreateModuleArgs) => Promise<void>); stage: number; }>
  registerContextModuleFactoryBeforeResolveTaps: (stages: Array<number>) => Array<{ function: ((arg: JsBeforeResolveArgs) => Promise<[boolean | undefined, JsBeforeResolveArgs]>); stage: number; }>
  registerContextModuleFactoryAfterResolveTaps: (stages: Array<number>) => Array<{ function: ((arg: JsAfterResolveData) => Promise<boolean | undefined>); stage: number; }>
}

/** Builtin loader runner */
export function runBuiltinLoader(builtin: string, options: string | undefined | null, loaderContext: JsLoaderContext): Promise<JsLoaderContext>

export interface ThreadsafeNodeFS {
  writeFile: (name: string, content: Buffer) => void
  removeFile: (name: string) => void
  mkdir: (name: string) => void
  mkdirp: (name: string) => string | void
  removeDirAll: (name: string) => string | void
}

