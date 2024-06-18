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
	ExternalObject,
	JsCompatSource,
	JsCompilation,
	JsDiagnostic,
	JsModule,
	JsPathData,
	JsRuntimeModule,
	JsStatsError,
	JsStatsWarning
} from "@rspack/binding";
import * as tapable from "tapable";
import { Source } from "webpack-sources";

import { ContextModuleFactory } from "./ContextModuleFactory";
import {
	Filename,
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance,
	StatsOptions,
	StatsValue
} from "./config";
import * as liteTapable from "./lite-tapable";
import ResolverFactory = require("./ResolverFactory");
import { Chunk } from "./Chunk";
import { ChunkGraph } from "./ChunkGraph";
import { Compiler } from "./Compiler";
import { Entrypoint } from "./Entrypoint";
import ErrorHelpers from "./ErrorHelpers";
import { CodeGenerationResult, Module } from "./Module";
import { NormalModule } from "./NormalModule";
import { NormalModuleFactory } from "./NormalModuleFactory";
import { Stats, StatsAsset, StatsError, StatsModule } from "./Stats";
import { LogType, Logger } from "./logging/Logger";
import { StatsFactory } from "./stats/StatsFactory";
import { StatsPrinter } from "./stats/StatsPrinter";
import { concatErrorMsgAndStack } from "./util";
import { type AssetInfo, JsAssetInfo } from "./util/AssetInfo";
import MergeCaller from "./util/MergeCaller";
import { createFakeCompilationDependencies } from "./util/fake";
import { memoizeValue } from "./util/memoize";
import { JsSource } from "./util/source";
export { type AssetInfo } from "./util/AssetInfo";

export type Assets = Record<string, Source>;
export interface Asset {
	name: string;
	source: Source;
	info: AssetInfo;
}

export type PathData = JsPathData;

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
}

export type CreateStatsOptionsContext = KnownCreateStatsOptionsContext &
	Record<string, any>;

export type NormalizedStatsOptions = KnownNormalizedStatsOptions &
	Omit<StatsOptions, keyof KnownNormalizedStatsOptions> &
	Record<string, any>;

export class Compilation {
	#inner: JsCompilation;
	#cachedAssets: Record<string, Source>;

	hooks: {
		processAssets: liteTapable.AsyncSeriesHook<Assets>;
		afterProcessAssets: liteTapable.SyncHook<Assets>;
		childCompiler: tapable.SyncHook<[Compiler, string, number]>;
		log: tapable.SyncBailHook<[string, LogEntry], true>;
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
		chunkAsset: liteTapable.SyncHook<[Chunk, string], void>;
		processWarnings: tapable.SyncWaterfallHook<[Error[]]>;
		succeedModule: liteTapable.SyncHook<[Module], void>;
		stillValidModule: liteTapable.SyncHook<[Module], void>;

		statsPreset: tapable.HookMap<
			tapable.SyncHook<[Partial<StatsOptions>, CreateStatsOptionsContext], void>
		>;
		statsNormalize: tapable.SyncHook<
			[Partial<StatsOptions>, CreateStatsOptionsContext],
			void
		>;
		statsFactory: tapable.SyncHook<[StatsFactory, StatsOptions], void>;
		statsPrinter: tapable.SyncHook<[StatsPrinter, StatsOptions], void>;

		buildModule: liteTapable.SyncHook<[Module]>;
		executeModule: liteTapable.SyncHook<
			[ExecuteModuleArgument, ExecuteModuleContext]
		>;
		additionalTreeRuntimeRequirements: liteTapable.SyncHook<
			[Chunk, Set<string>],
			void
		>;
		runtimeModule: liteTapable.SyncHook<[JsRuntimeModule, Chunk], void>;
		afterSeal: liteTapable.AsyncSeriesHook<[], void>;
	};
	name?: string;
	startTime?: number;
	endTime?: number;
	compiler: Compiler;

	resolverFactory: ResolverFactory;
	inputFileSystem: any;
	options: RspackOptionsNormalized;
	outputOptions: OutputNormalized;
	logging: Map<string, LogEntry[]>;
	childrenCounters: Record<string, number>;
	children: Compilation[];
	chunkGraph: ChunkGraph;
	fileSystemInfo = {
		createSnapshot() {
			// fake implement to support html-webpack-plugin
			return null;
		}
	};

	/**
	 * Records the dynamically added fields for Module on the JavaScript side, using the Module identifier for association.
	 * These fields are generally used within a plugin, so they do not need to be passed back to the Rust side.
	 */
	#customModules: Record<
		string,
		{
			buildInfo: Record<string, unknown>;
			buildMeta: Record<string, unknown>;
		}
	>;

	constructor(compiler: Compiler, inner: JsCompilation) {
		this.#inner = inner;
		this.#cachedAssets = this.#createCachedAssets();
		this.#customModules = {};

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
				if (typeof options === "string") options = { name: options };
				if (options.stage) {
					throw new Error(errorMessage("it's using the 'stage' option"));
				}
				return { ...options, stage: stage };
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
					fn: liteTapable.FnWithCallback<T, void>
				) => {
					processAssetsHook.tapAsync(getOptions(options), (assets, callback) =>
						(fn as any)(...getArgs(), callback)
					);
				},
				tapPromise: (
					options: liteTapable.Options,
					fn: liteTapable.Fn<T, void>
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
			childCompiler: new tapable.SyncHook([
				"childCompiler",
				"compilerName",
				"compilerIndex"
			]),
			log: new tapable.SyncBailHook(["origin", "logEntry"]),
			optimizeModules: new liteTapable.SyncBailHook(["modules"]),
			afterOptimizeModules: new liteTapable.SyncBailHook(["modules"]),
			optimizeTree: new liteTapable.AsyncSeriesHook(["chunks", "modules"]),
			optimizeChunkModules: new liteTapable.AsyncSeriesBailHook([
				"chunks",
				"modules"
			]),
			finishModules: new liteTapable.AsyncSeriesHook(["modules"]),
			chunkAsset: new liteTapable.SyncHook(["chunk", "filename"]),
			processWarnings: new tapable.SyncWaterfallHook(["warnings"]),
			succeedModule: new liteTapable.SyncHook(["module"]),
			stillValidModule: new liteTapable.SyncHook(["module"]),

			statsPreset: new tapable.HookMap(
				() => new tapable.SyncHook(["options", "context"])
			),
			statsNormalize: new tapable.SyncHook(["options", "context"]),
			statsFactory: new tapable.SyncHook(["statsFactory", "options"]),
			statsPrinter: new tapable.SyncHook(["statsPrinter", "options"]),

			buildModule: new liteTapable.SyncHook(["module"]),
			executeModule: new liteTapable.SyncHook(["options", "context"]),
			additionalTreeRuntimeRequirements: new liteTapable.SyncHook([
				"chunk",
				"runtimeRequirements"
			]),
			runtimeModule: new liteTapable.SyncHook(["module", "chunk"]),
			afterSeal: new liteTapable.AsyncSeriesHook([])
		};
		this.compiler = compiler;
		this.resolverFactory = compiler.resolverFactory;
		this.inputFileSystem = compiler.inputFileSystem;
		this.options = compiler.options;
		this.outputOptions = compiler.options.output;
		this.logging = new Map();
		this.childrenCounters = {};
		this.children = [];
		this.chunkGraph = new ChunkGraph(this);
	}

	get currentNormalModuleHooks() {
		return NormalModule.getCompilationHooks(this);
	}

	get hash() {
		return this.#inner.hash;
	}

	get fullHash() {
		return this.#inner.hash;
	}

	/**
	 * Get a map of all assets.
	 */
	get assets(): Record<string, Source> {
		return this.#cachedAssets;
	}

	/**
	 * Get a map of all entrypoints.
	 */
	get entrypoints(): ReadonlyMap<string, Entrypoint> {
		return new Map(
			Object.entries(this.#inner.entrypoints).map(([n, e]) => [
				n,
				Entrypoint.__from_binding(e, this.#inner)
			])
		);
	}

	get modules() {
		return memoizeValue(() => {
			return this.__internal__getModules().map(item =>
				Module.__from_binding(item, this)
			);
		});
	}

	// FIXME: Webpack returns a `Set`
	get chunks() {
		return memoizeValue(() => {
			return this.__internal__getChunks();
		});
	}

	/**
	 * Get the named chunks.
	 *
	 * Note: This is a proxy for webpack internal API, only method `get` is supported now.
	 */
	get namedChunks(): Map<string, Readonly<Chunk>> {
		return {
			get: (property: unknown) => {
				if (typeof property === "string") {
					const chunk = this.#inner.getNamedChunk(property) || undefined;
					return chunk && Chunk.__from_binding(chunk, this.#inner);
				}
			}
		} as Map<string, Readonly<Chunk>>;
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

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getCustomModule(moduleIdentifier: string) {
		let module = this.#customModules[moduleIdentifier];
		if (!module) {
			module = this.#customModules[moduleIdentifier] = {
				buildInfo: {},
				buildMeta: {}
			};
		}
		return module;
	}

	getCache(name: string) {
		return this.compiler.getCache(name);
	}

	createStatsOptions(
		optionsOrPreset: StatsValue | undefined,
		context: CreateStatsOptionsContext = {}
	): NormalizedStatsOptions {
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
		} else {
			const options: Partial<NormalizedStatsOptions> = {};
			this.hooks.statsNormalize.call(options, context);
			return options as NormalizedStatsOptions;
		}
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
	 *
	 * FIXME: *AssetInfo* may be undefined in update fn for webpack impl, but still not implemented in rspack
	 */
	updateAsset(
		filename: string,
		newSourceOrFunction: Source | ((source: Source) => Source),
		assetInfoUpdateOrFunction?:
			| AssetInfo
			| ((assetInfo: AssetInfo) => AssetInfo)
	) {
		let compatNewSourceOrFunction:
			| JsCompatSource
			| ((source: JsCompatSource) => JsCompatSource);

		if (typeof newSourceOrFunction === "function") {
			compatNewSourceOrFunction = function newSourceFunction(
				source: JsCompatSource
			) {
				return JsSource.__to_binding(
					newSourceOrFunction(JsSource.__from_binding(source))
				);
			};
		} else {
			compatNewSourceOrFunction = JsSource.__to_binding(newSourceOrFunction);
		}

		this.#inner.updateAsset(
			filename,
			compatNewSourceOrFunction,
			assetInfoUpdateOrFunction === undefined
				? assetInfoUpdateOrFunction
				: typeof assetInfoUpdateOrFunction === "function"
					? jsAssetInfo =>
							JsAssetInfo.__to_binding(assetInfoUpdateOrFunction(jsAssetInfo))
					: JsAssetInfo.__to_binding(assetInfoUpdateOrFunction)
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
		this.#inner.emitAsset(
			filename,
			JsSource.__to_binding(source),
			JsAssetInfo.__to_binding(assetInfo)
		);
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
					value: JsAssetInfo.__from_binding(asset.info)
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
				value: JsAssetInfo.__from_binding(asset.info)
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
	__internal__pushDiagnostic(
		severity: "error" | "warning",
		title: string,
		message: string
	) {
		this.#inner.pushDiagnostic(severity, title, message);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__pushNativeDiagnostics(
		diagnostics: ExternalObject<"Diagnostic[]">
	) {
		this.#inner.pushNativeDiagnostics(diagnostics);
	}

	get errors(): JsStatsError[] {
		const inner = this.#inner;
		type ErrorType = Error | JsStatsError | string;
		const errors = inner.getStats().getErrors();
		const proxyMethod = [
			{
				method: "push",
				handler(
					target: typeof Array.prototype.push,
					thisArg: Array<ErrorType>,
					errs: ErrorType[]
				) {
					for (let i = 0; i < errs.length; i++) {
						const error = errs[i];
						inner.pushDiagnostic(
							"error",
							error instanceof Error ? error.name : "Error",
							concatErrorMsgAndStack(error)
						);
					}
					return Reflect.apply(target, thisArg, errs);
				}
			},
			{
				method: "pop",
				handler(target: typeof Array.prototype.pop, thisArg: Array<ErrorType>) {
					inner.spliceDiagnostic(errors.length - 1, errors.length, []);
					return Reflect.apply(target, thisArg, []);
				}
			},
			{
				method: "shift",
				handler(
					target: typeof Array.prototype.shift,
					thisArg: Array<ErrorType>
				) {
					inner.spliceDiagnostic(0, 1, []);
					return Reflect.apply(target, thisArg, []);
				}
			},
			{
				method: "unshift",
				handler(
					target: typeof Array.prototype.unshift,
					thisArg: Array<ErrorType>,
					errs: ErrorType[]
				) {
					const errList = errs.map(error => {
						return {
							severity: "error",
							title: error instanceof Error ? error.name : "Error",
							message: concatErrorMsgAndStack(error)
						} satisfies JsDiagnostic;
					});
					inner.spliceDiagnostic(0, 0, errList);
					return Reflect.apply(target, thisArg, errs);
				}
			},
			{
				method: "splice",
				handler(
					target: typeof Array.prototype.splice,
					thisArg: Array<ErrorType>,
					[startIdx, delCount, ...errors]: [number, number, ...ErrorType[]]
				) {
					const errList = errors.map(error => {
						return {
							severity: "error",
							title: error instanceof Error ? error.name : "Error",
							message: concatErrorMsgAndStack(error)
						} satisfies JsDiagnostic;
					});
					inner.spliceDiagnostic(startIdx, startIdx + delCount, errList);
					return Reflect.apply(target, thisArg, [
						startIdx,
						delCount,
						...errors
					]);
				}
			}
		];
		proxyMethod.forEach(item => {
			const proxyedMethod = new Proxy(errors[item.method as any], {
				apply: item.handler as any
			});
			errors[item.method as any] = proxyedMethod;
		});
		return errors;
	}

	get warnings(): JsStatsWarning[] {
		const inner = this.#inner;
		type WarnType = Error | JsStatsWarning;
		const processWarningsHook = this.hooks.processWarnings;
		const warnings = inner.getStats().getWarnings();
		const proxyMethod = [
			{
				method: "push",
				handler(
					target: typeof Array.prototype.push,
					thisArg: Array<WarnType>,
					warns: WarnType[]
				) {
					warns = processWarningsHook.call(warns as any);
					for (let i = 0; i < warns.length; i++) {
						const warn = warns[i];
						inner.pushDiagnostic(
							"warning",
							warn instanceof Error ? warn.name : "Warning",
							concatErrorMsgAndStack(warn)
						);
					}
					return Reflect.apply(target, thisArg, warns);
				}
			},
			{
				method: "pop",
				handler(target: typeof Array.prototype.pop, thisArg: Array<WarnType>) {
					inner.spliceDiagnostic(warnings.length - 1, warnings.length, []);
					return Reflect.apply(target, thisArg, []);
				}
			},
			{
				method: "shift",
				handler(
					target: typeof Array.prototype.shift,
					thisArg: Array<WarnType>
				) {
					inner.spliceDiagnostic(0, 1, []);
					return Reflect.apply(target, thisArg, []);
				}
			},
			{
				method: "unshift",
				handler(
					target: typeof Array.prototype.unshift,
					thisArg: Array<WarnType>,
					warns: WarnType[]
				) {
					warns = processWarningsHook.call(warns as any);
					const warnList = warns.map(warn => {
						return {
							severity: "warning",
							title: warn instanceof Error ? warn.name : "Warning",
							message: concatErrorMsgAndStack(warn)
						} satisfies JsDiagnostic;
					});
					inner.spliceDiagnostic(0, 0, warnList);
					return Reflect.apply(target, thisArg, warns);
				}
			},
			{
				method: "splice",
				handler(
					target: typeof Array.prototype.splice,
					thisArg: Array<WarnType>,
					[startIdx, delCount, ...warns]: [number, number, ...WarnType[]]
				) {
					warns = processWarningsHook.call(warns as any);
					const warnList = warns.map(warn => {
						return {
							severity: "warning",
							title: warn instanceof Error ? warn.name : "Warning",
							message: concatErrorMsgAndStack(warn)
						} satisfies JsDiagnostic;
					});
					inner.spliceDiagnostic(startIdx, startIdx + delCount, warnList);
					return Reflect.apply(target, thisArg, [
						startIdx,
						delCount,
						...warnList
					]);
				}
			}
		];
		proxyMethod.forEach(item => {
			const proxyedMethod = new Proxy(warnings[item.method as any], {
				apply: item.handler as any
			});
			warnings[item.method as any] = proxyedMethod;
		});
		return warnings;
	}

	getPath(filename: Filename, data: PathData = {}) {
		return this.#inner.getPath(filename, data);
	}

	getPathWithInfo(filename: Filename, data: PathData = {}) {
		return this.#inner.getPathWithInfo(filename, data);
	}

	getAssetPath(filename: Filename, data: PathData = {}) {
		return this.#inner.getAssetPath(filename, data);
	}

	getAssetPathWithInfo(filename: Filename, data: PathData = {}) {
		return this.#inner.getAssetPathWithInfo(filename, data);
	}

	getLogger(name: string | (() => string)) {
		if (!name) {
			throw new TypeError("Compilation.getLogger(name) called without a name");
		}
		let logEntries: LogEntry[] | undefined;
		return new Logger(
			(type, args) => {
				if (typeof name === "function") {
					name = name();
					if (!name) {
						throw new TypeError(
							"Compilation.getLogger(name) called with a function not returning a name"
						);
					}
				}
				let trace: string[];
				switch (type) {
					case LogType.warn:
					case LogType.error:
					case LogType.trace:
						trace = ErrorHelpers.cutOffLoaderExecution(new Error("Trace").stack)
							.split("\n")
							.slice(3);
						break;
				}
				const logEntry: LogEntry = {
					time: Date.now(),
					type,
					args,
					// @ts-expect-error
					trace
				};
				if (this.hooks.log.call(name, logEntry) === undefined) {
					if (logEntry.type === LogType.profileEnd) {
						if (typeof console.profileEnd === "function") {
							console.profileEnd(`[${name}] ${logEntry.args[0]}`);
						}
					}
					if (logEntries === undefined) {
						logEntries = this.logging.get(name);
						if (logEntries === undefined) {
							logEntries = [];
							this.logging.set(name, logEntries);
						}
					}
					logEntries.push(logEntry);
					if (logEntry.type === LogType.profile) {
						if (typeof console.profile === "function") {
							console.profile(`[${name}] ${logEntry.args[0]}`);
						}
					}
				}
			},
			(childName): Logger => {
				if (typeof name === "function") {
					if (typeof childName === "function") {
						return this.getLogger(() => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compilation.getLogger(name) called with a function not returning a name"
									);
								}
							}
							if (typeof childName === "function") {
								childName = childName();
								if (!childName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					} else {
						return this.getLogger(() => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compilation.getLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					}
				} else {
					if (typeof childName === "function") {
						return this.getLogger(() => {
							if (typeof childName === "function") {
								childName = childName();
								if (!childName) {
									throw new TypeError(
										"Logger.getChildLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					} else {
						return this.getLogger(`${name}/${childName}`);
					}
				}
			}
		);
	}

	fileDependencies = createFakeCompilationDependencies(
		() => this.#inner.getFileDependencies(),
		d => this.#inner.addFileDependencies(d)
	);

	contextDependencies = createFakeCompilationDependencies(
		() => this.#inner.getContextDependencies(),
		d => this.#inner.addContextDependencies(d)
	);

	missingDependencies = createFakeCompilationDependencies(
		() => this.#inner.getMissingDependencies(),
		d => this.#inner.addMissingDependencies(d)
	);

	buildDependencies = createFakeCompilationDependencies(
		() => this.#inner.getBuildDependencies(),
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

	_rebuildModuleCaller = (function (compilation: Compilation) {
		return new MergeCaller(
			(args: Array<[string, (err: Error, m: Module) => void]>) => {
				compilation.#inner.rebuildModule(
					args.map(item => item[0]),
					function (err: Error, modules: JsModule[]) {
						for (const [id, callback] of args) {
							const m = modules.find(item => item.moduleIdentifier === id);
							if (m) {
								callback(err, Module.__from_binding(m, compilation));
							} else {
								callback(err || new Error("module no found"), null as any);
							}
						}
					}
				);
			},
			10
		);
	})(this);

	rebuildModule(m: Module, f: (err: Error, m: Module) => void) {
		this._rebuildModuleCaller.push([m.identifier(), f]);
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
		return JsSource.__from_binding(rawSource);
	}

	/**
	 * Set the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__setAssetSource(filename: string, source: Source) {
		this.#inner.setAssetSource(filename, JsSource.__to_binding(source));
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
	__internal__getModules(): JsModule[] {
		return this.#inner.getModules();
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getChunks(): Chunk[] {
		return this.#inner
			.getChunks()
			.map(c => Chunk.__from_binding(c, this.#inner));
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal_getInner() {
		return this.#inner;
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
