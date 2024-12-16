/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/config/normalization.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import path from "node:path";
import util from "node:util";
import type { Compilation } from "../Compilation";
import type {
	AssetModuleFilename,
	Bail,
	CacheOptions,
	ChunkFilename,
	ChunkLoading,
	ChunkLoadingGlobal,
	Clean,
	Context,
	CrossOriginLoading,
	CssChunkFilename,
	CssFilename,
	Dependencies,
	DevServer,
	DevTool,
	DevtoolFallbackModuleFilenameTemplate,
	DevtoolModuleFilenameTemplate,
	DevtoolNamespace,
	EnabledLibraryTypes,
	EnabledWasmLoadingTypes,
	EntryDescription,
	EntryStatic,
	Environment,
	Externals,
	ExternalsPresets,
	ExternalsType,
	Filename,
	GeneratorOptionsByModuleType,
	GlobalObject,
	HashDigest,
	HashDigestLength,
	HashFunction,
	HashSalt,
	HotUpdateChunkFilename,
	HotUpdateGlobal,
	HotUpdateMainFilename,
	Iife,
	ImportFunctionName,
	ImportMetaName,
	Incremental,
	InfrastructureLogging,
	LazyCompilationOptions,
	LibraryOptions,
	Loader,
	Mode,
	Name,
	NoParseOption,
	Node,
	Optimization,
	OptimizationRuntimeChunk,
	OutputModule,
	ParserOptionsByModuleType,
	Path,
	Performance,
	Plugins,
	Profile,
	PublicPath,
	Resolve,
	RspackFutureOptions,
	RspackOptions,
	RuleSetRules,
	ScriptType,
	SnapshotOptions,
	SourceMapFilename,
	StatsValue,
	StrictModuleErrorHandling,
	Target,
	TrustedTypes,
	UniqueName,
	WasmLoading,
	Watch,
	WatchOptions,
	WebassemblyModuleFilename,
	WorkerPublicPath
} from "./types";

export const getNormalizedRspackOptions = (
	config: RspackOptions
): RspackOptionsNormalized => {
	return {
		ignoreWarnings:
			config.ignoreWarnings !== undefined
				? config.ignoreWarnings.map(ignore => {
						if (typeof ignore === "function") {
							return ignore;
						}
						return (warning: Error) => {
							return ignore.test(warning.message);
						};
					})
				: undefined,
		name: config.name,
		dependencies: config.dependencies,
		context: config.context,
		mode: config.mode,
		entry:
			config.entry === undefined
				? { main: {} }
				: typeof config.entry === "function"
					? (
							fn => () =>
								Promise.resolve().then(fn).then(getNormalizedEntryStatic)
						)(config.entry)
					: getNormalizedEntryStatic(config.entry),
		output: nestedConfig(config.output, output => {
			if ("cssHeadDataCompression" in output) {
				util.deprecate(
					() => {},
					"cssHeadDataCompression is not used now, see https://github.com/web-infra-dev/rspack/pull/8534, this option could be removed in the future"
				)();
			}

			const { library } = output;
			const libraryAsName = library;
			const libraryBase =
				typeof library === "object" &&
				library &&
				!Array.isArray(library) &&
				"type" in library
					? library
					: libraryAsName || output.libraryTarget
						? ({
								name: libraryAsName
							} as LibraryOptions)
						: undefined;
			return {
				path: output.path,
				pathinfo: output.pathinfo,
				publicPath: output.publicPath,
				filename: output.filename,
				clean: output.clean,
				chunkFormat: output.chunkFormat,
				chunkLoading: output.chunkLoading,
				chunkFilename: output.chunkFilename,
				crossOriginLoading: output.crossOriginLoading,
				cssFilename: output.cssFilename,
				cssChunkFilename: output.cssChunkFilename,
				hotUpdateMainFilename: output.hotUpdateMainFilename,
				hotUpdateChunkFilename: output.hotUpdateChunkFilename,
				hotUpdateGlobal: output.hotUpdateGlobal,
				assetModuleFilename: output.assetModuleFilename,
				wasmLoading: output.wasmLoading,
				enabledChunkLoadingTypes: output.enabledChunkLoadingTypes
					? [...output.enabledChunkLoadingTypes]
					: ["..."],
				enabledWasmLoadingTypes: output.enabledWasmLoadingTypes
					? [...output.enabledWasmLoadingTypes]
					: ["..."],
				webassemblyModuleFilename: output.webassemblyModuleFilename,
				uniqueName: output.uniqueName,
				chunkLoadingGlobal: output.chunkLoadingGlobal,
				enabledLibraryTypes: output.enabledLibraryTypes
					? [...output.enabledLibraryTypes]
					: ["..."],
				globalObject: output.globalObject,
				importFunctionName: output.importFunctionName,
				importMetaName: output.importMetaName,
				iife: output.iife,
				module: output.module,
				sourceMapFilename: output.sourceMapFilename,
				library: libraryBase && {
					type:
						output.libraryTarget !== undefined
							? output.libraryTarget
							: libraryBase.type,
					auxiliaryComment:
						output.auxiliaryComment !== undefined
							? output.auxiliaryComment
							: libraryBase.auxiliaryComment,
					amdContainer: libraryBase.amdContainer,
					export:
						output.libraryExport !== undefined
							? output.libraryExport
							: libraryBase.export,
					name: libraryBase.name,
					umdNamedDefine:
						output.umdNamedDefine !== undefined
							? output.umdNamedDefine
							: libraryBase.umdNamedDefine
				},
				strictModuleErrorHandling:
					output.strictModuleErrorHandling ??
					output.strictModuleExceptionHandling,
				trustedTypes: optionalNestedConfig(
					output.trustedTypes,
					trustedTypes => {
						if (trustedTypes === true) return {};
						if (typeof trustedTypes === "string")
							return { policyName: trustedTypes };
						return { ...trustedTypes };
					}
				),
				hashDigest: output.hashDigest,
				hashDigestLength: output.hashDigestLength,
				hashFunction: output.hashFunction,
				hashSalt: output.hashSalt,
				asyncChunks: output.asyncChunks,
				workerChunkLoading: output.workerChunkLoading,
				workerWasmLoading: output.workerWasmLoading,
				workerPublicPath: output.workerPublicPath,
				scriptType: output.scriptType,
				devtoolNamespace: output.devtoolNamespace,
				devtoolModuleFilenameTemplate: output.devtoolModuleFilenameTemplate,
				devtoolFallbackModuleFilenameTemplate:
					output.devtoolFallbackModuleFilenameTemplate,
				chunkLoadTimeout: output.chunkLoadTimeout,
				charset: output.charset,
				environment: cloneObject(output.environment),
				compareBeforeEmit: output.compareBeforeEmit
			};
		}),
		resolve: nestedConfig(config.resolve, resolve => ({
			...resolve,
			tsConfig: optionalNestedConfig(resolve.tsConfig, tsConfig => {
				return typeof tsConfig === "string"
					? { configFile: tsConfig }
					: tsConfig;
			})
		})),
		resolveLoader: nestedConfig(config.resolveLoader, resolve => ({
			...resolve,
			tsConfig: optionalNestedConfig(resolve.tsConfig, tsConfig => {
				return typeof tsConfig === "string"
					? { configFile: tsConfig }
					: tsConfig;
			})
		})),
		module: nestedConfig(config.module, module => ({
			noParse: module.noParse,
			parser: keyedNestedConfig(
				module.parser as Record<string, any>,
				cloneObject,
				{}
			),
			generator: keyedNestedConfig(
				module.generator as Record<string, any>,
				cloneObject,
				{}
			),
			defaultRules: optionalNestedArray(module.defaultRules, r => [...r]),
			rules: nestedArray(module.rules, r => [...r])
		})),
		target: config.target,
		externals: config.externals,
		externalsType: config.externalsType,
		externalsPresets: cloneObject(config.externalsPresets),
		infrastructureLogging: cloneObject(config.infrastructureLogging),
		devtool: config.devtool,
		node: nestedConfig(
			config.node,
			node =>
				node && {
					...node
				}
		),
		loader: cloneObject(config.loader),
		snapshot: nestedConfig(config.snapshot, _snapshot => ({})),
		cache: optionalNestedConfig(config.cache, cache => cache),
		stats: nestedConfig(config.stats, stats => {
			if (stats === false) {
				return {
					preset: "none"
				};
			}
			if (stats === true) {
				return {
					preset: "normal"
				};
			}
			if (typeof stats === "string") {
				return {
					preset: stats
				};
			}
			return {
				...stats
			};
		}),
		optimization: nestedConfig(config.optimization, optimization => {
			return {
				...optimization,
				runtimeChunk: getNormalizedOptimizationRuntimeChunk(
					optimization.runtimeChunk
				),
				splitChunks: nestedConfig(
					optimization.splitChunks,
					splitChunks =>
						splitChunks && {
							...splitChunks,
							defaultSizeTypes: splitChunks.defaultSizeTypes
								? [...splitChunks.defaultSizeTypes]
								: ["..."],
							cacheGroups: cloneObject(splitChunks.cacheGroups)
						}
				)
			};
		}),
		performance: config.performance,
		plugins: nestedArray(config.plugins, p => [...p]),
		experiments: nestedConfig(config.experiments, experiments => ({
			...experiments,
			cache: optionalNestedConfig(experiments.cache, cache => {
				if (typeof cache === "boolean") {
					return cache;
				}
				if (cache.type === "memory") {
					return cache;
				}
				const snapshot = cache.snapshot || {};
				return {
					type: "persistent",
					buildDependencies: nestedArray(cache.buildDependencies, deps =>
						deps.map(d => path.resolve(config.context || process.cwd(), d))
					),
					version: cache.version || "",
					snapshot: {
						immutablePaths: nestedArray(snapshot.immutablePaths, p => [...p]),
						unmanagedPaths: nestedArray(snapshot.unmanagedPaths, p => [...p]),
						managedPaths: optionalNestedArray(snapshot.managedPaths, p => [
							...p
						]) || [/\/node_modules\//]
					},
					storage: {
						type: "filesystem",
						directory: path.resolve(
							config.context || process.cwd(),
							cache.storage?.directory || "node_modules/.cache/rspack"
						)
					}
				};
			}),
			lazyCompilation: optionalNestedConfig(
				experiments.lazyCompilation,
				options => (options === true ? {} : options)
			),
			incremental: optionalNestedConfig(experiments.incremental, options =>
				options === true
					? ({
							make: true,
							dependenciesDiagnostics: true,
							inferAsyncModules: true,
							providedExports: true,
							buildChunkGraph: true,
							moduleIds: true,
							chunkIds: true,
							modulesHashes: true,
							modulesCodegen: true,
							modulesRuntimeRequirements: true,
							chunksRuntimeRequirements: true,
							chunksHashes: true,
							chunksRender: true,
							emitAssets: true
						} satisfies Incremental)
					: options
			)
		})),
		watch: config.watch,
		watchOptions: cloneObject(config.watchOptions),
		devServer: config.devServer,
		profile: config.profile,
		amd: config.amd ? JSON.stringify(config.amd) : undefined,
		bail: config.bail
	};
};

const getNormalizedEntryStatic = (entry: EntryStatic) => {
	if (typeof entry === "string") {
		return {
			main: {
				import: [entry]
			}
		};
	}
	if (Array.isArray(entry)) {
		return {
			main: {
				import: entry
			}
		};
	}
	const result: EntryStaticNormalized = {};
	for (const key of Object.keys(entry)) {
		const value = entry[key];
		if (typeof value === "string") {
			result[key] = {
				import: [value]
			};
		} else if (Array.isArray(value)) {
			result[key] = {
				import: value
			};
		} else {
			result[key] = {
				import: Array.isArray(value.import) ? value.import : [value.import],
				runtime: value.runtime,
				publicPath: value.publicPath,
				baseUri: value.baseUri,
				chunkLoading: value.chunkLoading,
				asyncChunks: value.asyncChunks,
				filename: value.filename,
				library: value.library,
				layer: value.layer,
				dependOn: Array.isArray(value.dependOn)
					? value.dependOn
					: value.dependOn
						? [value.dependOn]
						: undefined
			};
		}
	}
	return result;
};

const getNormalizedOptimizationRuntimeChunk = (
	runtimeChunk?: OptimizationRuntimeChunk
): OptimizationRuntimeChunkNormalized | undefined => {
	if (runtimeChunk === undefined) return undefined;
	if (runtimeChunk === false) return false;
	if (runtimeChunk === "single") {
		return {
			name: "single"
		};
	}
	if (runtimeChunk === true || runtimeChunk === "multiple") {
		return {
			name: "multiple"
		};
	}
	if (runtimeChunk.name) {
		const opts: OptimizationRuntimeChunkNormalized = {
			name: runtimeChunk.name
		};
		return opts;
	}
};

const nestedConfig = <T, R>(value: T | undefined, fn: (value: T) => R) =>
	value === undefined ? fn({} as T) : fn(value);

const optionalNestedConfig = <T, R>(
	value: T | undefined,
	fn: (value: T) => R
) => (value === undefined ? undefined : fn(value));

const nestedArray = <T, R>(value: T[] | undefined, fn: (value: T[]) => R[]) =>
	Array.isArray(value) ? fn(value) : fn([]);

const optionalNestedArray = <T, R>(
	value: T[] | undefined,
	fn: (value: T[]) => R[]
) => (Array.isArray(value) ? fn(value) : undefined);

const cloneObject = <T>(value?: T) => ({ ...value });

const keyedNestedConfig = <T, R>(
	value: Record<string, T> | undefined,
	fn: (value: T) => R,
	customKeys: Record<string, (value: T) => R>
) => {
	const result =
		value === undefined
			? {}
			: Object.keys(value).reduce(
					(obj, key) => {
						obj[key] = (customKeys && key in customKeys ? customKeys[key] : fn)(
							value[key]
						);
						return obj;
					},
					{} as Record<string, R>
				);
	if (customKeys) {
		for (const key of Object.keys(customKeys)) {
			if (!(key in result)) {
				result[key] = customKeys[key]({} as T);
			}
		}
	}
	return result;
};

export type EntryDynamicNormalized = () => Promise<EntryStaticNormalized>;

export type EntryNormalized = EntryDynamicNormalized | EntryStaticNormalized;

export interface EntryStaticNormalized {
	[k: string]: EntryDescriptionNormalized;
}

export type EntryDescriptionNormalized = Pick<
	EntryDescription,
	| "runtime"
	| "chunkLoading"
	| "asyncChunks"
	| "publicPath"
	| "baseUri"
	| "filename"
	| "library"
	| "layer"
> & {
	import?: string[];
	dependOn?: string[];
};

export interface OutputNormalized {
	path?: Path;
	pathinfo?: boolean | "verbose";
	clean?: Clean;
	publicPath?: PublicPath;
	filename?: Filename;
	chunkFilename?: ChunkFilename;
	crossOriginLoading?: CrossOriginLoading;
	cssFilename?: CssFilename;
	cssChunkFilename?: CssChunkFilename;
	hotUpdateMainFilename?: HotUpdateMainFilename;
	hotUpdateChunkFilename?: HotUpdateChunkFilename;
	hotUpdateGlobal?: HotUpdateGlobal;
	assetModuleFilename?: AssetModuleFilename;
	uniqueName?: UniqueName;
	chunkLoadingGlobal?: ChunkLoadingGlobal;
	enabledLibraryTypes?: EnabledLibraryTypes;
	library?: LibraryOptions;
	module?: OutputModule;
	strictModuleErrorHandling?: StrictModuleErrorHandling;
	globalObject?: GlobalObject;
	importFunctionName?: ImportFunctionName;
	importMetaName?: ImportMetaName;
	iife?: Iife;
	wasmLoading?: WasmLoading;
	enabledWasmLoadingTypes?: EnabledWasmLoadingTypes;
	webassemblyModuleFilename?: WebassemblyModuleFilename;
	chunkFormat?: string | false;
	chunkLoading?: string | false;
	enabledChunkLoadingTypes?: string[];
	trustedTypes?: TrustedTypes;
	sourceMapFilename?: SourceMapFilename;
	hashDigest?: HashDigest;
	hashDigestLength?: HashDigestLength;
	hashFunction?: HashFunction;
	hashSalt?: HashSalt;
	asyncChunks?: boolean;
	workerChunkLoading?: ChunkLoading;
	workerWasmLoading?: WasmLoading;
	workerPublicPath?: WorkerPublicPath;
	scriptType?: ScriptType;
	devtoolNamespace?: DevtoolNamespace;
	devtoolModuleFilenameTemplate?: DevtoolModuleFilenameTemplate;
	devtoolFallbackModuleFilenameTemplate?: DevtoolFallbackModuleFilenameTemplate;
	environment?: Environment;
	charset?: boolean;
	chunkLoadTimeout?: number;
	compareBeforeEmit?: boolean;
}

export interface ModuleOptionsNormalized {
	defaultRules?: RuleSetRules;
	rules: RuleSetRules;
	parser: ParserOptionsByModuleType;
	generator: GeneratorOptionsByModuleType;
	noParse?: NoParseOption;
}

export type ExperimentCacheNormalized =
	| boolean
	| {
			type: "memory";
	  }
	| {
			type: "persistent";
			buildDependencies: string[];
			version: string;
			snapshot: {
				immutablePaths: Array<string | RegExp>;
				unmanagedPaths: Array<string | RegExp>;
				managedPaths: Array<string | RegExp>;
			};
			storage: {
				type: "filesystem";
				directory: string;
			};
	  };

export interface ExperimentsNormalized {
	cache?: ExperimentCacheNormalized;
	lazyCompilation?: false | LazyCompilationOptions;
	asyncWebAssembly?: boolean;
	outputModule?: boolean;
	topLevelAwait?: boolean;
	css?: boolean;
	layers?: boolean;
	incremental?: false | Incremental;
	futureDefaults?: boolean;
	rspackFuture?: RspackFutureOptions;
}

export type IgnoreWarningsNormalized = ((
	warning: Error,
	compilation: Compilation
) => boolean)[];

export type OptimizationRuntimeChunkNormalized =
	| false
	| {
			name: string | ((entrypoint: { name: string }) => string);
	  };

export interface RspackOptionsNormalized {
	name?: Name;
	dependencies?: Dependencies;
	context?: Context;
	mode?: Mode;
	entry: EntryNormalized;
	output: OutputNormalized;
	resolve: Resolve;
	resolveLoader: Resolve;
	module: ModuleOptionsNormalized;
	target?: Target;
	externals?: Externals;
	externalsType?: ExternalsType;
	externalsPresets: ExternalsPresets;
	infrastructureLogging: InfrastructureLogging;
	devtool?: DevTool;
	node: Node;
	loader: Loader;
	snapshot: SnapshotOptions;
	cache?: CacheOptions;
	stats: StatsValue;
	optimization: Optimization;
	plugins: Plugins;
	experiments: ExperimentsNormalized;
	watch?: Watch;
	watchOptions: WatchOptions;
	devServer?: DevServer;
	ignoreWarnings?: IgnoreWarningsNormalized;
	performance?: Performance;
	profile?: Profile;
	amd?: string;
	bail?: Bail;
}
