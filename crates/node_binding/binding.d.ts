/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface NodeFS {
  writeFile: (...args: any[]) => any
  mkdir: (...args: any[]) => any
  mkdirp: (...args: any[]) => any
}
export interface ThreadsafeNodeFS {
  writeFile: (...args: any[]) => any
  mkdir: (...args: any[]) => any
  mkdirp: (...args: any[]) => any
  removeDirAll: (...args: any[]) => any
}
export interface RawBannerCondition {
  type: "string" | "regexp"
  stringMatcher?: string
  regexpMatcher?: string
}
export interface RawBannerConditions {
  type: "string" | "regexp" | "array"
  stringMatcher?: string
  regexpMatcher?: string
  arrayMatcher?: Array<RawBannerCondition>
}
export interface RawBannerConfig {
  banner: string
  entryOnly?: boolean
  footer?: boolean
  raw?: boolean
  test?: RawBannerConditions
  include?: RawBannerConditions
  exclude?: RawBannerConditions
}
export interface RawPattern {
  from: string
  to?: string
  context?: string
  toType?: string
  noErrorOnMissing: boolean
  force: boolean
  priority: number
  globOptions: RawGlobOptions
}
export interface RawGlobOptions {
  caseSensitiveMatch?: boolean
  dot?: boolean
  ignore?: Array<string>
}
export interface RawCopyConfig {
  patterns: Array<RawPattern>
}
export interface RawCssPluginConfig {
  modules: RawCssModulesConfig
}
export interface RawCssModulesConfig {
  localsConvention: "asIs" | "camelCase" | "camelCaseOnly" | "dashes" | "dashesOnly"
  localIdentName: string
  exportsOnly: boolean
}
export interface RawDecoratorOptions {
  legacy: boolean
  emitMetadata: boolean
}
export interface RawHtmlPluginConfig {
  /** emitted file name in output path */
  filename?: string
  /** template html file */
  template?: string
  templateContent?: string
  templateParameters?: Record<string, string>
  /** `head`, `body` or None */
  inject?: "head" | "body"
  /** path or `auto` */
  publicPath?: string
  /** `blocking`, `defer`, or `module` */
  scriptLoading?: "blocking" | "defer" | "module"
  /** entry_chunk_name (only entry chunks are supported) */
  chunks?: Array<string>
  excludedChunks?: Array<string>
  sri?: "sha256" | "sha384" | "sha512"
  minify?: boolean
  title?: string
  favicon?: string
  meta?: Record<string, Record<string, string>>
}
export interface RawStyleConfig {
  styleLibraryDirectory?: string
  custom?: string
  css?: string
  bool?: boolean
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
export interface RawPostCssConfig {
  pxtorem?: RawPxToRemConfig
}
export interface RawPxToRemConfig {
  rootValue?: number
  unitPrecision?: number
  selectorBlackList?: Array<string>
  propList?: Array<string>
  replace?: boolean
  mediaQuery?: boolean
  minPixelValue?: number
}
export interface RawProgressPluginConfig {
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
export interface RawMinification {
  passes: number
  dropConsole: boolean
  pureFuncs: Array<string>
}
export interface RawPresetEnv {
  targets: Array<string>
  mode?: 'usage' | 'entry'
  coreJs?: string
}
export interface RawBuiltins {
  html?: Array<RawHtmlPluginConfig>
  css?: RawCssPluginConfig
  postcss?: RawPostCssConfig
  minifyOptions?: RawMinification
  presetEnv?: RawPresetEnv
  define: Record<string, string>
  provide: Record<string, string[]>
  treeShaking: boolean
  progress?: RawProgressPluginConfig
  react: RawReactOptions
  decorator?: RawDecoratorOptions
  noEmitAssets: boolean
  emotion?: string
  devFriendlySplitChunks: boolean
  copy?: RawCopyConfig
  banner?: Array<RawBannerConfig>
  pluginImport?: Array<RawPluginImportConfig>
  relay?: RawRelayConfig
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
export interface RawDevServer {
  hot: boolean
}
export interface RawEntryItem {
  import: Array<string>
  runtime?: string
}
export interface RawExperiments {
  lazyCompilation: boolean
  incrementalRebuild: boolean
  asyncWebAssembly: boolean
  newSplitChunks: boolean
}
export interface RawExternalItem {
  type: "string" | "regexp" | "object"
  stringPayload?: string
  regexpPayload?: string
  objectPayload?: Record<string, RawExternalItemValue>
}
export interface RawExternalItemValue {
  type: "string" | "bool"
  stringPayload?: string
  boolPayload?: boolean
}
export interface RawExternalsPresets {
  node: boolean
}
export interface JsLoader {
  /** composed loader name, xx-loader!yy-loader!zz-loader */
  name: string
  func: (...args: any[]) => any
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
}
export interface JsLoaderResult {
  content: Buffer
  fileDependencies: Array<string>
  contextDependencies: Array<string>
  missingDependencies: Array<string>
  buildDependencies: Array<string>
  sourceMap?: Buffer
  additionalData?: Buffer
  cacheable: boolean
}
/**
 * `loader` is for js side loader, `builtin_loader` is for rust side loader,
 * which is mapped to real rust side loader by [get_builtin_loader].
 *
 * `options` is
 *   - a `None` on rust side and handled by js side `getOptions` when
 * using with `loader`.
 *   - a `Some(string)` on rust side, deserialized by `serde_json::from_str`
 * and passed to rust side loader in [get_builtin_loader] when using with
 * `builtin_loader`.
 */
export interface RawModuleRuleUse {
  jsLoader?: JsLoader
  builtinLoader?: string
  options?: string
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
export interface RawModuleRule {
  /** A condition matcher matching an absolute path. */
  test?: RawRuleSetCondition
  include?: RawRuleSetCondition
  exclude?: RawRuleSetCondition
  /** A condition matcher matching an absolute path. */
  resource?: RawRuleSetCondition
  /** A condition matcher against the resource query. */
  resourceQuery?: RawRuleSetCondition
  descriptionData?: Record<string, RawRuleSetCondition>
  sideEffects?: boolean
  use?: Array<RawModuleRuleUse>
  type?: string
  parser?: RawModuleRuleParser
  generator?: RawModuleRuleGenerator
  resolve?: RawResolveOptions
  issuer?: RawRuleSetCondition
  dependency?: RawRuleSetCondition
  oneOf?: Array<RawModuleRule>
}
export interface RawModuleRuleGenerator {
  filename?: string
}
export interface RawModuleRuleParser {
  dataUrlCondition?: RawAssetParserDataUrlOption
}
export interface RawAssetParserDataUrlOption {
  maxSize?: number
}
export interface RawAssetParserOptions {
  dataUrlCondition?: RawAssetParserDataUrlOption
}
export interface RawParserOptions {
  asset?: RawAssetParserOptions
}
export interface RawModuleOptions {
  rules: Array<RawModuleRule>
  parser?: RawParserOptions
}
export interface RawNodeOption {
  dirname: string
  filename: string
  global: string
}
export interface RawOptimizationOptions {
  splitChunks?: RawSplitChunksOptions
  moduleIds: string
  removeAvailableModules: boolean
  sideEffects: string
}
export interface RawTrustedTypes {
  policyName?: string
}
export interface RawLibraryName {
  amd?: string
  commonjs?: string
  root?: Array<string>
}
export interface RawLibraryAuxiliaryComment {
  root?: string
  commonjs?: string
  commonjs2?: string
  amd?: string
}
export interface RawLibraryOptions {
  name?: RawLibraryName
  export?: Array<string>
  libraryType: string
  umdNamedDefine?: boolean
  auxiliaryComment?: RawLibraryAuxiliaryComment
}
export interface RawCrossOriginLoading {
  type: "bool" | "string"
  stringPayload?: string
  boolPayload?: boolean
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
  uniqueName: string
  chunkLoadingGlobal: string
  library?: RawLibraryOptions
  strictModuleErrorHandling: boolean
  enabledLibraryTypes?: Array<string>
  globalObject: string
  importFunctionName: string
  iife: boolean
  module: boolean
  chunkFormat?: string
  chunkLoading?: string
  enabledChunkLoadingTypes?: Array<string>
  trustedTypes?: RawTrustedTypes
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
}
export interface RawSnapshotStrategy {
  hash: boolean
  timestamp: boolean
}
export interface RawSnapshotOptions {
  resolve: RawSnapshotStrategy
  module: RawSnapshotStrategy
}
export interface RawSplitChunksOptions {
  cacheGroups?: Record<string, RawCacheGroupOptions>
  /** What kind of chunks should be selected. */
  chunks?: string
  maxAsyncRequests?: number
  maxInitialRequests?: number
  minChunks?: number
  minSize?: number
  enforceSizeThreshold?: number
  minRemainingSize?: number
}
export interface RawCacheGroupOptions {
  priority?: number
  test?: string
  /** What kind of chunks should be selected. */
  chunks?: string
  minChunks?: number
  name?: string
}
export interface RawStatsOptions {
  colors: boolean
  reasons: boolean
}
export interface RawOptions {
  entry: Record<string, RawEntryItem>
  /**
   * Using this Vector to track the original order of user land entry configuration
   * std::collection::HashMap does not guarantee the insertion order, for more details you could refer
   * https://doc.rust-lang.org/std/collections/index.html#iterators:~:text=For%20unordered%20collections%20like%20HashMap%2C%20the%20items%20will%20be%20yielded%20in%20whatever%20order%20the%20internal%20representation%20made%20most%20convenient.%20This%20is%20great%20for%20reading%20through%20all%20the%20contents%20of%20the%20collection.
   */
  entryOrder: Array<string>
  mode?: undefined | 'production' | 'development' | 'none'
  target: Array<string>
  context: string
  output: RawOutputOptions
  resolve: RawResolveOptions
  module: RawModuleOptions
  builtins: RawBuiltins
  externals?: Array<RawExternalItem>
  externalsType: string
  externalsPresets: RawExternalsPresets
  devtool: string
  optimization: RawOptimizationOptions
  stats: RawStatsOptions
  devServer: RawDevServer
  snapshot: RawSnapshotOptions
  cache: RawCacheOptions
  experiments: RawExperiments
  node?: RawNodeOption
}
export interface JsAssetInfoRelated {
  sourceMap?: string
}
export interface JsAssetInfo {
  /**
   * if the asset can be long term cached forever (contains a hash)
   * whether the asset is minimized
   */
  minimized: boolean
  /**
   * the value(s) of the full hash used for this asset
   * the value(s) of the chunk hash used for this asset
   * the value(s) of the module hash used for this asset
   * the value(s) of the content hash used for this asset
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
}
export interface JsAsset {
  name: string
  source?: JsCompatSource
  info: JsAssetInfo
}
export interface JsChunk {
  files: Array<string>
}
export interface JsChunkGroup {
  chunks: Array<JsChunk>
}
export interface JsHooks {
  processAssetsStageAdditional: (...args: any[]) => any
  processAssetsStagePreProcess: (...args: any[]) => any
  processAssetsStageAdditions: (...args: any[]) => any
  processAssetsStageNone: (...args: any[]) => any
  processAssetsStageOptimizeInline: (...args: any[]) => any
  processAssetsStageSummarize: (...args: any[]) => any
  processAssetsStageReport: (...args: any[]) => any
  compilation: (...args: any[]) => any
  thisCompilation: (...args: any[]) => any
  emit: (...args: any[]) => any
  afterEmit: (...args: any[]) => any
  make: (...args: any[]) => any
  optimizeModules: (...args: any[]) => any
  optimizeChunkModule: (...args: any[]) => any
  finishModules: (...args: any[]) => any
  normalModuleFactoryResolveForScheme: (...args: any[]) => any
}
export interface JsModule {
  originalSource?: JsCompatSource
  resource: string
  moduleIdentifier: string
}
export interface SchemeAndJsResourceData {
  resourceData: JsResourceData
  scheme: string
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
export interface JsCompatSource {
  /** Whether the underlying data structure is a `RawSource` */
  isRaw: boolean
  /** Whether the underlying value is a buffer or string */
  isBuffer: boolean
  source: Buffer
  map?: Buffer
}
export interface JsStatsError {
  message: string
  formatted: string
}
export interface JsStatsWarning {
  message: string
  formatted: string
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
export interface JsStatsModule {
  type: string
  moduleType: string
  identifier: string
  name: string
  id: string
  chunks: Array<string>
  size: number
  issuer?: string
  issuerName?: string
  issuerId?: string
  issuerPath: Array<JsStatsModuleIssuer>
  reasons?: Array<JsStatsModuleReason>
}
export interface JsStatsModuleIssuer {
  identifier: string
  name: string
  id: string
}
export interface JsStatsModuleReason {
  moduleIdentifier?: string
  moduleName?: string
  moduleId?: string
  type?: string
  userRequest?: string
}
export interface JsStatsChunk {
  type: string
  files: Array<string>
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
export interface JsStatsChunkGroupAsset {
  name: string
  size: number
}
export interface JsStatsChunkGroup {
  name: string
  assets: Array<JsStatsChunkGroupAsset>
  chunks: Array<string>
  assetsSize: number
}
export interface JsStatsAssetsByChunkName {
  name: string
  files: Array<string>
}
export interface JsStatsGetAssets {
  assets: Array<JsStatsAsset>
  assetsByChunkName: Array<JsStatsAssetsByChunkName>
}
/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/d1d0607158ab40463d1b123fed52cc526eba8385/bindings/binding_core_node/src/util.rs#L29-L58
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
*/
export function initCustomTraceSubscriber(): void
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
  get hash(): string
  getFileDependencies(): Array<string>
  getContextDependencies(): Array<string>
  getMissingDependencies(): Array<string>
  getBuildDependencies(): Array<string>
  pushDiagnostic(severity: "error" | "warning", title: string, message: string): void
  getStats(): JsStats
  addFileDependencies(deps: Array<string>): void
  addContextDependencies(deps: Array<string>): void
  addMissingDependencies(deps: Array<string>): void
  addBuildDependencies(deps: Array<string>): void
}
export class JsStats {
  getAssets(): JsStatsGetAssets
  getModules(): Array<JsStatsModule>
  getChunks(chunkModules: boolean, chunksRelations: boolean): Array<JsStatsChunk>
  getEntrypoints(): Array<JsStatsChunkGroup>
  getNamedChunkGroups(): Array<JsStatsChunkGroup>
  getErrors(): Array<JsStatsError>
  getWarnings(): Array<JsStatsWarning>
  getHash(): string
}
export class Rspack {
  constructor(options: RawOptions, jsHooks: JsHooks | undefined | null, outputFilesystem: ThreadsafeNodeFS)
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
