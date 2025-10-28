/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compilation.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type {
	AssetInfo,
	ChunkGroup,
	Dependency,
	ExternalObject,
	JsSource,
	JsCompilation,
	JsPathData,
	JsRuntimeModule
} from "@rspack/binding";
import binding from "@rspack/binding";

export type { AssetInfo } from "@rspack/binding";

import * as liteTapable from "@rspack/lite-tapable";
import type { Source } from "webpack-sources";
import type { EntryOptions, EntryPlugin } from "./builtin-plugin";
import type { Chunk } from "./Chunk";
import type { ChunkGraph } from "./ChunkGraph";
import type { Compiler } from "./Compiler";
import type { ContextModuleFactory } from "./ContextModuleFactory";
import type {
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance,
	StatsOptions,
	StatsValue
} from "./config";
import type { Entrypoint } from "./Entrypoint";
import { cutOffLoaderExecution } from "./ErrorHelpers";
import WebpackError from "./lib/WebpackError";
import { Logger, LogType } from "./logging/Logger";
import type { Module } from "./Module";
import ModuleGraph from "./ModuleGraph";
import type { NormalModuleCompilationHooks } from "./NormalModule";
import type { NormalModuleFactory } from "./NormalModuleFactory";
import type { ResolverFactory } from "./ResolverFactory";
import type { RspackError } from "./RspackError";
import { RuntimeModule } from "./RuntimeModule";
import {
	Stats,
	type StatsAsset,
	type StatsError,
	type StatsModule
} from "./Stats";
import { StatsFactory } from "./stats/StatsFactory";
import { StatsPrinter } from "./stats/StatsPrinter";
import { AsyncTask } from "./util/AsyncTask";
import { createReadonlyMap } from "./util/createReadonlyMap";
import { createFakeCompilationDependencies } from "./util/fake";
import type { InputFileSystem } from "./util/fs";
import type Hash from "./util/hash";
import { SourceAdapter } from "./util/source";
// patch Chunk
import "./Chunk";
// patch Chunks
import "./Chunks";
// patch ChunkGraph
import "./ChunkGraph";
// patch CodeGenerationResults
import "./CodeGenerationResults";
import { createDiagnosticArray } from "./Diagnostics";
import type { CodeGenerationResult } from "./taps/compilation";

export type Assets = Record<string, Source>;
export interface Asset {
	name: string;
	source: Source;
	info: AssetInfo;
}

export type ChunkPathData = {
	id?: string;
	name?: string;
	hash?: string;
	contentHash?: Record<string, string>;
};

export type PathData = {
	filename?: string;
	hash?: string;
	contentHash?: string;
	runtime?: string;
	url?: string;
	id?: string;
	chunk?: Chunk | ChunkPathData;
	contentHashType?: string;
};

export interface LogEntry {
	type: string;
	args: any[];
	time?: number;
	trace?: string[];
}

export interface CompilationParams {
	normalModuleFactory: NormalModuleFactory;
	contextModuleFactory: ContextModuleFactory;
}

export interface KnownCreateStatsOptionsContext {
	forToString?: boolean;
}

export interface ExecuteModuleArgument {
	codeGenerationResult: CodeGenerationResult;
	moduleObject: {
		id: string;
		exports: any;
		loaded: boolean;
		error?: Error;
	};
}

export interface ExecuteModuleContext {
	__webpack_require__: (id: string) => any;
}

export interface KnownNormalizedStatsOptions {
	context: string;
	// requestShortener: RequestShortener;
	chunksSort: string;
	modulesSort: string;
	chunkModulesSort: string;
	nestedModulesSort: string;
	assetsSort: string;
	ids: boolean;
	cachedAssets: boolean;
	groupAssetsByEmitStatus: boolean;
	groupAssetsByPath: boolean;
	groupAssetsByExtension: boolean;
	assetsSpace: number;
	excludeAssets: ((value: string, asset: StatsAsset) => boolean)[];
	excludeModules: ((
		name: string,
		module: StatsModule,
		type: "module" | "chunk" | "root-of-chunk" | "nested"
	) => boolean)[];
	warningsFilter: ((warning: StatsError, textValue: string) => boolean)[];
	cachedModules: boolean;
	orphanModules: boolean;
	dependentModules: boolean;
	runtimeModules: boolean;
	groupModulesByCacheStatus: boolean;
	groupModulesByLayer: boolean;
	groupModulesByAttributes: boolean;
	groupModulesByPath: boolean;
	groupModulesByExtension: boolean;
	groupModulesByType: boolean;
	entrypoints: boolean | "auto";
	chunkGroups: boolean;
	chunkGroupAuxiliary: boolean;
	chunkGroupChildren: boolean;
	chunkGroupMaxAssets: number;
	modulesSpace: number;
	chunkModulesSpace: number;
	nestedModulesSpace: number;
	logging: false | "none" | "error" | "warn" | "info" | "log" | "verbose";
	loggingDebug: ((value: string) => boolean)[];
	loggingTrace: boolean;
	chunkModules: boolean;
	chunkRelations: boolean;
	reasons: boolean;
	moduleAssets: boolean;
	nestedModules: boolean;
	source: boolean;
	usedExports: boolean;
	providedExports: boolean;
	optimizationBailout: boolean;
	depth: boolean;
	assets: boolean;
	chunks: boolean;
	errors: boolean;
	errorsCount: boolean;
	hash: boolean;
	modules: boolean;
	warnings: boolean;
	warningsCount: boolean;
}

export type CreateStatsOptionsContext = KnownCreateStatsOptionsContext &
	Record<string, any>;

export type NormalizedStatsOptions = KnownNormalizedStatsOptions &
	Omit<StatsOptions, keyof KnownNormalizedStatsOptions> &
	Record<string, any>;

export const checkCompilation = (compilation: Compilation) => {
	if (!(compilation instanceof Compilation)) {
		throw new TypeError(
			`The 'compilation' argument must be an instance of Compilation. This usually occurs when multiple versions of "@rspack/core" are used, or when the code in "@rspack/core" is executed multiple times.`
		);
	}
};

export class Compilation {
	#inner: JsCompilation;
	#shutdown: boolean;
	#errors?: RspackError[];
	#warnings?: RspackError[];
	#chunks?: ReadonlySet<Chunk>;

	hooks: Readonly<{
		processAssets: liteTapable.AsyncSeriesHook<Assets>;
		afterProcessAssets: liteTapable.SyncHook<Assets>;
		childCompiler: liteTapable.SyncHook<[Compiler, string, number]>;
		log: liteTapable.SyncBailHook<[string, LogEntry], true>;
		additionalAssets: any;
		optimizeModules: liteTapable.SyncBailHook<Iterable<Module>, void>;
		afterOptimizeModules: liteTapable.SyncHook<Iterable<Module>, void>;
		optimizeTree: liteTapable.AsyncSeriesHook<
			[Iterable<Chunk>, Iterable<Module>]
		>;
		optimizeChunkModules: liteTapable.AsyncSeriesBailHook<
			[Iterable<Chunk>, Iterable<Module>],
			void
		>;
		finishModules: liteTapable.AsyncSeriesHook<[Iterable<Module>], void>;
		chunkHash: liteTapable.SyncHook<[Chunk, Hash], void>;
		chunkAsset: liteTapable.SyncHook<[Chunk, string], void>;
		processWarnings: liteTapable.SyncWaterfallHook<[WebpackError[]]>;
		succeedModule: liteTapable.SyncHook<[Module], void>;
		stillValidModule: liteTapable.SyncHook<[Module], void>;

		statsPreset: liteTapable.HookMap<
			liteTapable.SyncHook<
				[Partial<StatsOptions>, CreateStatsOptionsContext],
				void
			>
		>;
		statsNormalize: liteTapable.SyncHook<
			[Partial<StatsOptions>, CreateStatsOptionsContext],
			void
		>;
		statsFactory: liteTapable.SyncHook<[StatsFactory, StatsOptions], void>;
		statsPrinter: liteTapable.SyncHook<[StatsPrinter, StatsOptions], void>;

		buildModule: liteTapable.SyncHook<[Module]>;
		executeModule: liteTapable.SyncHook<
			[ExecuteModuleArgument, ExecuteModuleContext]
		>;
		additionalTreeRuntimeRequirements: liteTapable.SyncHook<
			[Chunk, Set<string>],
			void
		>;
		runtimeRequirementInTree: liteTapable.HookMap<
			liteTapable.SyncBailHook<[Chunk, Set<string>], void>
		>;
		runtimeModule: liteTapable.SyncHook<[JsRuntimeModule, Chunk], void>;
		seal: liteTapable.SyncHook<[], void>;
		afterSeal: liteTapable.AsyncSeriesHook<[], void>;
		needAdditionalPass: liteTapable.SyncBailHook<[], boolean>;
	}>;
	name?: string;
	startTime?: number;
	endTime?: number;
	compiler: Compiler;
	resolverFactory: ResolverFactory;

	inputFileSystem: InputFileSystem | null;
	options: RspackOptionsNormalized;
	outputOptions: OutputNormalized;
	logging: Map<string, LogEntry[]>;
	childrenCounters: Record<string, number>;
	children: Compilation[];
	chunkGraph: ChunkGraph;
	moduleGraph: ModuleGraph;
	fileSystemInfo = {
		createSnapshot() {
			// fake implement to support html-webpack-plugin
			return null;
		}
	};
	needAdditionalPass: boolean;

	#addIncludeDispatcher: AddEntryItemDispatcher;
	#addEntryDispatcher: AddEntryItemDispatcher;

	[binding.COMPILATION_HOOKS_MAP_SYMBOL]: WeakMap<
		Compilation,
		NormalModuleCompilationHooks
	>;

	constructor(compiler: Compiler, inner: JsCompilation) {
		this.#inner = inner;
		this.#shutdown = false;

		const processAssetsHook = new liteTapable.AsyncSeriesHook<Assets>([
			"assets"
		]);
		const createProcessAssetsHook = <T>(
			name: string,
			stage: number,
			getArgs: () => liteTapable.AsArray<T>,
			code?: string
		) => {
			const errorMessage = (
				reason: string
			) => `Can't automatically convert plugin using Compilation.hooks.${name} to Compilation.hooks.processAssets because ${reason}.
BREAKING CHANGE: Asset processing hooks in Compilation has been merged into a single Compilation.hooks.processAssets hook.`;
			const getOptions = (options: liteTapable.Options) => {
				const isString = typeof options === "string";
				if (!isString && options.stage) {
					throw new Error(errorMessage("it's using the 'stage' option"));
				}
				return {
					...(isString ? { name: options } : options),
					stage: stage
				};
			};
			return Object.freeze({
				name,
				intercept() {
					throw new Error(errorMessage("it's using 'intercept'"));
				},
				tap: (options: liteTapable.Options, fn: liteTapable.Fn<T, void>) => {
					processAssetsHook.tap(getOptions(options), () => fn(...getArgs()));
				},
				tapAsync: (
					options: liteTapable.Options,
					fn: liteTapable.FnAsync<T, void>
				) => {
					processAssetsHook.tapAsync(getOptions(options), (assets, callback) =>
						(fn as any)(...getArgs(), callback)
					);
				},
				tapPromise: (
					options: liteTapable.Options,
					fn: liteTapable.FnPromise<T, void>
				) => {
					processAssetsHook.tapPromise(getOptions(options), () =>
						fn(...getArgs())
					);
				},
				_fakeHook: true
			});
		};
		this.hooks = {
			processAssets: processAssetsHook,
			afterProcessAssets: new liteTapable.SyncHook(["assets"]),
			/** @deprecated */
			additionalAssets: createProcessAssetsHook(
				"additionalAssets",
				Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
				() => []
			),
			childCompiler: new liteTapable.SyncHook([
				"childCompiler",
				"compilerName",
				"compilerIndex"
			]),
			log: new liteTapable.SyncBailHook(["origin", "logEntry"]),
			optimizeModules: new liteTapable.SyncBailHook(["modules"]),
			afterOptimizeModules: new liteTapable.SyncBailHook(["modules"]),
			optimizeTree: new liteTapable.AsyncSeriesHook(["chunks", "modules"]),
			optimizeChunkModules: new liteTapable.AsyncSeriesBailHook([
				"chunks",
				"modules"
			]),
			finishModules: new liteTapable.AsyncSeriesHook(["modules"]),
			chunkHash: new liteTapable.SyncHook(["chunk", "hash"]),
			chunkAsset: new liteTapable.SyncHook(["chunk", "filename"]),
			processWarnings: new liteTapable.SyncWaterfallHook(["warnings"]),
			succeedModule: new liteTapable.SyncHook(["module"]),
			stillValidModule: new liteTapable.SyncHook(["module"]),

			statsPreset: new liteTapable.HookMap(
				() => new liteTapable.SyncHook(["options", "context"])
			),
			statsNormalize: new liteTapable.SyncHook(["options", "context"]),
			statsFactory: new liteTapable.SyncHook(["statsFactory", "options"]),
			statsPrinter: new liteTapable.SyncHook(["statsPrinter", "options"]),

			buildModule: new liteTapable.SyncHook(["module"]),
			executeModule: new liteTapable.SyncHook(["options", "context"]),
			additionalTreeRuntimeRequirements: new liteTapable.SyncHook([
				"chunk",
				"runtimeRequirements"
			]),
			runtimeRequirementInTree: new liteTapable.HookMap(
				() => new liteTapable.SyncBailHook(["chunk", "runtimeRequirements"])
			),
			runtimeModule: new liteTapable.SyncHook(["module", "chunk"]),
			seal: new liteTapable.SyncHook([]),
			afterSeal: new liteTapable.AsyncSeriesHook([]),
			needAdditionalPass: new liteTapable.SyncBailHook([])
		};
		this.compiler = compiler;
		this.resolverFactory = compiler.resolverFactory;
		this.inputFileSystem = compiler.inputFileSystem;
		this.options = compiler.options;
		this.outputOptions = compiler.options.output;
		this.logging = new Map();
		this.childrenCounters = {};
		this.children = [];
		this.needAdditionalPass = false;

		this.chunkGraph = inner.chunkGraph;
		this.moduleGraph = ModuleGraph.__from_binding(inner.moduleGraph);

		this.#addIncludeDispatcher = new AddEntryItemDispatcher(
			inner.addInclude.bind(inner)
		);
		this.#addEntryDispatcher = new AddEntryItemDispatcher(
			inner.addEntry.bind(inner)
		);
		this[binding.COMPILATION_HOOKS_MAP_SYMBOL] = new WeakMap();
	}

	get hash(): Readonly<string | null> {
		return this.#inner.hash;
	}

	get fullHash(): Readonly<string | null> {
		return this.#inner.hash;
	}

	/**
	 * Get a map of all assets.
	 */
	get assets(): Record<string, Source> {
		return this.#createCachedAssets();
	}

	/**
	 * Get a map of all entrypoints.
	 */
	get entrypoints(): ReadonlyMap<string, Entrypoint> {
		return new Map(
			this.#inner.entrypoints.map(entrypoint => [entrypoint.name!, entrypoint])
		);
	}

	get chunkGroups(): ReadonlyArray<ChunkGroup> {
		return this.#inner.chunkGroups;
	}

	/**
	 * Get the named chunk groups.
	 *
	 * Note: This is a proxy for webpack internal API, only method `get`, `keys`, `values` and `entries` are supported now.
	 */
	get namedChunkGroups() {
		return createReadonlyMap<ChunkGroup>({
			keys: (): ReturnType<string[]["values"]> => {
				const names = this.#inner.getNamedChunkGroupKeys();
				return names[Symbol.iterator]();
			},
			get: (property: unknown) => {
				if (typeof property === "string") {
					return this.#inner.getNamedChunkGroup(property);
				}
			}
		});
	}

	get modules(): ReadonlySet<Module> {
		return new Set(this.#inner.modules);
	}

	get builtModules(): ReadonlySet<Module> {
		return new Set(this.#inner.builtModules);
	}

	get chunks(): ReadonlySet<Chunk> {
		if (!this.#chunks) {
			this.#chunks = this.#inner.chunks;
		}
		return this.#chunks;
	}

	/**
	 * Get the named chunks.
	 *
	 * Note: This is a proxy for webpack internal API, only method `get`, `keys`, `values` and `entries` are supported now.
	 */
	get namedChunks() {
		return createReadonlyMap<Chunk>({
			keys: (): ReturnType<string[]["values"]> => {
				const names = this.#inner.getNamedChunkKeys();
				return names[Symbol.iterator]();
			},
			get: (property: unknown) => {
				if (typeof property === "string") {
					return this.#inner.getNamedChunk(property);
				}
			}
		});
	}

	get entries(): Map<string, EntryData> {
		return new Entries(this.#inner.entries);
	}

	get codeGenerationResults(): binding.CodeGenerationResults {
		return this.#inner.codeGenerationResults;
	}

	#createCachedAssets() {
		return new Proxy(
			{},
			{
				get: (_, property) => {
					if (typeof property === "string") {
						return this.__internal__getAssetSource(property);
					}
				},
				set: (_, p, newValue) => {
					if (typeof p === "string") {
						this.__internal__setAssetSource(p, newValue);
						return true;
					}
					return false;
				},
				deleteProperty: (_, p) => {
					if (typeof p === "string") {
						this.__internal__deleteAssetSource(p);
						return true;
					}
					return false;
				},
				has: (_, property) => {
					if (typeof property === "string") {
						return this.__internal__hasAsset(property);
					}
					return false;
				},
				ownKeys: _ => {
					return this.__internal__getAssetFilenames();
				},
				getOwnPropertyDescriptor() {
					// To work with `Object.keys`, you should mark the property as enumerable.
					// See: https://262.ecma-international.org/7.0/#sec-enumerableownnames
					return {
						enumerable: true,
						configurable: true
					};
				}
			}
		);
	}

	getCache(name: string) {
		return this.compiler.getCache(name);
	}

	createStatsOptions(
		statsValue: StatsValue | undefined,
		context: CreateStatsOptionsContext = {}
	): NormalizedStatsOptions {
		let optionsOrPreset = statsValue;
		if (
			typeof optionsOrPreset === "boolean" ||
			typeof optionsOrPreset === "string"
		) {
			optionsOrPreset = { preset: optionsOrPreset };
		}
		if (typeof optionsOrPreset === "object" && optionsOrPreset !== null) {
			// We use this method of shallow cloning this object to include
			// properties in the prototype chain
			const options: Partial<NormalizedStatsOptions> = {};
			for (const key in optionsOrPreset) {
				options[key as keyof NormalizedStatsOptions] =
					optionsOrPreset[key as keyof StatsValue];
			}
			if (options.preset !== undefined) {
				this.hooks.statsPreset.for(options.preset).call(options, context);
			}
			this.hooks.statsNormalize.call(options, context);
			return options as NormalizedStatsOptions;
		}
		const options: Partial<NormalizedStatsOptions> = {};
		this.hooks.statsNormalize.call(options, context);
		return options as NormalizedStatsOptions;
	}

	createStatsFactory(options: StatsOptions) {
		const statsFactory = new StatsFactory();
		this.hooks.statsFactory.call(statsFactory, options);
		return statsFactory;
	}

	createStatsPrinter(options: StatsOptions) {
		const statsPrinter = new StatsPrinter();
		this.hooks.statsPrinter.call(statsPrinter, options);
		return statsPrinter;
	}

	/**
	 * Update an existing asset. Trying to update an asset that doesn't exist will throw an error.
	 */
	updateAsset(
		filename: string,
		newSourceOrFunction: Source | ((source: Source) => Source),
		assetInfoUpdateOrFunction?:
			| AssetInfo
			| ((assetInfo: AssetInfo) => AssetInfo | undefined)
	) {
		let compatNewSourceOrFunction: JsSource | ((source: JsSource) => JsSource);

		if (typeof newSourceOrFunction === "function") {
			compatNewSourceOrFunction = function newSourceFunction(source: JsSource) {
				return SourceAdapter.toBinding(
					newSourceOrFunction(SourceAdapter.fromBinding(source))
				);
			};
		} else {
			compatNewSourceOrFunction = SourceAdapter.toBinding(newSourceOrFunction);
		}

		this.#inner.updateAsset(
			filename,
			compatNewSourceOrFunction,
			assetInfoUpdateOrFunction
		);
	}

	/**
	 * Emit an not existing asset. Trying to emit an asset that already exists will throw an error.
	 *
	 * @param file - file name
	 * @param source - asset source
	 * @param assetInfo - extra asset information
	 */
	emitAsset(filename: string, source: Source, assetInfo?: AssetInfo) {
		this.#inner.emitAsset(filename, SourceAdapter.toBinding(source), assetInfo);
	}

	deleteAsset(filename: string) {
		this.#inner.deleteAsset(filename);
	}

	renameAsset(filename: string, newFilename: string) {
		this.#inner.renameAsset(filename, newFilename);
	}

	/**
	 * Get an array of Asset
	 */
	getAssets(): ReadonlyArray<Asset> {
		const assets = this.#inner.getAssets();

		return assets.map(asset => {
			return Object.defineProperties(asset, {
				info: {
					value: asset.info
				},
				source: {
					get: () => this.__internal__getAssetSource(asset.name)
				}
			}) as unknown as Asset;
		});
	}

	getAsset(name: string): Readonly<Asset> | void {
		const asset = this.#inner.getAsset(name);
		if (!asset) {
			return;
		}
		return Object.defineProperties(asset, {
			info: {
				value: asset.info
			},
			source: {
				get: () => this.__internal__getAssetSource(asset.name)
			}
		}) as unknown as Asset;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__pushRspackDiagnostic(diagnostic: binding.JsRspackDiagnostic) {
		this.#inner.pushDiagnostic(diagnostic);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__pushDiagnostic(diagnostic: ExternalObject<"Diagnostic">) {
		this.#inner.pushNativeDiagnostic(diagnostic);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__pushDiagnostics(diagnostics: ExternalObject<"Diagnostic[]">) {
		this.#inner.pushNativeDiagnostics(diagnostics);
	}

	get errors(): RspackError[] {
		if (!this.#errors) {
			this.#errors = createDiagnosticArray(this.#inner.errors);
		}
		return this.#errors;
	}

	set errors(errors: RspackError[]) {
		if (!this.#errors) {
			this.#errors = createDiagnosticArray(this.#inner.errors);
		}
		this.#errors.splice(0, this.#errors.length, ...errors);
	}

	get warnings(): RspackError[] {
		if (!this.#warnings) {
			this.#warnings = createDiagnosticArray(this.#inner.warnings);
		}
		return this.#warnings;
	}

	set warnings(warnings: RspackError[]) {
		if (!this.#warnings) {
			this.#warnings = createDiagnosticArray(this.#inner.warnings);
		}
		this.#warnings.splice(0, this.#warnings.length, ...warnings);
	}

	getPath(filename: string, data: PathData = {}) {
		const pathData: JsPathData = { ...data };
		if (data.contentHashType && data.chunk?.contentHash) {
			pathData.contentHash = data.chunk.contentHash[data.contentHashType];
		}
		return this.#inner.getPath(filename, pathData);
	}

	getPathWithInfo(filename: string, data: PathData = {}) {
		const pathData: JsPathData = { ...data };
		if (data.contentHashType && data.chunk?.contentHash) {
			pathData.contentHash = data.chunk.contentHash[data.contentHashType];
		}
		return this.#inner.getPathWithInfo(filename, pathData);
	}

	getAssetPath(filename: string, data: PathData = {}) {
		const pathData: JsPathData = { ...data };
		if (data.contentHashType && data.chunk?.contentHash) {
			pathData.contentHash = data.chunk.contentHash[data.contentHashType];
		}
		return this.#inner.getAssetPath(filename, pathData);
	}

	getAssetPathWithInfo(filename: string, data: PathData = {}) {
		const pathData: JsPathData = { ...data };
		if (data.contentHashType && data.chunk?.contentHash) {
			pathData.contentHash = data.chunk.contentHash[data.contentHashType];
		}
		return this.#inner.getAssetPathWithInfo(filename, pathData);
	}

	getLogger(name: string | (() => string)) {
		if (!name) {
			throw new TypeError("Compilation.getLogger(name) called without a name");
		}

		let logName = name;
		let logEntries: LogEntry[] | undefined;

		return new Logger(
			(type, args) => {
				if (typeof logName === "function") {
					logName = logName();
					if (!logName) {
						throw new TypeError(
							"Compilation.getLogger(name) called with a function not returning a name"
						);
					}
				}

				const logEntry: LogEntry = {
					time: Date.now(),
					type,
					args,
					get trace() {
						switch (type) {
							case LogType.warn:
							case LogType.error:
							case LogType.trace:
								return cutOffLoaderExecution(new Error("Trace").stack!)
									.split("\n")
									.slice(3);
							default:
								return undefined;
						}
					}
				};

				if (this.hooks.log.call(logName, logEntry) === undefined) {
					if (logEntry.type === LogType.profileEnd) {
						if (typeof console.profileEnd === "function") {
							console.profileEnd(`[${logName}] ${logEntry.args[0]}`);
						}
					}
					if (logEntries === undefined) {
						logEntries = this.logging.get(logName);
						if (logEntries === undefined) {
							logEntries = [];
							this.logging.set(logName, logEntries);
						}
					}
					logEntries.push(logEntry);
					if (logEntry.type === LogType.profile) {
						if (typeof console.profile === "function") {
							console.profile(`[${logName}] ${logEntry.args[0]}`);
						}
					}
				}
			},
			(childName): Logger => {
				let normalizedChildName = childName;
				if (typeof logName === "function") {
					if (typeof normalizedChildName === "function") {
						return this.getLogger(() => {
							if (typeof logName === "function") {
								logName = logName();
								if (!logName) {
									throw new TypeError(
										"Compilation.getLogger(name) called with a function not returning a name"
									);
								}
							}
							if (typeof normalizedChildName === "function") {
								normalizedChildName = normalizedChildName();
								if (!normalizedChildName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${logName}/${normalizedChildName}`;
						});
					}
					return this.getLogger(() => {
						if (typeof logName === "function") {
							logName = logName();
							if (!logName) {
								throw new TypeError(
									"Compilation.getLogger(name) called with a function not returning a name"
								);
							}
						}
						return `${logName}/${normalizedChildName}`;
					});
				}
				if (typeof normalizedChildName === "function") {
					return this.getLogger(() => {
						if (typeof normalizedChildName === "function") {
							normalizedChildName = normalizedChildName();
							if (!normalizedChildName) {
								throw new TypeError(
									"Logger.getChildLogger(name) called with a function not returning a name"
								);
							}
						}
						return `${logName}/${normalizedChildName}`;
					});
				}
				return this.getLogger(`${logName}/${normalizedChildName}`);
			}
		);
	}

	fileDependencies = createFakeCompilationDependencies(
		() => this.#inner.dependencies().fileDependencies,
		d => this.#inner.addFileDependencies(d)
	);

	get __internal__addedFileDependencies() {
		return this.#inner.dependencies().addedFileDependencies;
	}

	get __internal__removedFileDependencies() {
		return this.#inner.dependencies().removedFileDependencies;
	}

	get __internal__addedContextDependencies() {
		return this.#inner.dependencies().addedContextDependencies;
	}

	get __internal__removedContextDependencies() {
		return this.#inner.dependencies().removedContextDependencies;
	}

	get __internal__addedMissingDependencies() {
		return this.#inner.dependencies().addedMissingDependencies;
	}

	get __internal__removedMissingDependencies() {
		return this.#inner.dependencies().removedMissingDependencies;
	}

	contextDependencies = createFakeCompilationDependencies(
		() => this.#inner.dependencies().contextDependencies,
		d => this.#inner.addContextDependencies(d)
	);

	missingDependencies = createFakeCompilationDependencies(
		() => this.#inner.dependencies().missingDependencies,
		d => this.#inner.addMissingDependencies(d)
	);

	buildDependencies = createFakeCompilationDependencies(
		() => this.#inner.dependencies().buildDependencies,
		d => this.#inner.addBuildDependencies(d)
	);

	getStats() {
		return new Stats(this);
	}

	createChildCompiler(
		name: string,
		outputOptions: OutputNormalized,
		plugins: RspackPluginInstance[]
	) {
		const idx = this.childrenCounters[name] || 0;
		this.childrenCounters[name] = idx + 1;
		return this.compiler.createChildCompiler(
			this,
			name,
			idx,
			outputOptions,
			plugins
		);
	}

	#rebuildModuleTask = new AsyncTask<string, Module>(
		(moduleIdentifiers, doneWork) => {
			this.#inner.rebuildModule(
				moduleIdentifiers,
				(err: Error | null, modules: Module[]) => {
					/*
					 * 	TODO:
					 *	batch all call parameters, once a module is failed, we cannot know which module
					 * 	is failed to rebuild, we have to make all modules failed, this should be improved
					 *	in the future
					 */
					if (err) {
						doneWork(new Array(moduleIdentifiers.length).fill([err, null]));
					} else {
						doneWork(modules.map(module => [null, module]));
					}
				}
			);
		}
	);

	rebuildModule(
		module: Module,
		f: (err: Error | null, module: Module | null) => void
	) {
		this.#rebuildModuleTask.exec(module.identifier(), f);
	}

	addRuntimeModule(chunk: Chunk, runtimeModule: RuntimeModule) {
		runtimeModule.attach(this, chunk, this.chunkGraph);
		this.#inner.addRuntimeModule(
			chunk,
			RuntimeModule.__to_binding(this, runtimeModule)
		);
	}

	addInclude(
		context: string,
		dependency: ReturnType<typeof EntryPlugin.createDependency>,
		options: EntryOptions,
		callback: (err?: null | WebpackError, module?: Module) => void
	) {
		this.#addIncludeDispatcher.call(context, dependency, options, callback);
	}

	addEntry(
		context: string,
		dependency: ReturnType<typeof EntryPlugin.createDependency>,
		optionsOrName: EntryOptions | string,
		callback: (err?: null | WebpackError, module?: Module) => void
	) {
		const options =
			typeof optionsOrName === "object"
				? optionsOrName
				: { name: optionsOrName };
		this.#addEntryDispatcher.call(context, dependency, options, callback);
	}

	getWarnings(): WebpackError[] {
		return this.hooks.processWarnings.call(this.#inner.getWarnings());
	}

	getErrors(): WebpackError[] {
		return this.#inner.getErrors();
	}

	/**
	 * Get the `Source` of a given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in the future.
	 *
	 * @internal
	 */
	__internal__getAssetSource(filename: string): Source | void {
		const rawSource = this.#inner.getAssetSource(filename);
		if (!rawSource) {
			return;
		}
		return SourceAdapter.fromBinding(rawSource);
	}

	/**
	 * Set the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__setAssetSource(filename: string, source: Source) {
		this.#inner.setAssetSource(filename, SourceAdapter.toBinding(source));
	}

	/**
	 * Delete the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__deleteAssetSource(filename: string) {
		this.#inner.deleteAssetSource(filename);
	}

	/**
	 * Get a list of asset filenames.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getAssetFilenames(): string[] {
		return this.#inner.getAssetFilenames();
	}

	/**
	 * Test if an asset exists.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__hasAsset(name: string): boolean {
		return this.#inner.hasAsset(name);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal_getInner() {
		return this.#inner;
	}

	get __internal__shutdown() {
		return this.#shutdown;
	}

	set __internal__shutdown(shutdown) {
		this.#shutdown = shutdown;
	}

	seal() {}
	unseal() {}

	static PROCESS_ASSETS_STAGE_ADDITIONAL = -2000;
	static PROCESS_ASSETS_STAGE_PRE_PROCESS = -1000;
	static PROCESS_ASSETS_STAGE_DERIVED = -200;
	static PROCESS_ASSETS_STAGE_ADDITIONS = -100;
	static PROCESS_ASSETS_STAGE_NONE = 0;
	static PROCESS_ASSETS_STAGE_OPTIMIZE = 100;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_COUNT = 200;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_COMPATIBILITY = 300;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE = 400;
	static PROCESS_ASSETS_STAGE_DEV_TOOLING = 500;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE = 700;
	static PROCESS_ASSETS_STAGE_SUMMARIZE = 1000;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_HASH = 2500;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER = 3000;
	static PROCESS_ASSETS_STAGE_ANALYSE = 4000;
	static PROCESS_ASSETS_STAGE_REPORT = 5000;
}

// The AddEntryItemDispatcher class has two responsibilities:
//
// 1. It is responsible for combining multiple addInclude/addEntry calls that occur within the same event loop.
// The purpose of this is to send these combined calls to the add_include/add_entry method on the Rust side in a unified manner, thereby optimizing the call process and avoiding the overhead of multiple scattered calls.
//
// 2. It should be noted that the add_include/add_entry methods on the Rust side has a limitation. It does not allow multiple calls to execute in parallel.
// Based on this limitation, the AddEntryItemDispatcher class needs to properly coordinate and schedule the calls to ensure compliance with this execution rule.
class AddEntryItemDispatcher {
	#inner:
		| binding.JsCompilation["addInclude"]
		| binding.JsCompilation["addEntry"];
	#running: boolean;
	#args: [
		string,
		binding.EntryDependency,
		binding.JsEntryOptions | undefined
	][] = [];
	#cbs: ((err?: null | WebpackError, module?: Module) => void)[] = [];

	#execute = () => {
		if (this.#running) {
			return;
		}

		const args = this.#args;
		this.#args = [];
		const cbs = this.#cbs;
		this.#cbs = [];
		this.#inner(args, (wholeErr, results) => {
			if (this.#args.length !== 0) {
				queueMicrotask(this.#execute.bind(this));
			}

			if (wholeErr) {
				const webpackError = new WebpackError(wholeErr.message);
				for (const cb of cbs) {
					cb(webpackError);
				}
				return;
			}
			for (let i = 0; i < results.length; i++) {
				const [errMsg, module] = results[i];
				const cb = cbs[i];
				cb(errMsg ? new WebpackError(errMsg) : null, module);
			}
		});
	};

	constructor(
		binding:
			| binding.JsCompilation["addInclude"]
			| binding.JsCompilation["addEntry"]
	) {
		this.#inner = binding;
		this.#running = false;
	}

	call(
		context: string,
		dependency: ReturnType<typeof EntryPlugin.createDependency>,
		options: EntryOptions,
		callback: (err?: null | WebpackError, module?: Module) => void
	) {
		if (this.#args.length === 0) {
			queueMicrotask(this.#execute.bind(this));
		}

		this.#args.push([context, dependency, options as any]);
		this.#cbs.push(callback);
	}
}

export class EntryData {
	dependencies: Dependency[];
	includeDependencies: Dependency[];
	options: binding.JsEntryOptions;

	static __from_binding(binding: binding.JsEntryData): EntryData {
		return new EntryData(binding);
	}

	private constructor(binding: binding.JsEntryData) {
		this.dependencies = binding.dependencies;
		this.includeDependencies = binding.includeDependencies;
		this.options = binding.options;
	}
}

export class Entries implements Map<string, EntryData> {
	#data: binding.JsEntries;

	constructor(data: binding.JsEntries) {
		this.#data = data;
	}

	clear(): void {
		this.#data.clear();
	}

	forEach(
		callback: (
			value: EntryData,
			key: string,
			map: Map<string, EntryData>
		) => void,
		thisArg?: any
	): void {
		for (const [key, binding] of this) {
			const value = EntryData.__from_binding(binding);
			callback.call(thisArg, value, key, this);
		}
	}

	get size(): number {
		return this.#data.size;
	}

	*entries(): ReturnType<Map<string, EntryData>["entries"]> {
		for (const key of this.keys()) {
			yield [key, this.get(key)!];
		}
	}

	values(): ReturnType<Map<string, EntryData>["values"]> {
		return this.#data.values().map(EntryData.__from_binding)[Symbol.iterator]();
	}

	[Symbol.iterator](): ReturnType<Map<string, EntryData>["entries"]> {
		return this.entries();
	}

	get [Symbol.toStringTag](): string {
		return "Map";
	}

	has(key: string): boolean {
		return this.#data.has(key);
	}

	set(key: string, value: EntryData): this {
		this.#data.set(key, value);
		return this;
	}

	delete(key: string): boolean {
		return this.#data.delete(key);
	}

	get(key: string): EntryData | undefined {
		const binding = this.#data.get(key);
		return binding ? EntryData.__from_binding(binding) : undefined;
	}

	keys(): ReturnType<Map<string, EntryData>["keys"]> {
		return this.#data.keys()[Symbol.iterator]();
	}
}
