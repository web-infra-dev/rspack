import type { AssetInfo, RawModuleRuleUse, RawOptions } from '@rspack/binding';
import { getLightningcssLoaderOptions } from '../builtin-loader/lightningcss';
import { getSwcLoaderOptions } from '../builtin-loader/swc';
import type { Compilation } from '../Compilation';
import type { Compiler } from '../Compiler';
import type { LoaderObject } from '../loader-runner';
import type { Logger } from '../logging/Logger';
import type { Module } from '../Module';
import type { ResolveRequest } from '../Resolver';
import { isNil } from '../util';
import type { InputFileSystem } from '../util/fs';
import type Hash from '../util/hash';
import { parseResource } from '../util/identifier';
import type { RspackOptionsNormalized } from './normalization';
import type {
  Environment,
  Falsy,
  Mode,
  PublicPath,
  Resolve,
  RuleSetLoaderWithOptions,
  RuleSetUseItem,
  Target,
} from './types';

export const BUILTIN_LOADER_PREFIX = 'builtin:';

export interface ComposeJsUseOptions {
  context: RawOptions['context'];
  mode: RawOptions['mode'];
  experiments: RawOptions['experiments'];
  compiler: Compiler;
}

export interface RawSourceMap {
  /**
   * The version of the source map format, always 3
   */
  version: number;
  /**
   * A list of original sources used by the mappings field
   */
  sources: string[];
  /**
   * A string with the encoded mapping data
   */
  mappings: string;
  /**
   * The filename of the generated code that this source map is associated with
   */
  file: string;
  /**
   * An optional source root string, used for relocating source files on a server
   * or removing repeated values in the sources entry.
   */
  sourceRoot?: string;
  /**
   * An array containing the actual content of the original source files
   */
  sourcesContent?: string[];
  /**
   * A list of symbol names which may be used by the mappings field.
   */
  names: string[];
  /**
   * A unique identifier for debugging purposes
   */
  debugId?: string;
  /**
   * An array of indices into the sources array, indicating which sources
   * should be ignored by debuggers
   */
  ignoreList?: number[];
}

export interface AdditionalData {
  [index: string]: any;
}

export type LoaderContextCallback = (
  err?: Error | null,
  content?: string | Buffer,
  sourceMap?: string | RawSourceMap,
  additionalData?: AdditionalData,
) => void;

export type ErrorWithDetails = Error & { details?: string };

// aligned with https://github.com/webpack/webpack/blob/64e8e33151c3fabd3f1917851193e458a526e803/declarations/LoaderContext.d.ts#L19
export type ResolveCallback = (
  err: null | ErrorWithDetails,
  res?: string | false,
  req?: ResolveRequest,
) => void;

export interface DiagnosticLocation {
  /** Text for highlighting the location */
  text?: string;
  /** 1-based line */
  line: number;
  /** 0-based column in bytes */
  column: number;
  /** Length in bytes */
  length: number;
}

export interface Diagnostic {
  message: string;
  help?: string;
  sourceCode?: string;
  /**
   * Location to the source code.
   * If `sourceCode` is not provided, location will be omitted.
   */
  location?: DiagnosticLocation;
  /**
   * Optional filename to show.
   * If provided, it becomes the `StatsError.file` value in stats.
   */
  file?: string;
  severity: 'error' | 'warning';
}

export interface LoaderExperiments {
  /**
   * Certify this invocation's output, source map and effects under checkout relocation.
   * Declare exclusive whole-literal absolute path values; all other output must be invariant.
   * Each normal and invoked pitch must certify independently, before completion.
   * @experimental
   */
  portableOutput(declaration?: { paths?: readonly string[] }): void;

  /**
   * Emit an error or warning diagnostic without marking the current module as a compilation
   * failure.
   */
  emitDiagnostic(diagnostic: Diagnostic): void;
}

export interface ImportModuleOptions {
  /**
   * Specify a layer in which this module is placed/compiled
   */
  layer?: string;
  /**
   * The public path used for the built modules
   */
  publicPath?: PublicPath;
  /**
   * Target base uri
   */
  baseUri?: string;
}

export interface LoaderContext<OptionsType = {}> {
  /**
   * The version number of the loader API. Currently 2.
   * This is useful for providing backwards compatibility. Using the version you can specify
   * custom logic or fallbacks for breaking changes.
   */
  version: 2;
  /**
   * The path string of the current module.
   * @example `'/abc/resource.js?query#hash'`.
   */
  resource: string;
  /**
   * The path string of the current module, excluding the query and fragment parameters.
   * @example `'/abc/resource.js'` in `'/abc/resource.js?query#hash'`.
   */
  resourcePath: string;
  /**
   * The query parameter for the path string of the current module.
   * @example `'?query'` in `'/abc/resource.js?query#hash'`.
   */
  resourceQuery: string;
  /**
   * The fragment parameter of the current module's path string.
   * @example `'#hash'` in `'/abc/resource.js?query#hash'`.
   */
  resourceFragment: string;
  /**
   * Tells Rspack that this loader will be called asynchronously. Returns `this.callback`.
   */
  async(): LoaderContextCallback;
  /**
   * A function that can be called synchronously or asynchronously to return multiple results.
   * The expected arguments are:
   *
   * 1. The first parameter is an `Error` when the loader fails, or `null` or `undefined` when it
   * succeeds.
   * 2. The second parameter is the transformed content as a `string` or `Buffer`. It can be
   * omitted when reporting an error.
   * 3. The third parameter is an optional source map as a `string` or `RawSourceMap`.
   * 4. The fourth parameter is optional additional data. Rspack passes it as the third argument
   * to the next loader in the chain.
   */
  callback: LoaderContextCallback;
  /**
   * By default, the final build result produced for the current module by the entire loader chain
   * is cacheable. Passing `false` marks that result as non-cacheable. Calls from subsequent loaders
   * with `true` or no argument do not make it cacheable again; only `this.clearDependencies()`
   * resets this state.
   */
  cacheable(cacheable?: boolean): void;
  /**
   * Tells if source map should be generated. Since generating source maps can be an expensive task,
   * you should check if source maps are actually requested.
   */
  sourceMap: boolean;
  /**
   * The base path configured in Rspack config via `context`.
   */
  rootContext: string;
  /**
   * The directory path of the currently processed module, which changes with the
   * location of each processed module.
   * For example, if the loader is processing `/project/src/components/Button.js`,
   * then the value of `this.context` would be `/project/src/components`.
   * The value is `null` when the current module has no resource path.
   */
  context: string | null;
  /**
   * The index in the loaders array of the current loader.
   */
  loaderIndex: number;
  /**
   * A request string consisting of the loaders that follow the current loader
   * in the chain and the current resource, joined with `!`.
   */
  remainingRequest: string;
  /**
   * A request string consisting of the current loader, the loaders that follow it, and the current
   * resource, joined with `!`.
   */
  currentRequest: string;
  /**
   * A request string consisting of the loaders that precede the current loader, joined with `!`.
   * It does not include the current resource.
   */
  previousRequest: string;
  /**
   * The complete request string, consisting of all loaders and the current resource joined with
   * `!`. For example, if a `resource.js` is processed by `loader1.js` and `loader2.js`, the value
   * is `/path/to/loader1.js!/path/to/loader2.js!/path/to/resource.js`.
   */
  request: string;
  /**
   * An array containing all loaders applied to the current module. Each item
   * provides information such as the resolved request, path, query, and
   * options. The array can be modified during the pitch phase to adjust the
   * loader chain.
   */
  loaders: LoaderObject[];
  /**
   * The value of the `mode` configuration. It is `undefined` when `mode` is not configured, even
   * though Rspack applies production-oriented defaults in that case.
   */
  mode?: Mode;
  /**
   * A loader-facing target derived from the `target` configuration.
   */
  target?: Target;
  /**
   * Describes the capabilities supported by the target environment. By default, this is the
   * effective value of `output.environment`: Rspack infers capabilities from `target`, then
   * applies the explicit settings from `output.environment`.
   */
  environment: Environment;
  /**
   * Whether HMR is enabled.
   */
  hot?: boolean;
  /**
   * Get the options passed in by the loader's user.
   * @param schema To provide the best performance, Rspack does not perform the schema
   * validation. If your loader requires schema validation, please call schema-utils or
   * zod on your own.
   */
  getOptions(schema?: any): OptionsType;
  /**
   * Resolve a module specifier.
   * @param context The absolute path to a directory. This directory is used as the starting
   * location for resolving.
   * @param request The module specifier to be resolved.
   * @param callback Receives an error, the resolved path or `false`, and optional resolution
   * details.
   */
  resolve(context: string, request: string, callback: ResolveCallback): void;
  /**
   * Create a resolver like `this.resolve`. When the returned resolver is called without a
   * callback, it returns a Promise.
   * @param options Optional options used to customize the resolver.
   */
  getResolve(
    options?: Resolve,
  ): ((context: string, request: string, callback: ResolveCallback) => void) &
    ((context: string, request: string) => Promise<string | false | undefined>);
  /**
   * Get the logger of this compilation, through which messages can be logged.
   * @param name An optional name for the logger.
   */
  getLogger(name?: string): Logger;
  /**
   * Emit an error. Unlike `throw` and `this.callback(err)` in the loader, it does not mark the
   * current module as a compilation failure. It adds an error to Rspack's Compilation and displays
   * it on the command line at the end of this compilation.
   */
  emitError(error: Error): void;
  /**
   * Emit a warning.
   */
  emitWarning(warning: Error): void;
  /**
   * Emit a new file. This method allows you to create new files during the loader execution.
   */
  emitFile(
    name: string,
    content: string | Buffer,
    sourceMap?: string,
    assetInfo?: AssetInfo,
  ): void;
  /**
   * Add a file as a dependency on the loader results so that any changes to them can be listened to.
   * For example, `sass-loader`, `less-loader` use this trick to recompile when the imported style
   * files change.
   */
  addDependency(file: string): void;
  /**
   * Alias of `this.addDependency()`.
   */
  dependency(file: string): void;
  /**
   * Add the directory as a dependency for the loader results so that any changes to the
   * files in the directory can be listened to.
   */
  addContextDependency(context: string): void;
  /**
   * Add a currently non-existent file as a dependency of the loader result, so that its
   * creation and any changes can be listened. For example, when a new file is created at
   * that path, it will trigger a rebuild.
   */
  addMissingDependency(missing: string): void;
  /**
   * Clears all file, context, and missing dependencies collected by the loader chain. Build
   * dependencies are not cleared. This also resets `cacheable` to `true`, overriding any earlier
   * call to `this.cacheable(false)`. Only use this method when the current loader will register
   * every dependency required by the final result.
   */
  clearDependencies(): void;
  /**
   * Get a copy of all files the loader currently watches as dependencies.
   */
  getDependencies(): string[];
  /**
   * Get a copy of all directories the loader currently watches as context dependencies.
   */
  getContextDependencies(): string[];
  /**
   * Get a copy of all paths to files that the loader is watching but that do not exist yet.
   */
  getMissingDependencies(): string[];
  /**
   * Add a file as a build dependency of the loader result.
   * Build dependencies invalidate the persistent cache when they change.
   */
  addBuildDependency(file: string): void;
  /**
   * Compile and execute a module at the build time.
   * This is an alternative lightweight solution for the child compiler.
   * `importModule` will return a Promise if no callback is provided.
   *
   * @example
   * ```ts
   * const modulePath = path.resolve(__dirname, 'some-module.ts');
   * const moduleExports = await this.importModule(modulePath, {
   *   // optional options
   * });
   * ```
   */
  importModule<T = any>(
    request: string,
    options: ImportModuleOptions | undefined,
    callback: (err?: null | Error, exports?: T) => any,
  ): void;
  importModule<T = any>(
    request: string,
    options?: ImportModuleOptions,
  ): Promise<T>;
  /**
   * Access to the `compilation` object's `inputFileSystem` property.
   */
  fs: InputFileSystem;
  /**
   * This is an experimental API and maybe subject to change.
   * @experimental
   */
  experiments: LoaderExperiments;
  /**
   * Access to some utilities.
   */
  utils: {
    /**
     * Return a new request string using absolute paths when possible.
     */
    absolutify: (context: string, request: string) => string;
    /**
     * Return a new request string avoiding absolute paths when possible.
     */
    contextify: (context: string, request: string) => string;
    /**
     * Return a new Hash object from provided hash function.
     */
    createHash: (algorithm?: string) => Hash;
  };
  /**
   * The value depends on the loader configuration:
   * - If the current loader was configured with an options object, `this.query` will
   * point to that object.
   * - If the current loader has no options, but was invoked with a query string, this
   * will be a string starting with `?`.
   */
  query: string | OptionsType;
  /**
   * A data object shared between the pitch and the normal phase.
   */
  data: unknown;
  /**
   * Access to the current Compiler object of Rspack.
   */
  _compiler: Compiler;
  /**
   * Access to the current Compilation object of Rspack.
   */
  _compilation: Compilation;
  /**
   * @deprecated Hacky access to the Module object being loaded.
   */
  _module: Module;
  /**
   * Note: This is not a Rspack public API, maybe removed in future.
   * Store some data from loader, and consume it from parser, it may be removed in the future
   *
   * @internal
   */
  __internal__setParseMeta: (key: string, value: string) => void;
}

export type LoaderDefinitionFunction<
  OptionsType = {},
  ContextAdditions = {},
> = (
  this: LoaderContext<OptionsType> & ContextAdditions,
  content: string,
  sourceMap?: string | RawSourceMap,
  additionalData?: AdditionalData,
) => string | void | Buffer | Promise<string | Buffer | void>;

export type PitchLoaderDefinitionFunction<
  OptionsType = {},
  ContextAdditions = {},
> = (
  this: LoaderContext<OptionsType> & ContextAdditions,
  remainingRequest: string,
  previousRequest: string,
  data: object,
) => string | void | Buffer | Promise<string | Buffer | void>;

/**
 * Defines a loader for Rspack.
 * A loader is a transformer that converts various types of modules into Rspack
 * supported types. By using different kinds of loaders, you can extend Rspack to
 * process additional module types, including JSX, Markdown, Sass, Less, and more.
 *
 * @template OptionsType - The type of options that the loader accepts
 * @template ContextAdditions - Additional properties to add to the loader context
 *
 * @example
 * ```ts
 * import type { LoaderDefinition } from '@rspack/core';
 *
 * type MyLoaderOptions = {
 *   foo: string;
 * };
 *
 * const myLoader: LoaderDefinition<MyLoaderOptions> = function(source) {
 *   return someOperation(source);
 * };
 *
 * export default myLoader;
 * ```
 */
export type LoaderDefinition<
  OptionsType = {},
  ContextAdditions = {},
> = LoaderDefinitionFunction<OptionsType, ContextAdditions> & {
  raw?: false;
  pitch?: PitchLoaderDefinitionFunction;
};

export function createRawModuleRuleUses(
  uses: RuleSetUseItem | RuleSetUseItem[],
  path: string,
  options: ComposeJsUseOptions,
  fingerprint = false,
  strict = false,
): RawModuleRuleUse[] {
  const normalizeRuleSetUseItem = (
    item: RuleSetUseItem,
  ): RuleSetLoaderWithOptions | Falsy =>
    typeof item === 'string' ? { loader: item } : item;
  const allUses = Array.isArray(uses)
    ? [...uses].map(normalizeRuleSetUseItem)
    : [normalizeRuleSetUseItem(uses)];
  return createRawModuleRuleUsesImpl(
    allUses,
    path,
    options,
    fingerprint,
    strict,
  );
}

export type GetLoaderOptions = (
  o: RuleSetLoaderWithOptions['options'],
  options: ComposeJsUseOptions,
) => RuleSetLoaderWithOptions['options'];

function getBuiltinLoaderOptions(
  identifier: string,
  o: RuleSetLoaderWithOptions['options'],
  options: ComposeJsUseOptions,
): RuleSetLoaderWithOptions['options'] {
  if (identifier.startsWith(`${BUILTIN_LOADER_PREFIX}swc-loader`)) {
    return getSwcLoaderOptions(o, options);
  }

  if (identifier.startsWith(`${BUILTIN_LOADER_PREFIX}lightningcss-loader`)) {
    return getLightningcssLoaderOptions(o, options);
  }

  return o;
}

function createRawModuleRuleUsesImpl(
  uses: (RuleSetLoaderWithOptions | Falsy)[],
  path: string,
  options: ComposeJsUseOptions,
  fingerprint: boolean,
  strict: boolean,
): RawModuleRuleUse[] {
  if (!uses.length) {
    return [];
  }

  const filteredUses = uses.filter((use): use is RuleSetLoaderWithOptions =>
    Boolean(use),
  );
  return filteredUses.map((use, index) => {
    let o: string | undefined;
    let fingerprintOptions = use.options;
    let isBuiltin = false;
    if (use.loader.startsWith(BUILTIN_LOADER_PREFIX)) {
      const temp = getBuiltinLoaderOptions(use.loader, use.options, options);
      fingerprintOptions = temp;
      // keep json with indent so miette can show pretty error
      o = isNil(temp)
        ? undefined
        : typeof temp === 'string'
          ? temp
          : JSON.stringify(temp, null, 2);
      isBuiltin = true;
    }

    return {
      loader: resolveStringifyLoaders(
        use,
        `${path}[${index}]`,
        options.compiler,
        isBuiltin,
        fingerprintOptions,
      ),
      options: o,
      cache: use.cache ?? false,
      optionsCacheKey: use.cache
        ? (JSON.stringify(fingerprintOptions) ?? '')
        : '',
      comparableOptions: fingerprint
        ? encodeComparableOptions(fingerprintOptions, strict)
        : undefined,
      codeFilter: createRawLoaderCodeFilter(
        use.codeFilter,
        `${path}[${index}].codeFilter`,
      ),
    };
  });
}

function incomparable(): never {
  throw new Error('Incomparable loader options');
}

function encodeProperty(
  owner: object,
  key: PropertyKey,
  seen: Map<object, string | undefined>,
  strict: boolean,
): string {
  // A descriptor reads the value without running a getter.
  const descriptor = Reflect.getOwnPropertyDescriptor(owner, key);
  if (descriptor?.enumerable && 'value' in descriptor) {
    return encodeComparable(descriptor.value, seen, strict);
  }
  if (strict) incomparable();
  return 'g';
}

function encodeProperties(
  owner: object,
  keys: (string | symbol)[],
  seen: Map<object, string | undefined>,
  strict: boolean,
): string {
  const entries: string[] = [];
  for (const key of keys) {
    const encodable =
      typeof key === 'string' &&
      (strict
        ? key !== 'toJSON'
        : Object.prototype.propertyIsEnumerable.call(owner, key));
    if (encodable) {
      entries.push(
        `${JSON.stringify(key)}:${encodeProperty(owner, key, seen, strict)}`,
      );
    } else if (strict) {
      incomparable();
    }
  }
  return `{${entries.join()}}`;
}

// These methods read a value's internal state, so its own properties can't stand in for it.
function encodeBuiltin(
  value: object,
  prototype: object | null,
  seen: Map<object, string | undefined>,
  strict: boolean,
): string | undefined {
  const items: string[] = [];
  switch (prototype) {
    case Date.prototype:
      return `d${Date.prototype.getTime.call(value as Date)}`;
    case RegExp.prototype: {
      const { source, flags } = value as RegExp;
      return `r${JSON.stringify(source)}/${flags}`;
    }
    case Set.prototype:
      Set.prototype.forEach.call(value as Set<unknown>, (item) => {
        items.push(encodeComparable(item, seen, strict));
      });
      return `S[${items.join()}]`;
    case Map.prototype:
      Map.prototype.forEach.call(
        value as Map<unknown, unknown>,
        (item, key) => {
          const encodedKey = encodeComparable(key, seen, strict);
          items.push(`[${encodedKey},${encodeComparable(item, seen, strict)}]`);
        },
      );
      return `M[${items.join()}]`;
    case Boolean.prototype:
    case Number.prototype:
    case String.prototype:
    case Symbol.prototype:
    case BigInt.prototype:
      return incomparable();
    default:
      return undefined;
  }
}

function encodeObject(
  value: object,
  seen: Map<object, string | undefined>,
  strict: boolean,
): string {
  const prototype = Object.getPrototypeOf(value);
  const keys = Reflect.ownKeys(value);
  const builtin = encodeBuiltin(value, prototype, seen, strict);
  if (builtin !== undefined) {
    // `lastIndex` is the own key every regexp has, and it changes as the regexp runs.
    const ownKeys = prototype === RegExp.prototype ? 1 : 0;
    if (strict && keys.length !== ownKeys) incomparable();
    return builtin;
  }
  // Own keys are the indices and `length`, so any other key leaves a hole that `encodeProperty` sees.
  if (
    Array.isArray(value) &&
    prototype === Array.prototype &&
    keys.length === value.length + 1
  ) {
    const items: string[] = [];
    for (let index = 0; index < value.length; index++) {
      items.push(encodeProperty(value, index, seen, strict));
    }
    return `[${items.join()}]`;
  }
  const properties = encodeProperties(value, keys, seen, strict);
  if (prototype === Object.prototype) return properties;
  if (prototype === null) return `N${properties}`;
  if (strict) incomparable();
  // The prototype's own `constructor`, as reading through the value could run a getter.
  const constructor = Reflect.getOwnPropertyDescriptor(
    prototype,
    'constructor',
  )?.value;
  return `c${JSON.stringify(constructor?.name)}${properties}`;
}

// Every value has its own tag, so a string that reads like another value stays a string.
// Objects never reach JSON.stringify, so `toJSON` doesn't run.
function encodeComparable(
  value: unknown,
  seen: Map<object, string | undefined>,
  strict: boolean,
): string {
  switch (typeof value) {
    case 'undefined':
      return 'u';
    case 'boolean':
      return value ? 't' : 'F';
    case 'string':
      return `s${JSON.stringify(value)}`;
    case 'number':
      return Object.is(value, -0) ? 'n-0' : `n${value}`;
    case 'bigint':
      return `b${value}`;
    case 'object':
      break;
    default:
      if (strict) incomparable();
      return typeof value === 'symbol'
        ? `y${JSON.stringify(value.description)}`
        : 'f';
  }
  if (value === null) return 'l';
  if (seen.has(value)) {
    // Object identity is observable to a loader and has no place in the encoding, so results of a
    // function `use` can't repeat an object. A static config shares objects freely, and only a
    // cycle, still undefined here, has no encoding.
    const encoded = seen.get(value);
    if (strict || encoded === undefined) incomparable();
    return encoded;
  }
  seen.set(value, undefined);
  const encoded = encodeObject(value, seen, strict);
  seen.set(value, encoded);
  return encoded;
}

/**
 * Encodes loader options as typed data that a build in another process can compare.
 * Returns `undefined` for options that can't be compared.
 *
 * Results of a function `use` are built per module, so `strict` accepts only values that encode
 * exactly. Options of an array `use` are static config: functions, symbols and class instances
 * stand in for what they are, and are assumed not to depend on the checkout.
 */
function encodeComparableOptions(
  options: unknown,
  strict: boolean,
): string | undefined {
  try {
    return encodeComparable(options, new Map(), strict);
  } catch {
    return undefined;
  }
}

function createRawLoaderCodeFilter(
  codeFilter: RuleSetLoaderWithOptions['codeFilter'],
  path: string,
): string | RegExp | undefined {
  if (codeFilter === undefined) return undefined;
  if (typeof codeFilter === 'string' || codeFilter instanceof RegExp) {
    return codeFilter;
  }
  throw new Error(`Expected a string or RegExp at \`${path}\`.`);
}

function resolveStringifyLoaders(
  use: RuleSetLoaderWithOptions,
  path: string,
  compiler: Compiler,
  isBuiltin: boolean,
  normalizedOptions: RuleSetLoaderWithOptions['options'],
) {
  const obj = parseResource(use.loader);
  let ident = use.ident;

  if (use.options === null) {
  } else if (use.options === undefined) {
  } else if (typeof use.options === 'string') obj.query = `?${use.options}`;
  else if (use.ident) obj.query = `??${(ident = use.ident)}`;
  else if (typeof use.options === 'object' && use.options.ident)
    obj.query = `??${(ident = use.options.ident)}`;
  else if (typeof use.options === 'object') obj.query = `??${(ident = path)}`;
  else obj.query = `?${JSON.stringify(use.options)}`;

  const parallelism = use.parallel;

  if (!!parallelism && (!use.options || typeof use.options !== 'object')) {
    throw new Error(
      `\`Rule.use.parallel\` requires \`Rule.use.options\` to be an object.\nHowever the received value is \`${use.options}\` under option path \`${path}\`\nInternally, parallelism is provided by passing \`Rule.use.ident\` to the loader as an identifier to ident the parallelism option\nYou can either replace the \`Rule.use.loader\` with \`Rule.use.options = {}\` or remove \`Rule.use.parallel\`.`,
    );
  }

  if (use.options && typeof use.options === 'object') {
    if (!ident) ident = '[[missing ident]]';
    compiler.__internal__ruleSet.references.set(ident, use.options);
    compiler.__internal__ruleSet.references.set(
      `${ident}$$parallelism`,
      parallelism,
    );
    if (isBuiltin) {
      // Inline builtin loaders resolve their options from this native map.
      compiler.__internal__ruleSet.builtinReferences.set(
        ident,
        normalizedOptions,
      );
    }
  }

  return obj.path + obj.query + obj.fragment;
}

export function isUseSourceMap(
  devtool: RspackOptionsNormalized['devtool'],
): boolean {
  if (!devtool) {
    return false;
  }
  return (
    devtool.includes('source-map') &&
    (devtool.includes('module') || !devtool.includes('cheap'))
  );
}

export function isUseSimpleSourceMap(
  devtool: RspackOptionsNormalized['devtool'],
): boolean {
  if (!devtool) {
    return false;
  }
  return devtool.includes('source-map') && !isUseSourceMap(devtool);
}
