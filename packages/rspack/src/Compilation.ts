/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compilation.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import * as tapable from "tapable";
import { Source } from "webpack-sources";

import type {
	ExternalObject,
	JsAssetInfo,
	JsChunk,
	JsCompatSource,
	JsCompilation,
	JsModule,
	JsRuntimeModule,
	JsStatsChunk,
	JsStatsError,
	PathData
} from "@rspack/binding";

import {
	RspackOptionsNormalized,
	StatsOptions,
	OutputNormalized,
	StatsValue,
	RspackPluginInstance,
	Filename
} from "./config";
import * as liteTapable from "./lite-tapable";
import { ContextModuleFactory } from "./ContextModuleFactory";
import ResolverFactory from "./ResolverFactory";
import { ChunkGroup } from "./ChunkGroup";
import { Compiler } from "./Compiler";
import ErrorHelpers from "./ErrorHelpers";
import { LogType, Logger } from "./logging/Logger";
import { NormalModule } from "./NormalModule";
import { NormalModuleFactory } from "./NormalModuleFactory";
import {
	Stats,
	normalizeFilter,
	normalizeStatsPreset,
	optionsOrFallback
} from "./Stats";
import { StatsFactory } from "./stats/StatsFactory";
import { StatsPrinter } from "./stats/StatsPrinter";
import { concatErrorMsgAndStack, isJsStatsError, toJsAssetInfo } from "./util";
import { createRawFromSource, createSourceFromRaw } from "./util/createSource";
import { createFakeCompilationDependencies } from "./util/fake";
import MergeCaller from "./util/MergeCaller";
import { memoizeValue } from "./util/memoize";
import { Chunk } from "./Chunk";
import { CodeGenerationResult, Module } from "./Module";
import { ChunkGraph } from "./ChunkGraph";
import { Entrypoint } from "./Entrypoint";

export type AssetInfo = Partial<JsAssetInfo> & Record<string, any>;
export type Assets = Record<string, Source>;
export interface Asset {
	name: string;
	source: Source;
	info: JsAssetInfo;
}
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

type CreateStatsOptionsContext = KnownCreateStatsOptionsContext &
	Record<string, any>;

export class Compilation {
	#inner: JsCompilation;

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
		statsFactory: tapable.SyncHook<[StatsFactory, StatsOptions], void>;
		statsPrinter: tapable.SyncHook<[StatsPrinter, StatsOptions], void>;
		buildModule: liteTapable.SyncHook<[Module]>;
		executeModule: liteTapable.SyncHook<
			[ExecuteModuleArgument, ExecuteModuleContext]
		>;
		runtimeModule: liteTapable.SyncHook<[JsRuntimeModule, Chunk], void>;
		afterSeal: liteTapable.AsyncSeriesHook<[], void>;
	};
	options: RspackOptionsNormalized;
	outputOptions: OutputNormalized;
	compiler: Compiler;
	resolverFactory: ResolverFactory;
	inputFileSystem: any;
	logging: Map<string, LogEntry[]>;
	name?: string;
	childrenCounters: Record<string, number> = {};
	startTime?: number;
	endTime?: number;
	children: Compilation[] = [];
	chunkGraph: ChunkGraph;
	fileSystemInfo = {
		createSnapshot() {
			// fake implement to support html-webpack-plugin
			return null;
		}
	};

	constructor(compiler: Compiler, inner: JsCompilation) {
		this.name = undefined;
		this.startTime = undefined;
		this.endTime = undefined;

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
			statsFactory: new tapable.SyncHook(["statsFactory", "options"]),
			statsPrinter: new tapable.SyncHook(["statsPrinter", "options"]),
			buildModule: new liteTapable.SyncHook(["module"]),
			executeModule: new liteTapable.SyncHook(["options", "context"]),
			runtimeModule: new liteTapable.SyncHook(["module", "chunk"]),
			afterSeal: new liteTapable.AsyncSeriesHook([])
		};
		this.compiler = compiler;
		this.resolverFactory = compiler.resolverFactory;
		this.inputFileSystem = compiler.inputFileSystem;
		this.options = compiler.options;
		this.outputOptions = compiler.options.output;
		this.logging = new Map();
		this.chunkGraph = new ChunkGraph(this);
		this.#inner = inner;
		// Cache the current NormalModuleHooks
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
	 *
	 * Source: [assets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L1008-L1009)
	 */
	get assets(): Record<string, Source> {
		return new Proxy(
			{},
			{
				get: (_, property) => {
					if (typeof property === "string") {
						return this.__internal__getAssetSource(property);
					}
				},
				set: (target, p, newValue, receiver) => {
					if (typeof p === "string") {
						this.__internal__setAssetSource(p, newValue);
						return true;
					}
					return false;
				},
				deleteProperty: (target, p) => {
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

	getCache(name: string) {
		return this.compiler.getCache(name);
	}

	createStatsOptions(
		optionsOrPreset: StatsValue | undefined,
		context: CreateStatsOptionsContext = {}
	): StatsOptions {
		optionsOrPreset = normalizeStatsPreset(optionsOrPreset);

		let options: Partial<StatsOptions> = {};
		if (typeof optionsOrPreset === "object" && optionsOrPreset !== null) {
			options = Object.assign({}, optionsOrPreset);
		}

		const all = options.all;
		const optionOrLocalFallback = <V, D>(v: V, def: D) =>
			v !== undefined ? v : all !== undefined ? all : def;

		options.assets = optionOrLocalFallback(options.assets, true);
		options.chunks = optionOrLocalFallback(
			options.chunks,
			!context.forToString
		);
		options.chunkModules = optionOrLocalFallback(
			options.chunkModules,
			!context.forToString
		);
		options.chunkRelations = optionOrLocalFallback(
			options.chunkRelations,
			!context.forToString
		);
		options.modules = optionOrLocalFallback(options.modules, true);
		options.runtimeModules = optionOrLocalFallback(
			options.runtimeModules,
			!context.forToString
		);
		options.reasons = optionOrLocalFallback(
			options.reasons,
			!context.forToString
		);
		options.usedExports = optionOrLocalFallback(
			options.usedExports,
			!context.forToString
		);
		options.optimizationBailout = optionOrLocalFallback(
			options.optimizationBailout,
			!context.forToString
		);
		options.providedExports = optionOrLocalFallback(
			options.providedExports,
			!context.forToString
		);
		options.entrypoints = optionOrLocalFallback(options.entrypoints, true);
		options.chunkGroups = optionOrLocalFallback(
			options.chunkGroups,
			!context.forToString
		);
		options.errors = optionOrLocalFallback(options.errors, true);
		options.errorsCount = optionOrLocalFallback(options.errorsCount, true);
		options.warnings = optionOrLocalFallback(options.warnings, true);
		options.warningsCount = optionOrLocalFallback(options.warningsCount, true);
		options.hash = optionOrLocalFallback(options.hash, true);
		options.version = optionOrLocalFallback(options.version, true);
		options.publicPath = optionOrLocalFallback(options.publicPath, true);
		options.outputPath = optionOrLocalFallback(
			options.outputPath,
			!context.forToString
		);
		options.timings = optionOrLocalFallback(options.timings, true);
		options.builtAt = optionOrLocalFallback(
			options.builtAt,
			!context.forToString
		);
		options.moduleAssets = optionOrLocalFallback(options.moduleAssets, true);
		options.nestedModules = optionOrLocalFallback(
			options.nestedModules,
			!context.forToString
		);
		options.source = optionOrLocalFallback(options.source, false);
		options.logging = optionOrLocalFallback(
			options.logging,
			context.forToString ? "info" : true
		);
		options.loggingTrace = optionOrLocalFallback(
			options.loggingTrace,
			!context.forToString
		);
		options.loggingDebug = []
			.concat(optionsOrFallback(options.loggingDebug, []))
			.map(normalizeFilter);
		options.modulesSpace =
			options.modulesSpace || (context.forToString ? 15 : Infinity);
		options.ids = optionOrLocalFallback(options.ids, !context.forToString);
		options.children = optionOrLocalFallback(
			options.children,
			!context.forToString
		);

		return options;
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
	 * See: [Compilation.updateAsset](https://webpack.js.org/api/compilation-object/#updateasset)
	 * Source: [updateAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4320)
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
				return createRawFromSource(
					newSourceOrFunction(createSourceFromRaw(source))
				);
			};
		} else {
			compatNewSourceOrFunction = createRawFromSource(newSourceOrFunction);
		}

		this.#inner.updateAsset(
			filename,
			compatNewSourceOrFunction,
			assetInfoUpdateOrFunction === undefined
				? assetInfoUpdateOrFunction
				: typeof assetInfoUpdateOrFunction === "function"
					? jsAssetInfo => toJsAssetInfo(assetInfoUpdateOrFunction(jsAssetInfo))
					: toJsAssetInfo(assetInfoUpdateOrFunction)
		);
	}

	/**
	 * Emit an not existing asset. Trying to emit an asset that already exists will throw an error.
	 *
	 * See: [Compilation.emitAsset](https://webpack.js.org/api/compilation-object/#emitasset)
	 * Source: [emitAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4239)
	 *
	 * @param {string} file file name
	 * @param {Source} source asset source
	 * @param {JsAssetInfo} assetInfo extra asset information
	 * @returns {void}
	 */
	emitAsset(filename: string, source: Source, assetInfo?: AssetInfo) {
		this.#inner.emitAsset(
			filename,
			createRawFromSource(source),
			toJsAssetInfo(assetInfo)
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
	 *
	 * See: [Compilation.getAssets](https://webpack.js.org/api/compilation-object/#getassets)
	 * Source: [getAssets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4448)
	 */
	getAssets(): Readonly<Asset>[] {
		const assets = this.#inner.getAssets();

		return assets.map(asset => {
			return Object.defineProperty(asset, "source", {
				get: () => this.__internal__getAssetSource(asset.name)
			}) as Asset;
		});
	}

	getAsset(name: string): Asset | void {
		const asset = this.#inner.getAsset(name);
		if (!asset) {
			return;
		}
		return Object.defineProperty(asset, "source", {
			get: () => this.__internal__getAssetSource(asset.name)
		}) as Asset;
	}

	pushDiagnostic(
		severity: "error" | "warning",
		title: string,
		message: string
	) {
		this.#inner.pushDiagnostic(severity, title, message);
	}

	__internal__pushNativeDiagnostics(diagnostics: ExternalObject<any>) {
		this.#inner.pushNativeDiagnostics(diagnostics);
	}

	get errors() {
		const inner = this.#inner;
		return {
			push: (...errs: (Error | JsStatsError | string)[]) => {
				// compatible for javascript array
				for (let i = 0; i < errs.length; i++) {
					const error = errs[i];
					if (isJsStatsError(error)) {
						this.#inner.pushDiagnostic(
							"error",
							"Error",
							concatErrorMsgAndStack(error)
						);
					} else if (typeof error === "string") {
						this.#inner.pushDiagnostic("error", "Error", error);
					} else {
						this.#inner.pushDiagnostic(
							"error",
							error.name,
							concatErrorMsgAndStack(error)
						);
					}
				}
			},
			[Symbol.iterator]() {
				// TODO: this is obviously a bad design, optimize this after finishing angular prototype
				const errors = inner.getStats().getErrors();
				let index = 0;
				return {
					next() {
						if (index >= errors.length) {
							return { done: true };
						}
						return {
							value: errors[index++],
							done: false
						};
					}
				};
			}
		};
	}

	get warnings() {
		const inner = this.#inner;
		return {
			// compatible for javascript array
			push: (...warns: (Error | JsStatsError)[]) => {
				// TODO: find a way to make JsStatsError be actual errors
				warns = this.hooks.processWarnings.call(warns as any);
				for (let i = 0; i < warns.length; i++) {
					const warn = warns[i];
					this.#inner.pushDiagnostic(
						"warning",
						isJsStatsError(warn) ? "Warning" : warn.name,
						concatErrorMsgAndStack(warn)
					);
				}
			},
			[Symbol.iterator]() {
				// TODO: this is obviously a bad design, optimize this after finishing angular prototype
				const warnings = inner.getStats().getWarnings();
				let index = 0;
				return {
					next() {
						if (index >= warnings.length) {
							return { done: true };
						}
						return {
							value: [warnings[index++]],
							done: false
						};
					}
				};
			}
		};
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
					// @ts-expect-error
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

	get modules() {
		return memoizeValue(() => {
			return this.__internal__getModules().map(item =>
				Module.__from_binding(item)
			);
		});
	}

	// FIXME: This is not aligned with Webpack.
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
					const chunk = this.#inner.getNamedChunk(property);
					return chunk && Chunk.__from_binding(chunk, this.#inner);
				}
			}
		} as Map<string, Readonly<Chunk>>;
	}

	/**
	 * Get the associated `modules` of an given chunk.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getAssociatedModules(chunk: JsStatsChunk): any[] | undefined {
		const modules = this.__internal__getModules();
		const moduleMap: Map<string, JsModule> = new Map();
		for (const module of modules) {
			moduleMap.set(module.moduleIdentifier, module);
		}
		return chunk.modules?.flatMap(chunkModule => {
			const jsModule = this.__internal__findJsModule(
				chunkModule.issuer ?? chunkModule.identifier,
				moduleMap
			);
			return {
				...jsModule
				// dependencies: chunkModule.reasons?.flatMap(jsReason => {
				// 	let jsOriginModule = this.__internal__findJsModule(
				// 		jsReason.moduleIdentifier ?? "",
				// 		moduleMap
				// 	);
				// 	return {
				// 		...jsReason,
				// 		originModule: jsOriginModule
				// 	};
				// })
			};
		});
	}

	/**
	 * Find a modules in an array.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__findJsModule(
		identifier: string,
		modules: Map<string, JsModule>
	): JsModule | undefined {
		return modules.get(identifier);
	}

	/**
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getModules(): JsModule[] {
		return this.#inner.getModules();
	}

	/**
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getChunks(): Chunk[] {
		return this.#inner
			.getChunks()
			.map(c => Chunk.__from_binding(c, this.#inner));
	}

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

	_rebuildModuleCaller = new MergeCaller(
		(args: Array<[string, (err: Error, m: Module) => void]>) => {
			this.#inner.rebuildModule(
				args.map(item => item[0]),
				function (err: Error, modules: JsModule[]) {
					for (const [id, callback] of args) {
						const m = modules.find(item => item.moduleIdentifier === id);
						if (m) {
							callback(err, Module.__from_binding(m));
						} else {
							callback(err || new Error("module no found"), null as any);
						}
					}
				}
			);
		},
		10
	);
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
		return createSourceFromRaw(rawSource);
	}

	/**
	 * Set the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__setAssetSource(filename: string, source: Source) {
		this.#inner.setAssetSource(filename, createRawFromSource(source));
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
