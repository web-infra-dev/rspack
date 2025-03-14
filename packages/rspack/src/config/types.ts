import type { AssetInfo, RawFuncUseCtx } from "@rspack/binding";
import type * as webpackDevServer from "webpack-dev-server";
import type { ChunkGraph } from "../ChunkGraph";
import type { Compilation, PathData } from "../Compilation";
import type { Compiler } from "../Compiler";
import type { Module } from "../Module";
import type ModuleGraph from "../ModuleGraph";
import type { LazyCompilationDefaultBackendOptions } from "../builtin-plugin/lazy-compilation/backend";
import type { Chunk } from "../exports";

export type FilenameTemplate = string;

export type Filename =
	| FilenameTemplate
	| ((pathData: PathData, assetInfo?: AssetInfo) => string);

//#region Name
/** Name of the configuration. Used when loading multiple configurations. */
export type Name = string;
//#endregion

//#region Dependencies
/** A list of name defining all sibling configurations it depends on. Dependent configurations need to be compiled first. */
export type Dependencies = Name[];
//#endregion

//#region Context
/**
 * The context configuration is used to set the base directory for Rspack builds.
 * @default process.cwd()
 * */
export type Context = string;
//#endregion

//#region Mode
/**
 * The mode configuration is used to set the build mode of Rspack to enable the default optimization strategy.
 * @default 'production'
 * */
export type Mode = "development" | "production" | "none";
//#endregion

//#region Falsy
export type Falsy = false | "" | 0 | null | undefined;
//#endregion

//#region Entry
/** The publicPath of the resource referenced by this entry. */
export type PublicPath = "auto" | Filename;

/** The baseURI of the resource referenced by this entry. */
export type BaseUri = string;

/** How this entry load other chunks. */
export type ChunkLoadingType =
	| string
	| "jsonp"
	| "import-scripts"
	| "require"
	| "async-node"
	| "import";

/** How this entry load other chunks. */
export type ChunkLoading = false | ChunkLoadingType;

/** Whether to create a load-on-demand asynchronous chunk for entry. */
export type AsyncChunks = boolean;

/** Option to set the method of loading WebAssembly Modules. */
export type WasmLoadingType =
	| string
	| "fetch-streaming"
	| "fetch"
	| "async-node";

/** Option to set the method of loading WebAssembly Modules. */
export type WasmLoading = false | WasmLoadingType;

export type ScriptType = false | "text/javascript" | "module";

export type LibraryCustomUmdObject = {
	amd?: string;
	commonjs?: string;
	root?: string | string[];
};

/** Specify a name for the library. */
export type LibraryName = string | string[] | LibraryCustomUmdObject;

export type LibraryCustomUmdCommentObject = {
	amd?: string;
	commonjs?: string;
	commonjs2?: string;
	root?: string;
};

/** Use a container(defined in global space) for calling define/require functions in an AMD module. */
export type AmdContainer = string;

/** Add a comment in the UMD wrapper. */
export type AuxiliaryComment = string | LibraryCustomUmdCommentObject;

/** Specify which export should be exposed as a library. */
export type LibraryExport = string | string[];

/** Configure how the library will be exposed. */
export type LibraryType =
	| string
	| "var"
	| "module"
	| "assign"
	| "assign-properties"
	| "this"
	| "window"
	| "self"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "commonjs-static"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system";

/** When using output.library.type: "umd", setting output.library.umdNamedDefine to true will name the AMD module of the UMD build. */
export type UmdNamedDefine = boolean;

/** Options for library.  */
export type LibraryOptions = {
	/** Use a container(defined in global space) for calling define/require functions in an AMD module. */
	amdContainer?: AmdContainer;

	/** Add a comment in the UMD wrapper. */
	auxiliaryComment?: AuxiliaryComment;

	/** Specify which export should be exposed as a library. */
	export?: LibraryExport;

	/** Specify a name for the library. */
	name?: LibraryName;

	/** Configure how the library will be exposed. */
	type: LibraryType;

	/**
	 * When using output.library.type: "umd", setting output.library.umdNamedDefine to true will name the AMD module of the UMD build.
	 * Otherwise, an anonymous define is used.
	 * */
	umdNamedDefine?: UmdNamedDefine;
};

/** Options for library. */
export type Library = LibraryName | LibraryOptions | undefined;

/** The layer of this entry. */
export type Layer = string | null;

/** The filename of the entry chunk. */
export type EntryFilename = Filename;

/** The name of the runtime chunk. */
export type EntryRuntime = false | string;

/** The path to the entry module. */
export type EntryItem = string | string[];

/** The entry that the current entry depends on. With dependOn option you can share the modules from one entry chunk to another. */
export type EntryDependOn = string | string[];

/**
 * An object with entry point description.
 */
export type EntryDescription = {
	/**
	 * The path to the entry module.
	 * @default './src/index.js'
	 * @example ['./src/index.js', './src/foo.js']
	 * */
	import: EntryItem;

	/**
	 * The name of the runtime chunk.
	 * When runtime is set, a new runtime chunk will be created.
	 * You can also set it to false to avoid a new runtime chunk.
	 * */
	runtime?: EntryRuntime;

	/** The publicPath of the resource referenced by this entry. */
	publicPath?: PublicPath;

	/** The baseURI of the resource referenced by this entry. */
	baseUri?: BaseUri;

	/** How this entry load other chunks. */
	chunkLoading?: ChunkLoading;

	/** Whether to create a load-on-demand asynchronous chunk for this entry. */
	asyncChunks?: AsyncChunks;

	/** Option to set the method of loading WebAssembly Modules. */
	wasmLoading?: WasmLoading;

	/** The filename of the entry chunk. */
	filename?: EntryFilename;

	/**
	 * The format of the chunk generated by this entry as a library.
	 * For detailed configuration, see `output.library`.
	 */
	library?: LibraryOptions;

	/**
	 * The entry that the current entry depends on. With `dependOn` option
	 * you can share the modules from one entry chunk to another.
	 */
	dependOn?: EntryDependOn;

	/**
	 * Specifies the layer in which modules of this entrypoint are placed.
	 * Make the corresponding configuration take effect through layer matching
	 * in split chunks, rules, stats, and externals.
	 */
	layer?: Layer;
};

export type EntryUnnamed = EntryItem;

export type EntryObject = Record<string, EntryItem | EntryDescription>;

/** A static entry.  */
export type EntryStatic = EntryObject | EntryUnnamed;

/** A Function returning entry options. */
export type EntryDynamic = () => EntryStatic | Promise<EntryStatic>;

/** The entry options for building */
export type Entry = EntryStatic | EntryDynamic;
//#endregion

//#region Output
/** The output directory as an absolute path. */
export type Path = string;

/** Tells Rspack to include comments in bundles with information about the contained modules. */
export type Pathinfo = boolean | "verbose";

/** Before generating the products, delete all files in the output directory. */
export type AssetModuleFilename = Filename;

/** Specifies the filename of WebAssembly modules. */
export type WebassemblyModuleFilename = string;

/** This option determines the name of non-initial chunk files. */
export type ChunkFilename = Filename;

/** Allows you to set the [crossorigin attribute](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script)  */
export type CrossOriginLoading = false | "anonymous" | "use-credentials";

/** This option determines the name of CSS output files on disk. */
export type CssFilename = Filename;

/** This option determines the name of non-initial CSS output files on disk. */
export type CssChunkFilename = Filename;

/** Customize the filenames of hot update chunks. */
export type HotUpdateChunkFilename = FilenameTemplate;

/** Customize the main hot update filename. */
export type HotUpdateMainFilename = FilenameTemplate;

/** Which uses JSONP for loading hot updates. */
export type HotUpdateGlobal = string;

/** A unique name of the Rspack build */
export type UniqueName = string;

/** The global variable is used by Rspack for loading chunks. */
export type ChunkLoadingGlobal = string;

/** List of library types enabled for use by entry points. */
export type EnabledLibraryTypes = string[];

/** Whether delete all files in the output directory. */
export type Clean = boolean | { keep?: string };

/** Output JavaScript files as module type. */
export type OutputModule = boolean;

/** Tell Rspack to remove a module from the module instance cache (require.cache) if it throws an exception when it is required. */
export type StrictModuleExceptionHandling = boolean;

/** Handle error in module loading as per EcmaScript Modules spec at a performance cost. */
export type StrictModuleErrorHandling = boolean;

/** Indicates what global object will be used to mount the library. */
export type GlobalObject = string;

/** List of wasm loading types enabled for use by entry points. */
export type EnabledWasmLoadingTypes = string[];

/** The name of the native import() function. */
export type ImportFunctionName = string;

/** The name of the native import.meta object. */
export type ImportMetaName = string;

/** Tells Rspack to add IIFE wrapper around emitted code. */
export type Iife = boolean;

/** List of chunk loading types enabled for use by entry points. */
export type EnabledChunkLoadingTypes = string[];

/** The format of chunks */
export type ChunkFormat = string | false;

/** Set a public path for Worker. */
export type WorkerPublicPath = string;

/** Controls [Trusted Types](https://web.dev/articles/trusted-types) compatibility. */
export type TrustedTypes = {
	/**
	 * The name of the Trusted Types policy created by webpack to serve bundle chunks.
	 */
	policyName?: string;
	/**
	 * If the call to `trustedTypes.createPolicy(...)` fails -- e.g., due to the policy name missing from the CSP `trusted-types` list, or it being a duplicate name, etc. -- controls whether to continue with loading in the hope that `require-trusted-types-for 'script'` isn't enforced yet, versus fail immediately. Default behavior is 'stop'.
	 */
	onPolicyCreationFailure?: "continue" | "stop";
};

/** The encoding to use when generating the hash. */
export type HashDigest = string;

/** The prefix length of the hash digest to use. */
export type HashDigestLength = number;

/** The hashing algorithm to use. */
export type HashFunction = "md4" | "xxhash64";

/** An optional salt to update the hash. */
export type HashSalt = string;

/** Configure how source maps are named. */
export type SourceMapFilename = string;

/** This option determines the module's namespace */
export type DevtoolNamespace = string;

/** This option is only used when devtool uses an option that requires module names. */
export type DevtoolModuleFilenameTemplate = string | ((info: any) => any);

/** A fallback is used when the template string or function above yields duplicates. */
export type DevtoolFallbackModuleFilenameTemplate =
	DevtoolModuleFilenameTemplate;

/** Tell Rspack what kind of ES-features may be used in the generated runtime-code. */
export type Environment = {
	/** The environment supports arrow functions ('() => { ... }'). */
	arrowFunction?: boolean;

	/** The environment supports async function and await ('async function () { await ... }'). */
	asyncFunction?: boolean;

	/** The environment supports BigInt as literal (123n). */
	bigIntLiteral?: boolean;

	/** The environment supports const and let for variable declarations. */
	const?: boolean;

	/** The environment supports destructuring ('{ a, b } = obj'). */
	destructuring?: boolean;

	/** The environment supports 'document' variable. */
	document?: boolean;

	/** The environment supports an async import() function to import EcmaScript modules. */
	dynamicImport?: boolean;

	/** The environment supports an async import() when creating a worker, only for web targets at the moment. */
	dynamicImportInWorker?: boolean;

	/** The environment supports 'for of' iteration ('for (const x of array) { ... }'). */
	forOf?: boolean;

	/** The environment supports 'globalThis'. */
	globalThis?: boolean;

	/** The environment supports ECMAScript Module syntax to import ECMAScript modules (import ... from '...'). */
	module?: boolean;

	/**
	 * Determines if the node: prefix is generated for core module imports in environments that support it.
	 * This is only applicable to Webpack runtime code.
	 * */
	nodePrefixForCoreModules?: boolean;

	/** The environment supports optional chaining ('obj?.a' or 'obj?.()'). */
	optionalChaining?: boolean;

	/** The environment supports template literals. */
	templateLiteral?: boolean;
};

export type Output = {
	/**
	 * The output directory as an absolute path.
	 * @default path.resolve(process.cwd(), 'dist')
	 * */
	path?: Path;

	/**
	 * Tells Rspack to include comments in bundles with information about the contained modules.
	 * @default true
	 */
	pathinfo?: Pathinfo;

	/**
	 * Before generating the products, whether delete all files in the output directory.
	 * @default false
	 * */
	clean?: Clean;

	/** This option determines the URL prefix of the referenced resource, such as: image, file, etc. */
	publicPath?: PublicPath;

	/** This option determines the name of each output bundle. */
	filename?: Filename;

	/** This option determines the name of non-initial chunk files. */
	chunkFilename?: ChunkFilename;

	/** Allows you to set the [crossorigin attribute](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script) for dynamically loaded chunks. */
	crossOriginLoading?: CrossOriginLoading;

	/** This option determines the name of CSS output files on disk. */
	cssFilename?: CssFilename;

	/**
	 * @deprecated this config is unused, and will be removed in the future.
	 * Rspack adds some metadata in CSS to parse CSS modules, and this configuration determines whether to compress these metadata.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 * */
	cssHeadDataCompression?: boolean;

	/** This option determines the name of non-initial CSS output files on disk. */
	cssChunkFilename?: CssChunkFilename;

	/**
	 * Customize the main hot update filename. [fullhash] and [runtime] are available as placeholder.
	 * @default '[runtime].[fullhash].hot-update.json'
	 * */
	hotUpdateMainFilename?: HotUpdateMainFilename;

	/**
	 * Customize the filenames of hot update chunks.
	 * @default '[id].[fullhash].hot-update.js'
	 * */
	hotUpdateChunkFilename?: HotUpdateChunkFilename;

	/**
	 * Only used when target is set to 'web', which uses JSONP for loading hot updates.
	 * @default 'webpackHotUpdate' + output.uniqueName
	 * */
	hotUpdateGlobal?: HotUpdateGlobal;

	/**
	 * This option determines the name of each asset modules.
	 * @default '[hash][ext][query]'
	 * */
	assetModuleFilename?: AssetModuleFilename;

	/** A unique name of the Rspack build to avoid multiple Rspack runtimes to conflict when using globals. */
	uniqueName?: UniqueName;

	/**
	 * The global variable is used by Rspack for loading chunks.
	 * Determined by output.uniqueName default.
	 * */
	chunkLoadingGlobal?: ChunkLoadingGlobal;

	/**
	 * List of library types enabled for use by entry points.
	 * Determined by output.library and Entry default.
	 * */
	enabledLibraryTypes?: EnabledLibraryTypes;

	/** Output a library exposing the exports of your entry point. */
	library?: Library;

	/**
	 * Specify which export should be exposed as a library.
	 * @deprecated We might drop support for this, so prefer to use output.library.export
	 * */
	libraryExport?: LibraryExport;

	/**
	 * Configure how the library will be exposed.
	 * @deprecated Use output.library.type instead as we might drop support for output.libraryTarget in the future.
	 * */
	libraryTarget?: LibraryType;

	/**
	 * When using output.library.type: "umd", setting output.umdNamedDefine to true will name the AMD module of the UMD build.
	 * @deprecated Use output.library.umdNamedDefine instead.
	 */
	umdNamedDefine?: UmdNamedDefine;

	/**
	 * Add a comment in the UMD wrapper.
	 * @deprecated use output.library.auxiliaryComment instead.
	 * */
	auxiliaryComment?: AuxiliaryComment;

	/**
	 * Output JavaScript files as module type.
	 * Disabled by default as it's an experimental feature. To use it, you must set experiments.outputModule to true.
	 * @default false
	 */
	module?: OutputModule;

	/** Tell Rspack to remove a module from the module instance cache (require.cache) if it throws an exception when it is required. */
	strictModuleExceptionHandling?: StrictModuleExceptionHandling;

	/**
	 * Handle error in module loading as per EcmaScript Modules spec at a performance cost.
	 * @default false
	 * */
	strictModuleErrorHandling?: StrictModuleErrorHandling;

	/**
	 * When targeting a library, especially when library.type is 'umd', this option indicates what global object will be used to mount the library.
	 * @default 'self'
	 */
	globalObject?: GlobalObject;

	/**
	 * The name of the native import() function.
	 * @default 'import'
	 * */
	importFunctionName?: ImportFunctionName;

	/**
	 * The name of the native import.meta object (can be exchanged for a polyfill).
	 * @default 'import.meta'
	 */
	importMetaName?: ImportMetaName;

	/**
	 * Tells Rspack to add IIFE wrapper around emitted code.
	 * @default true
	 */
	iife?: Iife;

	/**
	 * Option to set the method of loading WebAssembly Modules.
	 * @default 'fetch'
	 * */
	wasmLoading?: WasmLoading;

	/** List of wasm loading types enabled for use by entry points. */
	enabledWasmLoadingTypes?: EnabledWasmLoadingTypes;

	/**
	 * Specifies the filename of WebAssembly modules.
	 * @default '[hash].module.wasm'
	 * */
	webassemblyModuleFilename?: WebassemblyModuleFilename;

	/** The format of chunks (formats included by default are 'array-push' (web/webworker), 'commonjs' (node.js), 'module' (ESM). */
	chunkFormat?: ChunkFormat;

	/** The method to load chunks (methods included by default are 'jsonp' (web), 'import' (ESM), 'importScripts' (webworker), 'require' (sync node.js), 'async-node' (async node.js) */
	chunkLoading?: ChunkLoading;

	/** List of chunk loading types enabled for use by entry points. */
	enabledChunkLoadingTypes?: EnabledChunkLoadingTypes;

	/** Controls [Trusted Types](https://web.dev/articles/trusted-types) compatibility. */
	trustedTypes?: true | string | TrustedTypes;

	/**
	 * Configure how source maps are named.
	 * Only takes effect when devtool is set to 'source-map', which writes an output file.
	 * @default '[file].map[query]'
	 * */
	sourceMapFilename?: SourceMapFilename;

	/** The encoding to use when generating the hash. */
	hashDigest?: HashDigest;

	/**
	 * The prefix length of the hash digest to use.
	 * @default 16
	 * */
	hashDigestLength?: HashDigestLength;

	/**
	 * The hashing algorithm to use.
	 * @default 'xxhash64'
	 * */
	hashFunction?: HashFunction;

	/** An optional salt to update the hash. */
	hashSalt?: HashSalt;

	/**
	 * Create async chunks that are loaded on demand.
	 * @default true
	 * */
	asyncChunks?: AsyncChunks;

	/**
	 * The new option workerChunkLoading controls the chunk loading of workers.
	 * @default false
	 * */
	workerChunkLoading?: ChunkLoading;

	/**
	 * Option to set the method of loading WebAssembly Modules in workers, defaults to the value of output.wasmLoading.
	 * @default false
	 * */
	workerWasmLoading?: WasmLoading;

	/** Set a public path for Worker, defaults to value of output.publicPath. */
	workerPublicPath?: WorkerPublicPath;

	/**
	 * This option allows loading asynchronous chunks with a custom script type.
	 * @default false
	 * */
	scriptType?: ScriptType;

	/** This option determines the module's namespace used with the output.devtoolModuleFilenameTemplate */
	devtoolNamespace?: DevtoolNamespace;

	/** This option is only used when devtool uses an option that requires module names. */
	devtoolModuleFilenameTemplate?: DevtoolModuleFilenameTemplate;

	/** A fallback is used when the template string or function above yields duplicates. */
	devtoolFallbackModuleFilenameTemplate?: DevtoolFallbackModuleFilenameTemplate;

	/**
	 * The Number of milliseconds before chunk request timed out.
	 * @default 120000
	 * */
	chunkLoadTimeout?: number;

	/**
	 * Add charset="utf-8" to the HTML <script> tag.
	 * @default true
	 * */
	charset?: boolean;

	/** Tell Rspack what kind of ES-features may be used in the generated runtime-code. */
	environment?: Environment;

	/**
	 * Check if to be emitted file already exists and have the same content before writing to output filesystem.
	 */
	compareBeforeEmit?: boolean;
};

//#endregion

//#region Resolve
/**
 * Path alias
 * @example
 * ```js
 * {
 * 	"@": path.resolve(__dirname, './src'),
 * 	"abc$": path.resolve(__dirname, './node_modules/abc/index.js'),
 * }
 * // - require("@/a") will attempt to resolve <root>/src/a.
 * // - require("abc") will attempt to resolve <root>/src/abc.
 * // - require("abc/file.js") will not match, and it will attempt to resolve node_modules/abc/file.js.
 * ```
 * */
export type ResolveAlias = {
	[x: string]: string | false | (string | false)[];
};

/** The replacement of [tsconfig-paths-webpack-plugin](https://www.npmjs.com/package/tsconfig-paths-webpack-plugin) in Rspack. */
export type ResolveTsConfig =
	| string
	| {
			configFile: string;
			references?: string[] | "auto" | undefined;
	  };

/** Used to configure the Rspack module resolution */
export type ResolveOptions = {
	/** Path alias */
	alias?: ResolveAlias;

	/** Same as node's [conditionNames](https://nodejs.org/api/packages.html#conditional-exports) for the exports and imports fields in package.json. */
	conditionNames?: string[];

	/**
	 * Parse modules in order.
	 * @default [".js", ".json", ".wasm"]
	 * */
	extensions?: string[];

	/** Redirect module requests when normal resolving fails. */
	fallback?: ResolveAlias;

	/** Try to parse the fields in package.json */
	mainFields?: string[];

	/**
	 * The filename suffix when resolving directories, e.g. require('. /dir/') will try to resolve '. /dir/index'.
	 * @default ['index']
	 */
	mainFiles?: string[];

	/**
	 * The name of the directory to use when resolving dependencies.
	 * @default ["node_modules"]
	 */
	modules?: string[];

	/**
	 * When enabled, require('file') will first look for the . /file file in the current directory, not <modules>/file.
	 * @default false
	 */
	preferRelative?: boolean;

	/**
	 * Opt for absolute paths when resolving, in relation to resolve.roots.
	 * @default false
	 */
	preferAbsolute?: boolean;

	/**
	 * Whether to resolve symlinks to their symlinked location.
	 * @default true
	 */
	symlinks?: boolean;

	/**
	 * By default, It changes to true if resolve.extensions contains an empty string;
	 * otherwise, this value changes to false.
	 */
	enforceExtension?: boolean;

	/**
	 * Customize the imports field in package.json which are used to provide the internal requests of a package (requests starting with # are considered internal).
	 * @default ["imports"]
	 */
	importsFields?: string[];

	/**
	 * The JSON files to use for descriptions.
	 * @default ['package.json']
	 */
	descriptionFiles?: string[];

	/** The replacement of [tsconfig-paths-webpack-plugin](https://www.npmjs.com/package/tsconfig-paths-webpack-plugin) in Rspack. */
	tsConfig?: ResolveTsConfig;

	/**
	 * No longer resolve extensions, no longer resolve mainFiles in package.json (but does not affect requests from mainFiles, browser, alias).
	 * @default false
	 * */
	fullySpecified?: boolean;

	/**
	 * Customize the exports field in package.json.
	 * @default ["exports"]
	 * */
	exportsFields?: string[];

	/** Define alias for the extension. */
	extensionAlias?: Record<string, string | string[]>;

	/**
	 * Define a field, such as browser, that should be parsed in accordance with this [specification](https://github.com/defunctzombie/package-browser-field-spec).
	 * @default ['browser']
	 * */
	aliasFields?: string[];

	/**
	 * A list of resolve restrictions to restrict the paths that a request can be resolved on.
	 * @default []
	 * */
	restrictions?: string[];

	/**
	 * A list of directories where server-relative URLs (beginning with '/') are resolved.
	 * It defaults to the context configuration option.
	 * On systems other than Windows, these requests are initially resolved as an absolute path.
	 * @default []
	 */
	roots?: string[];

	/** Customize the Resolve configuration based on the module type. */
	byDependency?: Record<string, ResolveOptions>;
	/** enable Yarn PnP */
	pnp?: boolean;
};

/** Used to configure the Rspack module resolution */
export type Resolve = ResolveOptions;
//#endregion

//#region Module
export type RuleSetCondition =
	| string
	| RegExp
	| ((value: string) => boolean)
	| RuleSetConditions
	| RuleSetLogicalConditions;

export type RuleSetConditions = RuleSetCondition[];

export type RuleSetLogicalConditions = {
	and?: RuleSetConditions;
	or?: RuleSetConditions;
	not?: RuleSetCondition;
};

export type RuleSetLoader = string;

export type RuleSetLoaderOptions = string | Record<string, any>;

export type RuleSetLoaderWithOptions = {
	ident?: string;

	loader: RuleSetLoader;

	options?: RuleSetLoaderOptions;
};

export type RuleSetUseItem = RuleSetLoader | RuleSetLoaderWithOptions;

export type RuleSetUse =
	| RuleSetUseItem
	| RuleSetUseItem[]
	| ((data: RawFuncUseCtx) => RuleSetUseItem[]);

/** Rule defines the conditions for matching a module and the behavior of handling those modules. */
export type RuleSetRule = {
	/** Matches all modules that match this resource, and will match against Resource. */
	test?: RuleSetCondition;

	/** Excludes all modules that match this condition and will match against the absolute path of the resource */
	exclude?: RuleSetCondition;

	/** Matches all modules that match this condition against the absolute path of the resource */
	include?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against Resource */
	issuer?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against layer of the module that issued the current module. */
	issuerLayer?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against the category of the dependency that introduced the current module */
	dependency?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against Resource */
	resource?: RuleSetCondition;

	/** Matches all modules that match this resource against the Resource's fragment. */
	resourceFragment?: RuleSetCondition;

	/** Matches all modules that match this resource against the Resource's query. */
	resourceQuery?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against the Resource's mimetype. */
	mimetype?: RuleSetCondition;

	/** Matches all modules that match this resource, and will match against the Resource's scheme. */
	scheme?: RuleSetCondition;

	/** Allows you to match values of properties in the description file, typically package.json, to determine which modules a rule should apply to. */
	descriptionData?: Record<string, RuleSetCondition>;

	/** Used in conjunction with [import attributes](https://github.com/tc39/proposal-import-attributes). */
	with?: Record<string, RuleSetCondition>;

	/** Used to mark the type of the matching module, which affects how the module is handled by Rspack's built-in processing. */
	type?: string;

	/** Used to mark the layer of the matching module. */
	layer?: string;

	/** A loader name */
	loader?: RuleSetLoader;

	/** A loader options */
	options?: RuleSetLoaderOptions;

	/** An array to pass the Loader package name and its options.  */
	use?: RuleSetUse;

	/**
	 * Parser options for the specific modules that matched by the rule conditions
	 * It will override the parser options in module.parser.
	 * @default {}
	 * */
	parser?: Record<string, any>;

	/**
	 * Generator options for the specific modules that matched by the rule conditions
	 * It will override the parser options in module.generator.
	 * @default {}
	 */
	generator?: Record<string, any>;

	/** Matches all modules that match this resource, and will match against Resource. */
	resolve?: ResolveOptions;

	/** Flag the module for side effects */
	sideEffects?: boolean;

	/** Specify loader category.  */
	enforce?: "pre" | "post";

	/** A kind of Nested Rule, an array of Rules from which only the first matching Rule is used when the parent Rule matches. */
	oneOf?: (RuleSetRule | Falsy)[];

	/** A kind of Nested Rule, an array of Rules that is also used when the parent Rule matches. */
	rules?: (RuleSetRule | Falsy)[];
};

/** A list of rules. */
export type RuleSetRules = ("..." | RuleSetRule | Falsy)[];

/**
 * Options object for DataUrl condition.
 * */
export type AssetParserDataUrlOptions = {
	maxSize?: number | undefined;
};

/**
 * Options object for DataUrl condition.
 * */
export type AssetParserDataUrl = AssetParserDataUrlOptions;

/** Options object for `asset` modules. */
export type AssetParserOptions = {
	/**
	 * It be used only for Asset Module scenarios.
	 * @default { maxSize: 8096 }
	 * */
	dataUrlCondition?: AssetParserDataUrlOptions;
};

export type CssParserNamedExports = boolean;

/** Options object for `css` modules. */
export type CssParserOptions = {
	/**
	 * Use ES modules named export for CSS exports.
	 * @default true
	 * */
	namedExports?: CssParserNamedExports;
};

/** Options object for `css/auto` modules. */
export type CssAutoParserOptions = {
	/**
	 * Use ES modules named export for CSS exports.
	 * @default true
	 * */
	namedExports?: CssParserNamedExports;
};

/** Options object for `css/module` modules. */
export type CssModuleParserOptions = {
	/**
	 * Use ES modules named export for CSS exports.
	 * @default true
	 * */
	namedExports?: CssParserNamedExports;
};

type ExportsPresence = "error" | "warn" | "auto" | false;

export type JavascriptParserOptions = {
	/**
	 * Specifies global mode for dynamic import.
	 * @default 'lazy'
	 * */
	dynamicImportMode?: "eager" | "lazy" | "weak" | "lazy-once";

	/**
	 * Specifies global preload for dynamic import.
	 * @default false
	 * */
	dynamicImportPreload?: boolean | number;

	/**
	 * Specifies global prefetch for dynamic import
	 * @default false
	 * */
	dynamicImportPrefetch?: boolean | number;

	/**
	 * Specifies global fetchPriority for dynamic import
	 * @default 'auto'
	 */
	dynamicImportFetchPriority?: "low" | "high" | "auto";

	/**
	 * Enable or disable evaluating import.meta.
	 * @default true
	 */
	importMeta?: boolean;

	/**
	 * Enable parsing of new URL() syntax.
	 * @default true
	 * */
	url?: "relative" | boolean;

	/**
	 * Enable warnings for full dynamic dependencies
	 * @default true
	 * */
	exprContextCritical?: boolean;

	/**
	 * Enable warnings for partial dynamic dependencies
	 * @default false
	 * */
	wrappedContextCritical?: boolean;

	/**
	 * Set the inner regular expression for partial dynamic dependencies
	 * */
	wrappedContextRegExp?: RegExp;

	/**
	 * Warn or error for using non-existent exports and conflicting re-exports.
	 * @default 'auto'
	 */
	exportsPresence?: ExportsPresence;

	/** Warn or error for using non-existent exports */
	importExportsPresence?: ExportsPresence;

	/** Warn or error for conflicting re-exports */
	reexportExportsPresence?: ExportsPresence;

	/** Emit errors instead of warnings when imported names don't exist in imported module. */
	strictExportPresence?: boolean;

	/** Provide custom syntax for Worker parsing, commonly used to support Worklet */
	worker?: string[] | boolean;

	/** Override the module to strict or non-strict. */
	overrideStrict?: "strict" | "non-strict";

	// TODO: add docs
	requireAsExpression?: boolean;

	// TODO: add docs
	requireDynamic?: boolean;

	// TODO: add docs
	requireResolve?: boolean;

	// TODO: add docs
	importDynamic?: boolean;
};

export type JsonParserOptions = {
	/**
	 * The depth of json dependency flagged as `exportInfo`.
	 */
	exportsDepth?: number;
	/**
	 * If Rule.type is set to 'json' then Rules.parser.parse option may be a function that implements custom logic to parse module's source and convert it to a json-compatible data.
	 */
	parse?: (source: string) => any;
};

/** Configure all parsers' options in one place with module.parser. */
export type ParserOptionsByModuleTypeKnown = {
	/** Parser options for `asset` modules. */
	asset?: AssetParserOptions;

	/** Parser options for `css` modules. */
	css?: CssParserOptions;

	/** Parser options for `css/auto` modules. */
	"css/auto"?: CssAutoParserOptions;

	/** Parser options for `css/module` modules. */
	"css/module"?: CssModuleParserOptions;

	/** Parser options for `javascript` modules. */
	javascript?: JavascriptParserOptions;

	/** Parser options for `javascript/auto` modules. */
	"javascript/auto"?: JavascriptParserOptions;

	/** Parser options for `javascript/dynamic` modules. */
	"javascript/dynamic"?: JavascriptParserOptions;

	/** Parser options for `javascript/esm` modules. */
	"javascript/esm"?: JavascriptParserOptions;

	/** Parser options for `json` modules. */
	json?: JsonParserOptions;
};

/** Configure all parsers' options in one place with module.parser. */
export type ParserOptionsByModuleTypeUnknown = {
	[x: string]: Record<string, any>;
};

/** Configure all parsers' options in one place with module.parser. */
export type ParserOptionsByModuleType =
	| ParserOptionsByModuleTypeKnown
	| ParserOptionsByModuleTypeUnknown;

export type AssetGeneratorDataUrlOptions = {
	encoding?: false | "base64";
	mimetype?: string;
};

export type AssetGeneratorDataUrlFunction = (
	content: Buffer,
	context: {
		filename: string;
		module: Module;
	}
) => string;

export type AssetGeneratorDataUrl =
	| AssetGeneratorDataUrlOptions
	| AssetGeneratorDataUrlFunction;

/** Options for asset inline modules. */
export type AssetInlineGeneratorOptions = {
	/** Only for modules with module type 'asset' or 'asset/inline'. */
	dataUrl?: AssetGeneratorDataUrl;
};

/** Emit the asset in the specified folder relative to 'output.path'. */
export type AssetModuleOutputPath = Filename;

/**
 * If "url", a URL pointing to the asset will be generated based on publicPath.
 * If "preserve", preserve import/require statement from generated asset.
 * Only for modules with module type 'asset' or 'asset/resource'.
 * @default "url"
 */
export type AssetModuleImportMode = "url" | "preserve";

/** Options for asset modules. */
export type AssetResourceGeneratorOptions = {
	/**
	 * Whether to output assets to disk.
	 * @default true
	 * */
	emit?: boolean;

	/** This option determines the name of each asset resource output bundle.*/
	filename?: Filename;

	/** Emit the asset in the specified folder relative to 'output.path' */
	outputPath?: AssetModuleOutputPath;

	/** This option determines the URL prefix of the referenced 'asset' or 'asset/resource'*/
	publicPath?: PublicPath;

	/**
	 * If "url", a URL pointing to the asset will be generated based on publicPath.
	 * If "preserve", preserve import/require statement from generated asset.
	 * Only for modules with module type 'asset' or 'asset/resource'.
	 * @default "url"
	 */
	importMode?: AssetModuleImportMode;
};

/** Generator options for asset modules. */
export type AssetGeneratorOptions = AssetInlineGeneratorOptions &
	AssetResourceGeneratorOptions;

export type CssGeneratorExportsConvention =
	| "as-is"
	| "camel-case"
	| "camel-case-only"
	| "dashes"
	| "dashes-only";

export type CssGeneratorExportsOnly = boolean;

export type CssGeneratorLocalIdentName = string;

export type CssGeneratorEsModule = boolean;

/** Generator options for css modules. */
export type CssGeneratorOptions = {
	/**
	 * If true, only exports the identifier mappings from CSS into the output JavaScript files
	 * If false, generate stylesheets and embed them in the template.
	 */
	exportsOnly?: CssGeneratorExportsOnly;

	/** This configuration is available for improved ESM-CJS interoperability purposes. */
	esModule?: CssGeneratorEsModule;
};

/** Generator options for css/auto modules. */
export type CssAutoGeneratorOptions = {
	/**
	 * Customize how CSS export names are exported to javascript modules
	 * @default 'as-is'
	 * */
	exportsConvention?: CssGeneratorExportsConvention;

	/**
	 * If true, only exports the identifier mappings from CSS into the output JavaScript files
	 * If false, generate stylesheets and embed them in the template.
	 */
	exportsOnly?: CssGeneratorExportsOnly;

	/** Customize the format of the local class names generated for CSS modules */
	localIdentName?: CssGeneratorLocalIdentName;

	/** This configuration is available for improved ESM-CJS interoperability purposes. */
	esModule?: CssGeneratorEsModule;
};

/** Generator options for css/module modules. */
export type CssModuleGeneratorOptions = CssAutoGeneratorOptions;

/** Generator options for json modules. */
export type JsonGeneratorOptions = {
	/**
	 * Use `JSON.parse` when the JSON string is longer than 20 characters.
	 * @default true
	 */
	JSONParse?: boolean;
};

export type GeneratorOptionsByModuleTypeKnown = {
	/** Generator options for asset modules. */
	asset?: AssetGeneratorOptions;

	/** Generator options for asset/inline modules. */
	"asset/inline"?: AssetInlineGeneratorOptions;

	/** Generator options for asset/resource modules. */
	"asset/resource"?: AssetResourceGeneratorOptions;

	/** Generator options for css modules. */
	css?: CssGeneratorOptions;

	/** Generator options for css/auto modules. */
	"css/auto"?: CssAutoGeneratorOptions;

	/** Generator options for css/module modules. */
	"css/module"?: CssModuleGeneratorOptions;

	/** Generator options for json modules. */
	json?: JsonGeneratorOptions;
};

export type GeneratorOptionsByModuleTypeUnknown = Record<
	string,
	Record<string, any>
>;

/** Options for module.generator */
export type GeneratorOptionsByModuleType =
	| GeneratorOptionsByModuleTypeKnown
	| GeneratorOptionsByModuleTypeUnknown;

type NoParseOptionSingle = string | RegExp | ((request: string) => boolean);

/** Options for module.noParse */
export type NoParseOption = NoParseOptionSingle | NoParseOptionSingle[];

export type ModuleOptions = {
	/** Used to decide how to handle different types of modules in a project. */
	defaultRules?: RuleSetRules;

	/**
	 * An array of rules that match the module's requests when it is created.
	 * @default []
	 * */
	rules?: RuleSetRules;

	/**
	 * Configure all parsers' options in one place with module.parser.
	 * @default {}
	 * */
	parser?: ParserOptionsByModuleType;

	/** Configure all generators' options in one place with module.generator. */
	generator?: GeneratorOptionsByModuleType;

	/** Keep module mechanism of the matched modules as-is, such as module.exports, require, import. */
	noParse?: NoParseOption;
};

//#endregion

//#region Target
type AllowTarget =
	| "web"
	| "webworker"
	| "es3"
	| "es5"
	| "es2015"
	| "es2016"
	| "es2017"
	| "es2018"
	| "es2019"
	| "es2020"
	| "es2021"
	| "es2022"
	| "node"
	| "async-node"
	| `node${number}`
	| `async-node${number}`
	| `node${number}.${number}`
	| `async-node${number}.${number}`
	| "electron-main"
	| `electron${number}-main`
	| `electron${number}.${number}-main`
	| "electron-renderer"
	| `electron${number}-renderer`
	| `electron${number}.${number}-renderer`
	| "electron-preload"
	| `electron${number}-preload`
	| `electron${number}.${number}-preload`
	| "nwjs"
	| `nwjs${number}`
	| `nwjs${number}.${number}`
	| "node-webkit"
	| `node-webkit${number}`
	| `node-webkit${number}.${number}`
	| "browserslist"
	| `browserslist:${string}`;

/** Used to configure the target environment of Rspack output and the ECMAScript version of Rspack runtime code. */
export type Target = false | AllowTarget | AllowTarget[];
//#endregion

//#region ExternalsType
/**
 * Specify the default type of externals.
 * `amd`, `umd`, `system` and `jsonp` externals depend on the `output.libraryTarget` being set to the same value e.g. you can only consume amd externals within an amd library.
 * @default 'var'
 */
export type ExternalsType =
	| "var"
	| "module"
	| "assign"
	| "this"
	| "window"
	| "self"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "commonjs-static"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system"
	| "promise"
	| "import"
	| "module-import"
	| "script"
	| "node-commonjs"
	| "commonjs-import";
//#endregion

//#region Externals

/**
 * External item object when both libraryTarget and externalsType is 'umd'
 */
export type ExternalItemUmdValue = {
	root: string | string[];
	commonjs: string | string[];
	commonjs2: string | string[];
	amd: string | string[];
};

/**
 * External item object when not umd
 */
export type ExternalItemObjectValue = Record<string, string | string[]>;

/**
 * The dependency used for the external.
 */
export type ExternalItemValue =
	| string
	| boolean
	| string[]
	| ExternalItemUmdValue
	/**
	 * when libraryTarget and externalsType is not 'umd'
	 */
	| ExternalItemObjectValue;

/**
 * If an dependency matches exactly a property of the object, the property value is used as dependency.
 */
export type ExternalItemObjectUnknown = {
	[x: string]: ExternalItemValue;
};

/**
 * Data object passed as argument when a function is set for 'externals'.
 */
export type ExternalItemFunctionData = {
	context?: string;
	dependencyType?: string;
	request?: string;
	contextInfo?: {
		issuer: string;
		issuerLayer?: string | null;
	};
	/**
	 * Get a resolve function with the current resolver options.
	 */
	getResolve?: (
		options?: ResolveOptions
	) =>
		| ((
				context: string,
				request: string,
				callback: (err?: Error, result?: string) => void
		  ) => void)
		| ((context: string, request: string) => Promise<string>);
};

/**
 * Prevent bundling of certain imported package and instead retrieve these external dependencies at runtime.
 *
 * @example
 * ```js
 * // jquery lib will be excluded from bundling.
 * module.exports = {
 * 	externals: {
 * 		jquery: 'jQuery',
 * 	}
 * }
 * ```
 * */
export type ExternalItem =
	| string
	| RegExp
	| ExternalItemObjectUnknown
	| ((data: ExternalItemFunctionData) => ExternalItemValue)
	| ((
			data: ExternalItemFunctionData,
			callback: (
				err?: Error,
				result?: ExternalItemValue,
				type?: ExternalsType
			) => void
	  ) => void)
	| ((data: ExternalItemFunctionData) => Promise<ExternalItemValue>);

/**
 * Prevent bundling of certain imported packages and instead retrieve these external dependencies at runtime.
 *
 * @example
 * ```js
 * // jquery lib will be excluded from bundling.
 * module.exports = {
 * 	externals: {
 * 		jquery: 'jQuery',
 * 	}
 * }
 * ```
 * */
export type Externals = ExternalItem | ExternalItem[];
//#endregion

//#region ExternalsPresets
/** Enable presets of externals for specific targets. */
export type ExternalsPresets = {
	/** Treat node.js built-in modules like `fs`, `path` or `vm` as external and load them via `require()` when used. */
	node?: boolean;

	/** Treat references to `http(s)://...` and `std:...` as external and load them via import when used. */
	web?: boolean;

	/** Treat references to `http(s)://...` and `std:...` as external and load them via async import() when used  */
	webAsync?: boolean;

	/** Treat common electron built-in modules in main and preload context like `electron`, `ipc` or `shell` as external and load them via `require()` when used. */
	electron?: boolean;

	/** Treat electron built-in modules in the main context like `app`, `ipc-main` or `shell` as external and load them via `require()` when used. */
	electronMain?: boolean;

	/** Treat electron built-in modules in the preload context like `web-frame`, `ipc-renderer` or `shell` as external and load them via require() when used. */
	electronPreload?: boolean;

	/** Treat electron built-in modules in the preload context like `web-frame`, `ipc-renderer` or `shell` as external and load them via require() when used. */
	electronRenderer?: boolean;

	/** Treat `NW.js` legacy `nw.gui` module as external and load it via `require()` when used. */
	nwjs?: boolean;
};

//#endregion

//#region InfrastructureLogging
/**
 * Represents a filter item type for infrastructure logging.
 * Can be a RegExp, a string, or a function that takes a string and returns a boolean.
 */
export type FilterItemTypes = RegExp | string | ((value: string) => boolean);

/**
 * Represents filter types for infrastructure logging.
 * Can be a single FilterItemTypes or an array of FilterItemTypes.
 */
export type FilterTypes = FilterItemTypes | FilterItemTypes[];

/**
 * Options for infrastructure level logging.
 */
export type InfrastructureLogging = {
	/**
	 * Append lines to the output instead of updating existing output, useful for status messages.
	 */
	appendOnly?: boolean;

	/**
	 * Enable colorful output for infrastructure level logging.
	 */
	colors?: boolean;

	/**
	 * Customize the console used for infrastructure level logging.
	 */
	console?: Console;

	/**
	 * Enable debug information of specified loggers such as plugins or loaders.
	 */
	debug?: boolean | FilterTypes;

	/**
	 * Enable infrastructure logging output.
	 */
	level?: "none" | "error" | "warn" | "info" | "log" | "verbose";

	/**
	 * Stream used for logging output.
	 */
	stream?: NodeJS.WritableStream;
};
//#endregion

//#region DevTool
/**
 * Configuration used to control the behavior of the Source Map generation.
 */
export type DevTool =
	| false
	| "eval"
	| "cheap-source-map"
	| "cheap-module-source-map"
	| "source-map"
	| "inline-cheap-source-map"
	| "inline-cheap-module-source-map"
	| "inline-source-map"
	| "inline-nosources-cheap-source-map"
	| "inline-nosources-cheap-module-source-map"
	| "inline-nosources-source-map"
	| "nosources-cheap-source-map"
	| "nosources-cheap-module-source-map"
	| "nosources-source-map"
	| "hidden-nosources-cheap-source-map"
	| "hidden-nosources-cheap-module-source-map"
	| "hidden-nosources-source-map"
	| "hidden-cheap-source-map"
	| "hidden-cheap-module-source-map"
	| "hidden-source-map"
	| "eval-cheap-source-map"
	| "eval-cheap-module-source-map"
	| "eval-source-map"
	| "eval-nosources-cheap-source-map"
	| "eval-nosources-cheap-module-source-map"
	| "eval-nosources-source-map";
//#endregion

//#region Node
/**
 * Options for mocking Node.js globals and modules.
 */
export type NodeOptions = {
	/**
	 * Controls the behavior of `__dirname`.
	 * @description
	 * - `true`: The dirname of the input file relative to the context option.
	 * - `false`: Regular Node.js `__dirname` behavior. The dirname of the output file when run in a Node.js environment.
	 * - `"mock"`: The fixed value '/'.
	 * - `"warn-mock"`: Use the fixed value of '/' but show a warning.
	 * - `"node-module"`: Replace `__dirname` in CommonJS modules to `fileURLToPath(import.meta.url + "/..")` when `output.module` is enabled.
	 * - `"eval-only"`: Equivalent to `false`.
	 */
	__dirname?: boolean | "warn-mock" | "mock" | "eval-only" | "node-module";

	/**
	 * Controls the behavior of `__filename`.
	 * @description
	 * - `true`: The filename of the input file relative to the context option.
	 * - `false`: Regular Node.js `__filename` behavior. The filename of the output file when run in a Node.js environment.
	 * - `"mock"`: The fixed value '/index.js'.
	 * - `"warn-mock"`: Use the fixed value of '/index.js' but show a warning.
	 * - `"node-module"`: Replace `__filename` in CommonJS modules to `fileURLToPath(import.meta.url)` when `output.module` is enabled.
	 * - `"eval-only"`: Equivalent to `false`.
	 */
	__filename?: boolean | "warn-mock" | "mock" | "eval-only" | "node-module";

	/**
	 * Controls the behavior of `global`.
	 * @description
	 * - `true`: Provide a polyfill.
	 * - `false`: Don't provide a polyfill.
	 * - `"warn"`: Provide a polyfill but show a warning.
	 * @see {@link https://nodejs.org/api/globals.html#globals_global | Node.js documentation} for the exact behavior of this object.
	 * @default "warn"
	 */
	global?: boolean | "warn";
};

/**
 * Options for mocking Node.js globals and modules.
 * @description Set to `false` to disable all mocking, or use `NodeOptions` to configure specific behaviors.
 */
export type Node = false | NodeOptions;

export type Loader = Record<string, any>;
//#endregion

//#region Snapshot
export type SnapshotOptions = {};
//#endregion

//#region Cache
/**
 * Options for caching snapshots and intermediate products during the build process.
 * @description Controls whether caching is enabled or disabled.
 * @default true in development mode, false in production mode
 * @example
 * // Enable caching
 * cache: true
 *
 * // Disable caching
 * cache: false
 */
export type CacheOptions = boolean;
//#endregion

//#region Stats

type StatsPresets =
	| "normal"
	| "none"
	| "verbose"
	| "errors-only"
	| "errors-warnings"
	| "minimal"
	| "detailed"
	| "summary";

type ModuleFilterItemTypes =
	| RegExp
	| string
	| ((name: string, module: any, type: any) => boolean);

type ModuleFilterTypes =
	| boolean
	| ModuleFilterItemTypes
	| ModuleFilterItemTypes[];

/** Options for stats */
export type StatsOptions = {
	/**
	 * Enables or disables the display of all stats.
	 */
	all?: boolean;
	/**
	 * Sets the preset for stats or enables/disables them.
	 */
	preset?: boolean | StatsPresets;
	/**
	 * Enables or disables the display of asset stats.
	 * @default true
	 */
	assets?: boolean;
	/**
	 * Enables or disables the display of chunk stats.
	 * @default true
	 */
	chunks?: boolean;
	/**
	 * Enables or disables the display of module stats.
	 * @default true
	 */
	modules?: boolean;
	/**
	 * Enables or disables the display of entrypoint stats or sets it to 'auto'.
	 * @default false
	 */
	entrypoints?: boolean | "auto";
	/**
	 * Enables or disables the display of chunk group stats.
	 * @default true
	 */
	chunkGroups?: boolean;
	/**
	 * Enables or disables the display of warning stats.
	 * @default true
	 */
	warnings?: boolean;
	/**
	 * Enables or disables the display of warning counts.
	 * @default true
	 */
	warningsCount?: boolean;
	/**
	 * Enables or disables the display of error stats.
	 * @default true
	 */
	errors?: boolean;
	/**
	 * Enables or disables the display of error counts.
	 * @default true
	 */
	errorsCount?: boolean;
	/**
	 * Enables or disables the use of colors in the output.
	 * @default false
	 */
	colors?: boolean;
	/**
	 * Enables or disables the display of the hash.
	 * @default true
	 */
	hash?: boolean;
	/**
	 * Enables or disables the display of the version.
	 * @default true
	 */
	version?: boolean;
	/**
	 * Enables or disables the display of reasons.
	 * @default true
	 */
	reasons?: boolean;
	/**
	 * Enables or disables the display of the public path.
	 * @default true
	 */
	publicPath?: boolean;
	/**
	 * Enables or disables the display of the output path.
	 * @default true
	 */
	outputPath?: boolean;
	/**
	 * Enables or disables the display of chunk module stats.
	 * @default true
	 */
	chunkModules?: boolean;
	/**
	 * Enables or disables the display of chunk relations.
	 * @default false
	 */
	chunkRelations?: boolean;
	/**
	 * Enables or disables the display of module IDs.
	 * @default false
	 */
	ids?: boolean;
	/**
	 * Enables or disables the display of build timings.
	 * @default true
	 */
	timings?: boolean;
	/**
	 * Enables or disables the display of the build date.
	 * @default true
	 */
	builtAt?: boolean;
	/**
	 * Enables or disables the display of module assets.
	 * @default true
	 */
	moduleAssets?: boolean;
	/**
	 * Enables or disables the display of nested modules.
	 * @default true
	 */
	nestedModules?: boolean;
	/**
	 * Enables or disables the display of source code.
	 * @default false
	 */
	source?: boolean;
	/**
	 * Configures the level of logging output.
	 * Can be set to a string value of "none", "error", "warn", "info", "log", "verbose", or a boolean value.
	 *
	 * @description
	 * - `'none'`, false: Logging is disabled.
	 * - `'error'`: Only errors are logged.
	 * - `'warn'`: Errors and warnings are logged.
	 * - `'info'`: Errors, warnings, and info messages are logged.
	 * - `'log'`, true: Errors, warnings, info messages, log messages, groups, and clears are logged. Collapsed groups are initially collapsed.
	 * - `'verbose'`: All log levels except debug and trace are logged. Collapsed groups are initially expanded.
	 */
	logging?: "none" | "error" | "warn" | "info" | "log" | "verbose" | boolean;
	/**
	 * Enables or disables debug logging, or specifies a filter for debug logging.
	 */
	loggingDebug?: boolean | FilterTypes;
	/**
	 * Enables or disables trace logging.
	 * @default true
	 */
	loggingTrace?: boolean;
	/**
	 * Enables or disables the display of runtime modules.
	 * @default true
	 */
	runtimeModules?: boolean;
	/**
	 * Enables or disables the display of children modules.
	 * @default true
	 */
	children?: boolean;
	/**
	 * Enables or disables the display of used exports.
	 * @default false
	 */
	usedExports?: boolean;
	/**
	 * Enables or disables the display of provided exports.
	 * @default false
	 */
	providedExports?: boolean;
	/**
	 * Enables or disables optimization bailout.
	 * @default false
	 */
	optimizationBailout?: boolean;
	/**
	 * Enables or disables grouping of modules by type.
	 */
	groupModulesByType?: boolean;
	/**
	 * Enables or disables grouping of modules by cache status.
	 */
	groupModulesByCacheStatus?: boolean;
	/**
	 * Enables or disables grouping of modules by layer.
	 */
	groupModulesByLayer?: boolean;
	/**
	 * Enables or disables grouping of modules by attributes.
	 */
	groupModulesByAttributes?: boolean;
	/**
	 * Enables or disables grouping of modules by path.
	 */
	groupModulesByPath?: boolean;
	/**
	 * Enables or disables grouping of modules by extension.
	 */
	groupModulesByExtension?: boolean;
	/**
	 * Specifies the space to use for displaying modules.
	 * @default 15
	 */
	modulesSpace?: number;
	/**
	 * Specifies the space to use for displaying chunk modules.
	 * @default 10
	 */
	chunkModulesSpace?: number;
	/**
	 * Specifies the space to use for displaying nested modules.
	 * @default 10
	 */
	nestedModulesSpace?: number;
	/**
	 * Enables or disables the display of related assets.
	 * @default false
	 */
	relatedAssets?: boolean;
	/**
	 * Enables or disables grouping of assets by emit status.
	 */
	groupAssetsByEmitStatus?: boolean;
	/**
	 * Enables or disables grouping of assets by info.
	 */
	groupAssetsByInfo?: boolean;
	/**
	 * Enables or disables grouping of assets by path.
	 */
	groupAssetsByPath?: boolean;
	/**
	 * Enables or disables grouping of assets by extension.
	 */
	groupAssetsByExtension?: boolean;
	/**
	 * Enables or disables grouping of assets by chunk.
	 */
	groupAssetsByChunk?: boolean;
	/**
	 * Specifies the space to use for displaying assets.
	 * @default 15
	 */
	assetsSpace?: number;
	/**
	 * Enables or disables the display of orphan modules.
	 * @default false
	 */
	orphanModules?: boolean;
	/**
	 * Specifies modules to exclude from the bundle.
	 * @default false
	 */
	excludeModules?: ModuleFilterTypes;
	/**
	 * Exclude the matching assets information.
	 * @default false
	 */
	excludeAssets?: ModuleFilterTypes;
	/**
	 * Specifies the sorting order for modules.
	 * @default 'id'
	 */
	modulesSort?: string;
	/**
	 * Specifies the sorting order for chunk modules.
	 */
	chunkModulesSort?: string;
	/**
	 * Specifies the sorting order for nested modules.
	 */
	nestedModulesSort?: string;
	/**
	 * Specifies the sorting order for chunks.
	 * @default 'id'
	 */
	chunksSort?: string;
	/**
	 * Specifies the sorting order for assets.
	 * @default 'id'
	 */
	assetsSort?: string;
	/**
	 * Enables or disables performance optimization.
	 * @default true
	 */
	performance?: boolean;
	/**
	 * Enables or disables environment variables.
	 * @default false
	 */
	env?: boolean;
	/**
	 * Enables or disables auxiliary chunk grouping.
	 * @default true
	 */
	chunkGroupAuxiliary?: boolean;
	/**
	 * Enables or disables child chunk grouping.
	 * @default true
	 */
	chunkGroupChildren?: boolean;
	/**
	 * Specifies the maximum number of assets per chunk group.
	 * @default 5
	 */
	chunkGroupMaxAssets?: number;
	/**
	 * Enables or disables the display of dependent modules.
	 * @default false
	 */
	dependentModules?: boolean;
	/**
	 * Enables or disables the display of chunk origins.
	 * @default true
	 */
	chunkOrigins?: boolean;
	/**
	 * Enables or disables the display of runtime information.
	 */
	runtime?: boolean;
	/**
	 * Enables or disables the display of depth information.
	 * @default false
	 */
	depth?: boolean;
	/**
	 * Specifies the space to use for displaying reasons.
	 * @default 100
	 */
	reasonsSpace?: number;
	/**
	 * Enables or disables grouping of reasons by origin.
	 */
	groupReasonsByOrigin?: boolean;
	/**
	 * Enables or disables the display of error details.
	 * @default false
	 */
	errorDetails?: boolean;
	/**
	 * Enables or disables the display of error stack traces.
	 * @default true
	 */
	errorStack?: boolean;
	/**
	 * Enables or disables the display of module trace information.
	 * @default true
	 */
	moduleTrace?: boolean;
	/**
	 * Enables or disables the display of cached modules.
	 * @default true
	 */
	cachedModules?: boolean;
	/**
	 * Enables or disables the display of cached assets.
	 * @default true
	 */
	cachedAssets?: boolean;
	/**
	 * Enables or disables the display of cached information.
	 */
	cached?: boolean;
	/**
	 * Specifies the space to use for displaying errors.
	 * @default 5
	 */
	errorsSpace?: number;
	/**
	 * Specifies the space to use for displaying warnings.
	 * @default 5
	 */
	warningsSpace?: number;
};

/**
 * Represents the value for stats configuration.
 */
export type StatsValue = boolean | StatsOptions | StatsPresets;
//#endregion

//#region Plugins
export interface RspackPluginInstance {
	apply: (compiler: Compiler) => void;
	[k: string]: any;
}

export type RspackPluginFunction = (this: Compiler, compiler: Compiler) => void;

// The Compiler type of webpack is not exactly the same as Rspack.
// It is allowed to use webpack plugins in in the Rspack config,
// so we have defined a loose type here to adapt to webpack plugins.
export type WebpackCompiler = any;

export interface WebpackPluginInstance {
	apply: (compiler: WebpackCompiler) => void;
	[k: string]: any;
}

export type WebpackPluginFunction = (
	this: WebpackCompiler,
	compiler: WebpackCompiler
) => void;

export type Plugin =
	| RspackPluginInstance
	| RspackPluginFunction
	| WebpackPluginInstance
	| WebpackPluginFunction
	| Falsy;

export type Plugins = Plugin[];
//#endregion

//#region Optimization
/** Used to control how the runtime chunk is generated. */

export type OptimizationRuntimeChunk =
	| boolean
	| "single"
	| "multiple"
	| {
			name?: string | ((value: { name: string }) => string);
	  };

export type OptimizationSplitChunksNameFunction = (
	module: Module,
	chunks: Chunk[],
	cacheGroupKey: string
) => string | undefined;

type OptimizationSplitChunksName =
	| string
	| false
	| OptimizationSplitChunksNameFunction;

type OptimizationSplitChunksSizes = number | Record<string, number>;

type OptimizationSplitChunksChunks =
	| "initial"
	| "async"
	| "all"
	| RegExp
	| ((chunk: Chunk) => boolean);

type SharedOptimizationSplitChunksCacheGroup = {
	/**
	 * This indicates which chunks will be selected for optimization.
	 * @default 'async''
	 * */
	chunks?: OptimizationSplitChunksChunks;

	/** Sets the size types which are used when a number is used for sizes. */
	defaultSizeTypes?: string[];

	/**
	 * The minimum times must a module be shared among chunks before splitting.
	 * @default 1
	 */
	minChunks?: number;

	/**
	 * Enabling this, the splitting of chunks will be grouped based on the usage of modules exports in different runtimes,
	 * ensuring the optimal loading size in each runtime.
	 */
	usedExports?: boolean;

	/**
	 * The name of the split chunk.
	 * @default false
	 * */
	name?: false | OptimizationSplitChunksName;

	/** Allows to override the filename when and only when it's an initial chunk. */
	filename?: Filename;

	/**
	 * Minimum size, in bytes, for a chunk to be generated.
	 *
	 * The value is `20000` in production mode.
	 * The value is `10000` in others mode.
	 */
	minSize?: OptimizationSplitChunksSizes;

	minSizeReduction?: OptimizationSplitChunksSizes;

	/** Maximum size, in bytes, for a chunk to be generated. */
	maxSize?: OptimizationSplitChunksSizes;

	/** Maximum size, in bytes, for a async chunk to be generated. */
	maxAsyncSize?: OptimizationSplitChunksSizes;

	/** Maximum size, in bytes, for a initial chunk to be generated. */
	maxInitialSize?: OptimizationSplitChunksSizes;

	/**
	 * Maximum number of parallel requests when on-demand loading.
	 * @default 30
	 * */
	maxAsyncRequests?: number;

	/**
	 * Maximum number of parallel requests at an entry point.
	 * @default 30
	 */
	maxInitialRequests?: number;

	/**
	 * Tell Rspack what delimiter to use for the generated names.
	 *
	 * @default '-''
	 */
	automaticNameDelimiter?: string;
};

export type OptimizationSplitChunksCacheGroupTestFn = (
	module: Module,
	ctx: {
		chunkGraph: ChunkGraph;
		moduleGraph: ModuleGraph;
	}
) => boolean;

/** How to splitting chunks. */
export type OptimizationSplitChunksCacheGroup = {
	/** Controls which modules are selected by this cache group. */
	test?: string | RegExp | OptimizationSplitChunksCacheGroupTestFn;

	/**
	 * A module can belong to multiple cache groups.
	 * @default -20
	 */
	priority?: number;

	/**
	 * Tells Rspack to ignore `splitChunks.minSize`, `splitChunks.minChunks`, `splitChunks.maxAsyncRequests` and `splitChunks.maxInitialRequests` options and always create chunks for this cache group.
	 */
	enforce?: boolean;

	/**
	 * Whether to reuse existing chunks when possible.
	 * @default false
	 * */
	reuseExistingChunk?: boolean;

	/** Allows to assign modules to a cache group by module type. */
	type?: string | RegExp;

	/** Sets the hint for chunk id. It will be added to chunk's filename. */
	idHint?: string;

	/**
	 * Assign modules to a cache group by module layer.
	 */
	layer?: string | ((layer?: string) => boolean) | RegExp;
} & SharedOptimizationSplitChunksCacheGroup;

/** Tell Rspack how to splitting chunks. */
export type OptimizationSplitChunksOptions = {
	/**
	 * Options for module cache group
	 * */
	cacheGroups?: Record<string, false | OptimizationSplitChunksCacheGroup>;

	/**
	 * Options for modules not selected by any other group.
	 */
	fallbackCacheGroup?: {
		chunks?: OptimizationSplitChunksChunks;
		minSize?: number;
		maxSize?: number;
		maxAsyncSize?: number;
		maxInitialSize?: number;
		automaticNameDelimiter?: string;
	};

	/**
	 * Prevents exposing path info when creating names for parts splitted by maxSize.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 * */
	hidePathInfo?: boolean;
} & SharedOptimizationSplitChunksCacheGroup;

export type Optimization = {
	/**
	 * Which algorithm to use when choosing module ids.
	 */
	moduleIds?: "named" | "natural" | "deterministic";

	/**
	 * Which algorithm to use when choosing chunk ids.
	 */
	chunkIds?: "natural" | "named" | "deterministic" | "size" | "total-size";

	/**
	 * Whether to minimize the bundle.
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 */
	minimize?: boolean;

	/**
	 * Customize the minimizer.
	 * By default, `rspack.SwcJsMinimizerRspackPlugin` and `rspack.LightningCssMinimizerRspackPlugin` are used.
	 */
	minimizer?: Array<"..." | Plugin>;

	/**
	 * Whether to merge chunks which contain the same modules.
	 * Setting optimization.mergeDuplicateChunks to false will disable this optimization.
	 * @default true
	 */
	mergeDuplicateChunks?: boolean;

	/**
	 * Support splitting chunks.
	 * It is enabled by default for dynamically imported modules.
	 * To turn it off, set it to false.
	 * */
	splitChunks?: false | OptimizationSplitChunksOptions;

	/**
	 * Used to control how the runtime chunk is generated.
	 * Setting it to true or 'multiple' will add an additional chunk containing only the runtime for each entry point.
	 * Setting it to 'single' will extract the runtime code of all entry points into a single separate chunk.
	 * @default false
	 */
	runtimeChunk?: OptimizationRuntimeChunk;

	/** Detect and remove modules from chunks these modules are already included in all parents. */
	removeAvailableModules?: boolean;

	/**
	 * Remove empty chunks generated in the compilation.
	 * @default true
	 * */
	removeEmptyChunks?: boolean;

	/**
	 * Adds an additional hash compilation pass after the assets have been processed to get the correct asset content hashes.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 */
	realContentHash?: boolean;

	/**
	 * Tells Rspack to recognise the sideEffects flag in package.json or rules to skip over modules which are flagged to contain no side effects when exports are not used.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 * */
	sideEffects?: "flag" | boolean;

	/**
	 * After enabling, Rspack will analyze which exports the module provides, including re-exported modules.
	 * @default true
	 * */
	providedExports?: boolean;

	/**
	 * Tells Rspack to find segments of the module graph which can be safely concatenated into a single module.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 */
	concatenateModules?: boolean;

	/**
	 * Tells Rspack whether to perform a more detailed analysis of variable assignments.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 */
	innerGraph?: boolean;

	/**
	 * Tells Rspack to determine used exports for each module.
	 *
	 * The value is `true` in production mode.
	 * The value is `false` in development mode.
	 * */
	usedExports?: "global" | boolean;

	/**
	 * Allows to control export mangling.
	 *
	 * The value is `isdeterministic` in production mode.
	 * The value is `false` in development mode.
	 */
	mangleExports?: "size" | "deterministic" | boolean;

	/**
	 * Tells Rspack to set process.env.NODE_ENV to a given string value.
	 * @default false
	 */
	nodeEnv?: string | false;

	/**
	 * Emit assets whenever there are errors while compiling.
	 *
	 * The value is `false` in production mode.
	 * The value is `true` in development mode.
	 * */
	emitOnErrors?: boolean;

	/**
	 * Avoid wrapping the entry module in an IIFE.
	 */
	avoidEntryIife?: boolean;
};
//#endregion

//#region Experiments
/**
 * Options for caching snapshots and intermediate products during the build process.
 * @description Controls whether caching is enabled or disabled.
 * @default true in development mode, false in production mode
 * @example
 * // Enable caching
 * cache: true
 *
 * // Disable caching
 * cache: false
 */
export type ExperimentCacheOptions =
	| boolean
	| {
			type: "memory";
	  }
	| {
			type: "persistent";
			buildDependencies?: string[];
			version?: string;
			snapshot?: {
				immutablePaths?: Array<string | RegExp>;
				unmanagedPaths?: Array<string | RegExp>;
				managedPaths?: Array<string | RegExp>;
			};
			storage?: {
				type: "filesystem";
				directory?: string;
			};
	  };

/**
 * Options for future Rspack features.
 */
export type RspackFutureOptions = {
	/**
	 * Information about the bundler.
	 */
	bundlerInfo?: {
		/**
		 * Version of the bundler.
		 */
		version?: string;
		/**
		 * Name of the bundler.
		 */
		bundler?: string;
		/**
		 * Force specific features.
		 */
		force?: boolean | ("version" | "uniqueId")[];
	};
};

/**
 * Options for lazy compilation.
 */
export type LazyCompilationOptions = {
	/**
	 * Backend configuration for lazy compilation.
	 */
	backend?: LazyCompilationDefaultBackendOptions;
	/**
	 * Enable lazy compilation for imports.
	 */
	imports?: boolean;
	/**
	 * Enable lazy compilation for entries.
	 */
	entries?: boolean;
	/**
	 * Test function or regex to determine which modules to include.
	 */
	test?: RegExp | ((module: Module) => boolean);
};

/**
 * Options for incremental builds.
 */
export type Incremental = {
	/**
	 * Enable incremental make.
	 */
	make?: boolean;

	/**
	 * Enable inference of async modules.
	 */
	inferAsyncModules?: boolean;

	/**
	 * Enable incremental provided exports.
	 */
	providedExports?: boolean;

	/**
	 * Enables diagnostics for dependencies.
	 */
	dependenciesDiagnostics?: boolean;

	/**
	 * Enables incremental side effects optimization.
	 */
	sideEffects?: boolean;

	/**
	 * Enable incremental build chunk graph.
	 */
	buildChunkGraph?: boolean;

	/**
	 * Enable incremental module ids.
	 */
	moduleIds?: boolean;

	/**
	 * Enable incremental chunk ids.
	 */
	chunkIds?: boolean;

	/**
	 * Enable incremental module hashes.
	 */
	modulesHashes?: boolean;

	/**
	 * Enable incremental module code generation.
	 */
	modulesCodegen?: boolean;

	/**
	 * Enable incremental module runtime requirements.
	 */
	modulesRuntimeRequirements?: boolean;

	/**
	 * Enable incremental chunk runtime requirements.
	 */
	chunksRuntimeRequirements?: boolean;

	/**
	 * Enable incremental chunk hashes.
	 */
	chunksHashes?: boolean;

	/**
	 * Enable incremental chunk render.
	 */
	chunksRender?: boolean;

	/**
	 * Enable incremental asset emission.
	 */
	emitAssets?: boolean;
};

/**
 * Experimental features configuration.
 */
export type Experiments = {
	/**
	 * Enable new cache.
	 */
	cache?: ExperimentCacheOptions;
	/**
	 * Enable lazy compilation.
	 * @default false
	 */
	lazyCompilation?: boolean | LazyCompilationOptions;
	/**
	 * Enable async WebAssembly.
	 * Support the new WebAssembly according to the [updated specification](https://github.com/WebAssembly/esm-integration), it makes a WebAssembly module an async module.
	 * @default false
	 */
	asyncWebAssembly?: boolean;
	/**
	 * Enable output as ES module.
	 * @default false
	 */
	outputModule?: boolean;
	/**
	 * Enable top-level await.
	 * @default true
	 */
	topLevelAwait?: boolean;
	/**
	 * Enable CSS support.
	 *
	 * @description
	 * Once enabled, Rspack will enable native CSS support, and CSS related parser and generator options.
	 * - `module.parser["css/auto"]`
	 * - `module.parser.css`
	 * - `module.parser["css/module"]`
	 * - `module.generator["css/auto"]`
	 * - `module.generator.css`
	 * - `module.generator["css/module"]`
	 */
	css?: boolean;
	/**
	 * Enable module layers feature.
	 * @default false
	 */
	layers?: boolean;
	/**
	 * Enable incremental builds.
	 */
	incremental?: boolean | Incremental;
	/**
	 * Enable multi-threaded code splitting algorithm.
	 */
	parallelCodeSplitting?: boolean;
	/**
	 * Enable future default options.
	 * @default false
	 */
	futureDefaults?: boolean;
	/**
	 * Enable future Rspack features default options.
	 */
	rspackFuture?: RspackFutureOptions;
};
//#endregion

//#region Watch
export type Watch = boolean;
//#endregion

//#region WatchOptions

/** Options for watch mode. */
export type WatchOptions = {
	/**
	 * Add a delay before rebuilding once the first file changed.
	 * This allows webpack to aggregate any other changes made during this time period into one rebuild.
	 * @default 5
	 */
	aggregateTimeout?: number;

	/**
	 * Follow symlinks while looking for files.
	 * This is usually not needed as webpack already resolves symlinks ('resolve.symlinks' and 'resolve.alias').
	 */
	followSymlinks?: boolean;

	/**
	 * Ignore some files from being watched.
	 */
	ignored?: string | RegExp | string[];

	/**
	 * Turn on polling by passing true, or specifying a poll interval in milliseconds.
	 * @default false
	 */
	poll?: number | boolean;

	/**
	 * Stop watching when stdin stream has ended.
	 */
	stdin?: boolean;
};
//#endregion

//#region DevServer
/**
 * Options for devServer, it based on `webpack-dev-server@5`
 * */
export interface DevServer extends webpackDevServer.Configuration {}
//#endregion

//#region IgnoreWarnings
/**
 * An array of either regular expressions or functions that determine if a warning should be ignored.
 */
export type IgnoreWarnings = (
	| RegExp
	| ((error: Error, compilation: Compilation) => boolean)
)[];
//#endregion

//#region Profile
/**
 * Capture a "profile" of the application, including statistics and hints, which can then be dissected using the Analyze tool.
 * */
export type Profile = boolean;
//#endregion

//#region amd
/**
 * Set the value of `require.amd` and `define.amd`. Or disable AMD support.
 */
export type Amd = false | Record<string, any>;
//#endregion

//#region Bail
/**
 * Fail out on the first error instead of tolerating it.
 * @default false
 * */
export type Bail = boolean;
//#endregion

//#region Performance
/** Options to control how Rspack notifies you of assets and entry points that exceed a specific file limit.   */
export type Performance =
	| false
	| {
			/**
			 * Filter function to select assets that are checked.
			 */
			assetFilter?: (assetFilename: string) => boolean;
			/**
			 * Sets the format of the hints: warnings, errors or nothing at all.
			 */
			hints?: false | "warning" | "error";
			/**
			 * File size limit (in bytes) when exceeded, that webpack will provide performance hints.
			 * @default 250000
			 */
			maxAssetSize?: number;
			/**
			 * Total size of an entry point (in bytes).
			 * @default 250000
			 */
			maxEntrypointSize?: number;
	  };
//#endregion

export type RspackOptions = {
	/**
	 * The name of the Rspack configuration.
	 */
	name?: Name;
	/**
	 * An array of dependencies required by the project.
	 */
	dependencies?: Dependencies;
	/**
	 * Configuration files to extend from. The configurations are merged from right to left,
	 * with the rightmost configuration taking precedence(only works when using @rspack/cli).
	 */
	extends?: string | string[];
	/**
	 * The entry point of the application.
	 */
	entry?: Entry;
	/**
	 * Configuration for the output of the compilation.
	 */
	output?: Output;
	/**
	 * The environment in which the code should run.
	 */
	target?: Target;
	/**
	 * The mode in which Rspack should operate.
	 */
	mode?: Mode;
	/**
	 * Options for experimental features.
	 */
	experiments?: Experiments;
	/**
	 * External libraries that should not be bundled.
	 */
	externals?: Externals;
	/**
	 * The type of externals.
	 */
	externalsType?: ExternalsType;
	/**
	 * Presets for external libraries.
	 */
	externalsPresets?: ExternalsPresets;
	/**
	 * Logging options for infrastructure.
	 */
	infrastructureLogging?: InfrastructureLogging;
	/**
	 * Options for caching.
	 */
	cache?: CacheOptions;
	/**
	 * The context in which the compilation should occur.
	 */
	context?: Context;
	/**
	 * The source map configuration.
	 */
	devtool?: DevTool;
	/**
	 * Options for Node.js environment.
	 */
	node?: Node;
	/**
	 * Configuration for loaders.
	 */
	loader?: Loader;
	/**
	 * Warnings to ignore during compilation.
	 */
	ignoreWarnings?: IgnoreWarnings;
	/**
	 * Options for watch mode.
	 */
	watchOptions?: WatchOptions;
	/**
	 * Whether to enable watch mode.
	 */
	watch?: Watch;
	/**
	 * Options for the stats output.
	 */
	stats?: StatsValue;
	/**
	 * Options for snapshotting.
	 */
	snapshot?: SnapshotOptions;
	/**
	 * Optimization options.
	 */
	optimization?: Optimization;
	/**
	 * Options for resolving modules.
	 */
	resolve?: ResolveOptions;
	/**
	 * Options for resolving loader modules.
	 */
	resolveLoader?: ResolveOptions;
	/**
	 * Plugins to use during compilation.
	 */
	plugins?: Plugins;
	/**
	 * Configuration for the development server.
	 */
	devServer?: DevServer;
	/**
	 * Options for module configuration.
	 */
	module?: ModuleOptions;
	/**
	 * Whether to capture a profile of the application.
	 */
	profile?: Profile;
	/**
	 * Set the value of `require.amd` or `define.amd`.
	 * Setting `amd` to false will disable rspack's AMD support.
	 */
	amd?: Amd;
	/**
	 * Whether to fail on the first error.
	 */
	bail?: Bail;
	/**
	 * Performance optimization options.
	 */
	performance?: Performance;
};

/** Configuration for Rspack */
export type Configuration = RspackOptions;
