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
  getChunks(): Array<JsChunk>
  /**
   * Only available for those none Js and Css source,
   * return true if set module source successfully, false if failed.
   */
  setNoneAstModuleSource(moduleIdentifier: string, source: JsCompatSource): boolean
  setAssetSource(name: string, source: JsCompatSource): void
  deleteAssetSource(name: string): void
  getAssetFilenames(): Array<string>
  hasAsset(name: string): boolean
  emitAsset(filename: string, source: JsCompatSource, assetInfo: JsAssetInfo): void
  deleteAsset(filename: string): void
  get entrypoints(): Record<string, JsChunkGroup>
  get hash(): string | null
  getFileDependencies(): Array<string>
  getContextDependencies(): Array<string>
  getMissingDependencies(): Array<string>
  getBuildDependencies(): Array<string>
  pushDiagnostic(severity: "error" | "warning", title: string, message: string): void
  pushNativeDiagnostics(diagnostics: ExternalObject<Array<Diagnostic>>): void
  getStats(): JsStats
  getAssetPath(filename: string, data: PathData): string
  getAssetPathWithInfo(filename: string, data: PathData): PathWithInfo
  getPath(filename: string, data: PathData): string
  getPathWithInfo(filename: string, data: PathData): PathWithInfo
  addFileDependencies(deps: Array<string>): void
  addContextDependencies(deps: Array<string>): void
  addMissingDependencies(deps: Array<string>): void
  addBuildDependencies(deps: Array<string>): void
  rebuildModule(moduleIdentifiers: Array<string>, f: (...args: any[]) => any): void
}

export class JsStats {
  getAssets(): JsStatsGetAssets
  getModules(reasons: boolean, moduleAssets: boolean, nestedModules: boolean, source: boolean): Array<JsStatsModule>
  getChunks(chunkModules: boolean, chunksRelations: boolean, reasons: boolean, moduleAssets: boolean, nestedModules: boolean, source: boolean): Array<JsStatsChunk>
  getEntrypoints(): Array<JsStatsChunkGroup>
  getNamedChunkGroups(): Array<JsStatsChunkGroup>
  getErrors(): Array<JsStatsError>
  getWarnings(): Array<JsStatsWarning>
  getLogging(acceptedTypes: number): Array<JsStatsLogging>
  getHash(): string
}

export class Rspack {
  constructor(options: RawOptions, builtinPlugins: Array<BuiltinPlugin>, jsHooks: JsHooks | undefined | null, outputFilesystem: ThreadsafeNodeFS, jsLoaderRunner: (...args: any[]) => any)
  unsafe_set_disabled_hooks(hooks: Array<string>): void
  /**
   * Build with the given option passed to the constructor
   *
   * Warning:
   * Calling this method recursively might cause a deadlock.
   */
  unsafe_build(callback: (err: null | Error) => void): void
  /**
   * Rebuild with the given option passed to the constructor
   *
   * Warning:
   * Calling this method recursively will cause a deadlock.
   */
  unsafe_rebuild(changed_files: string[], removed_files: string[], callback: (err: null | Error) => void): void
  /**
   * Get the last compilation
   *
   * Warning:
   *
   * Calling this method under the build or rebuild method might cause a deadlock.
   *
   * **Note** that this method is not safe if you cache the _JsCompilation_ on the Node side, as it will be invalidated by the next build and accessing a dangling ptr is a UB.
   */
  unsafe_last_compilation(f: (arg0: JsCompilation) => void): void
  /**
   * Destroy the compiler
   *
   * Warning:
   *
   * Anything related to this compiler will be invalidated after this method is called.
   */
  unsafe_drop(): void
}

export interface AfterResolveData {
  request: string
  context: string
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  factoryMeta: FactoryMeta
}

export interface BeforeResolveData {
  request: string
  context: string
}

export interface BuiltinPlugin {
  name: BuiltinPluginName
  options: unknown
}

export const enum BuiltinPluginName {
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
  CommonJsChunkFormatPlugin = 'CommonJsChunkFormatPlugin',
  ArrayPushCallbackChunkFormatPlugin = 'ArrayPushCallbackChunkFormatPlugin',
  ModuleChunkFormatPlugin = 'ModuleChunkFormatPlugin',
  HttpExternalsRspackPlugin = 'HttpExternalsRspackPlugin',
  CopyRspackPlugin = 'CopyRspackPlugin',
  HtmlRspackPlugin = 'HtmlRspackPlugin',
  SwcJsMinimizerRspackPlugin = 'SwcJsMinimizerRspackPlugin',
  SwcCssMinimizerRspackPlugin = 'SwcCssMinimizerRspackPlugin'
}

export function cleanupGlobalTrace(): void

export interface FactoryMeta {
  sideEffectFree?: boolean
}

export interface JsAsset {
  name: string
  source?: JsCompatSource
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
  /**
   * when asset was created from a source file (potentially transformed), the original filename relative to compilation context
   * size in bytes, only set after asset has been emitted
   * when asset is only used for development and doesn't count towards user-facing assets
   */
  development: boolean
  /** when asset ships data for updating an existing application (HMR) */
  hotModuleReplacement: boolean
  /**
   * when asset is javascript and an ESM
   * related object to other assets, keyed by type of relation (only points from parent to child)
   */
  related: JsAssetInfoRelated
  /**
   * the asset version, emit can be skipped when both filename and version are the same
   * An empty string means no version, it will always emit
   */
  version: string
}

export interface JsAssetInfoRelated {
  sourceMap?: string
}

export interface JsChunk {
  name?: string
  files: Array<string>
}

export interface JsChunkAssetArgs {
  chunk: JsChunk
  filename: string
}

export interface JsChunkGroup {
  chunks: Array<JsChunk>
}

export interface JsCompatSource {
  /** Whether the underlying data structure is a `RawSource` */
  isRaw: boolean
  /** Whether the underlying value is a buffer or string */
  isBuffer: boolean
  source: Buffer
  map?: Buffer
}

export interface JsHooks {
  processAssetsStageAdditional: (...args: any[]) => any
  processAssetsStagePreProcess: (...args: any[]) => any
  processAssetsStageDerived: (...args: any[]) => any
  processAssetsStageAdditions: (...args: any[]) => any
  processAssetsStageNone: (...args: any[]) => any
  processAssetsStageOptimize: (...args: any[]) => any
  processAssetsStageOptimizeCount: (...args: any[]) => any
  processAssetsStageOptimizeCompatibility: (...args: any[]) => any
  processAssetsStageOptimizeSize: (...args: any[]) => any
  processAssetsStageDevTooling: (...args: any[]) => any
  processAssetsStageOptimizeInline: (...args: any[]) => any
  processAssetsStageSummarize: (...args: any[]) => any
  processAssetsStageOptimizeHash: (...args: any[]) => any
  processAssetsStageOptimizeTransfer: (...args: any[]) => any
  processAssetsStageAnalyse: (...args: any[]) => any
  processAssetsStageReport: (...args: any[]) => any
  compilation: (...args: any[]) => any
  thisCompilation: (...args: any[]) => any
  emit: (...args: any[]) => any
  assetEmitted: (...args: any[]) => any
  afterEmit: (...args: any[]) => any
  make: (...args: any[]) => any
  optimizeModules: (...args: any[]) => any
  optimizeTree: (...args: any[]) => any
  optimizeChunkModule: (...args: any[]) => any
  beforeCompile: (...args: any[]) => any
  afterCompile: (...args: any[]) => any
  finishModules: (...args: any[]) => any
  finishMake: (...args: any[]) => any
  buildModule: (...args: any[]) => any
  beforeResolve: (...args: any[]) => any
  afterResolve: (...args: any[]) => any
  contextModuleBeforeResolve: (...args: any[]) => any
  normalModuleFactoryResolveForScheme: (...args: any[]) => any
  chunkAsset: (...args: any[]) => any
  succeedModule: (...args: any[]) => any
  stillValidModule: (...args: any[]) => any
}

export interface JsLoaderContext {
  /** Content maybe empty in pitching stage */
  content?: Buffer
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
   * Internal loader context
   * @internal
   */
  context: ExternalObject<LoaderRunnerContext>
  /**
   * Internal loader diagnostic
   * @internal
   */
  diagnostics: ExternalObject<Array<Diagnostic>>
}

export interface JsLoaderResult {
  /** Content in pitching stage can be empty */
  content?: Buffer
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  buildDependencies: Array<string>
  sourceMap?: Buffer
  additionalData?: Buffer
  cacheable: boolean
  /** Used to instruct how rust loaders should execute */
  isPitching: boolean
}

export interface JsModule {
  originalSource?: JsCompatSource
  resource: string
  moduleIdentifier: string
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

export interface JsStatsAsset {
  type: string
  name: string
  size: number
  chunks: Array<string>
  chunkNames: Array<string>
  info: JsStatsAssetInfo
  emitted: boolean
}

export interface JsStatsAssetInfo {
  development: boolean
  hotModuleReplacement: boolean
}

export interface JsStatsAssetsByChunkName {
  name: string
  files: Array<string>
}

export interface JsStatsChunk {
  type: string
  files: Array<string>
  auxiliaryFiles: Array<string>
  id: string
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
  chunks: Array<string>
  assetsSize: number
}

export interface JsStatsChunkGroupAsset {
  name: string
  size: number
}

export interface JsStatsError {
  message: string
  formatted: string
  title: string
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
  chunks: Array<string>
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

export interface JsStatsWarning {
  message: string
  formatted: string
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
}

export interface PathWithInfo {
  path: string
  info: JsAssetInfo
}

export interface RawAssetGeneratorDataUrl {
  type: "options"
  options?: RawAssetGeneratorDataUrlOptions
}

export interface RawAssetGeneratorDataUrlOptions {
  encoding?: "base64" | "false" | undefined
  mimetype?: string
}

export interface RawAssetGeneratorOptions {
  filename?: string
  publicPath?: string
  dataUrl?: RawAssetGeneratorDataUrl
}

export interface RawAssetInlineGeneratorOptions {
  dataUrl?: RawAssetGeneratorDataUrl
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

export interface RawBannerContent {
  type: "string" | "function"
  stringPayload?: string
  fnPayload?: (...args: any[]) => any
}

export interface RawBannerContentFnCtx {
  hash: string
  chunk: JsChunk
  filename: string
}

export interface RawBannerPluginOptions {
  banner: RawBannerContent
  entryOnly?: boolean
  footer?: boolean
  raw?: boolean
  test?: RawBannerRules
  include?: RawBannerRules
  exclude?: RawBannerRules
}

export interface RawBannerRule {
  type: "string" | "regexp"
  stringMatcher?: string
  regexpMatcher?: string
}

export interface RawBannerRules {
  type: "string" | "regexp" | "array"
  stringMatcher?: string
  regexpMatcher?: string
  arrayMatcher?: Array<RawBannerRule>
}

export interface RawBuiltins {
  css?: RawCssPluginConfig
  presetEnv?: RawPresetEnv
  treeShaking: string
  react: RawReactOptions
  decorator?: RawDecoratorOptions
  noEmitAssets: boolean
  emotion?: string
  devFriendlySplitChunks: boolean
  pluginImport?: Array<RawPluginImportConfig>
  relay?: RawRelayConfig
}

export interface RawCacheGroupOptions {
  priority?: number
  test?: RegExp | string
  idHint?: string
  /** What kind of chunks should be selected. */
  chunks?: RegExp | 'async' | 'initial' | 'all'
  type?: RegExp | string
  minChunks?: number
  minSize?: number
  maxSize?: number
  maxAsyncSize?: number
  maxInitialSize?: number
  name?: string
  reuseExistingChunk?: boolean
  enforce?: boolean
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
}

export interface RawDecoratorOptions {
  legacy: boolean
  emitMetadata: boolean
}

export interface RawDevServer {
  hot: boolean
}

export interface RawEntryOptions {
  name?: string
  runtime?: string
  chunkLoading?: string
  asyncChunks?: boolean
  publicPath?: string
  baseUri?: string
  filename?: string
}

export interface RawEntryPluginOptions {
  context: string
  entry: string
  options: RawEntryOptions
}

export interface RawExperiments {
  lazyCompilation: boolean
  incrementalRebuild: RawIncrementalRebuild
  asyncWebAssembly: boolean
  newSplitChunks: boolean
  css: boolean
  rspackFuture: RawRspackFuture
}

export interface RawExternalItem {
  type: "string" | "regexp" | "object" | "function"
  stringPayload?: string
  regexpPayload?: string
  objectPayload?: Record<string, RawExternalItemValue>
  fnPayload?: (value: any) => any
}

export interface RawExternalItemFnCtx {
  request: string
  context: string
  dependencyType: string
}

export interface RawExternalItemFnResult {
  externalType?: string
  result?: RawExternalItemValue
}

export interface RawExternalItemValue {
  type: "string" | "bool" | "array"
  stringPayload?: string
  boolPayload?: boolean
  arrayPayload?: Array<string>
}

export interface RawExternalsPluginOptions {
  type: string
  externals: Array<RawExternalItem>
}

export interface RawExternalsPresets {
  node: boolean
  web: boolean
  electron: boolean
  electronMain: boolean
  electronPreload: boolean
  electronRenderer: boolean
}

export interface RawFallbackCacheGroupOptions {
  chunks?: RegExp | 'async' | 'initial' | 'all'
  minSize?: number
  maxSize?: number
  maxAsyncSize?: number
  maxInitialSize?: number
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

export interface RawIncrementalRebuild {
  make: boolean
  emitAsset: boolean
}

export interface RawLibraryAuxiliaryComment {
  root?: string
  commonjs?: string
  commonjs2?: string
  amd?: string
}

export interface RawLibraryName {
  amd?: string
  commonjs?: string
  root?: Array<string>
}

export interface RawLibraryOptions {
  name?: RawLibraryName
  export?: Array<string>
  libraryType: string
  umdNamedDefine?: boolean
  auxiliaryComment?: RawLibraryAuxiliaryComment
}

export interface RawModuleOptions {
  rules: Array<RawModuleRule>
  parser?: Record<string, RawParserOptions>
  generator?: Record<string, RawGeneratorOptions>
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
  funcUse?: (...args: any[]) => any
}

export interface RawNodeOption {
  dirname: string
  filename: string
  global: string
}

export interface RawOptimizationOptions {
  splitChunks?: RawSplitChunksOptions
  moduleIds: string
  chunkIds: string
  removeAvailableModules: boolean
  removeEmptyChunks: boolean
  sideEffects: string
  usedExports: string
  providedExports: boolean
  realContentHash: boolean
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
  devServer: RawDevServer
  snapshot: RawSnapshotOptions
  cache: RawCacheOptions
  experiments: RawExperiments
  node?: RawNodeOption
  profile: boolean
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
  filename: string
  chunkFilename: string
  crossOriginLoading: RawCrossOriginLoading
  cssFilename: string
  cssChunkFilename: string
  hotUpdateMainFilename: string
  hotUpdateChunkFilename: string
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
}

export interface RawParserOptions {
  type: "asset" | "unknown"
  asset?: RawAssetParserOptions
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

export interface RawPresetEnv {
  targets: Array<string>
  mode?: 'usage' | 'entry'
  coreJs?: string
}

export interface RawProgressPluginOptions {
  prefix?: string
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

export interface RawRelayConfig {
  artifactDirectory?: string
  language: 'javascript' | 'typescript' | 'flow'
}

export interface RawResolveOptions {
  preferRelative?: boolean
  extensions?: Array<string>
  mainFiles?: Array<string>
  mainFields?: Array<string>
  browserField?: boolean
  conditionNames?: Array<string>
  alias?: Record<string, Array<string | false>>
  fallback?: Record<string, Array<string | false>>
  symlinks?: boolean
  tsConfigPath?: string
  modules?: Array<string>
  byDependency?: Record<string, RawResolveOptions>
  fullySpecified?: boolean
  exportsFields?: Array<string>
  extensionAlias?: Record<string, Array<string>>
}

export interface RawRspackFuture {
  newResolver: boolean
  newTreeshaking: boolean
  disableTransformByDefault: boolean
}

export interface RawRuleSetCondition {
  type: "string" | "regexp" | "logical" | "array" | "function"
  stringMatcher?: string
  regexpMatcher?: string
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

export interface RawSplitChunksOptions {
  fallbackCacheGroup?: RawFallbackCacheGroupOptions
  name?: string
  cacheGroups?: Record<string, RawCacheGroupOptions>
  /** What kind of chunks should be selected. */
  chunks?: RegExp | 'async' | 'initial' | 'all'
  maxAsyncRequests?: number
  maxInitialRequests?: number
  minChunks?: number
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
  passes: number
  dropConsole: boolean
  keepClassNames: boolean
  keepFnNames: boolean
  comments: "all" | "some" | "false"
  asciiOnly: boolean
  pureFuncs: Array<string>
  extractComments?: string
  test?: RawSwcJsMinimizerRules
  include?: RawSwcJsMinimizerRules
  exclude?: RawSwcJsMinimizerRules
}

export interface RawSwcJsMinimizerRule {
  type: "string" | "regexp"
  stringMatcher?: string
  regexpMatcher?: string
}

export interface RawSwcJsMinimizerRules {
  type: "string" | "regexp" | "array"
  stringMatcher?: string
  regexpMatcher?: string
  arrayMatcher?: Array<RawSwcJsMinimizerRule>
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

/** Builtin loader runner */
export function runBuiltinLoader(builtin: string, options: string | undefined | null, loaderContext: JsLoaderContext): Promise<JsLoaderContext>

export interface ThreadsafeNodeFS {
  writeFile: (...args: any[]) => any
  removeFile: (...args: any[]) => any
  mkdir: (...args: any[]) => any
  mkdirp: (...args: any[]) => any
  removeDirAll: (...args: any[]) => any
}

