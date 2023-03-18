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
import { RawSource, Source } from "webpack-sources";
import { Resolver } from "enhanced-resolve";

import {
	JsCompilation,
	JsAssetInfo,
	JsCompatSource,
	JsAsset,
	JsModule,
	JsChunk
} from "@rspack/binding";

import {
	RspackOptionsNormalized,
	StatsOptions,
	OutputNormalized,
	StatsValue
} from "./config";
import { createRawFromSource, createSourceFromRaw } from "./util/createSource";
import { ChunkGroup } from "./chunk_group";
import { Compiler } from "./compiler";
import ResolverFactory from "./ResolverFactory";
import {
	createFakeCompilationDependencies,
	createFakeProcessAssetsHook
} from "./util/fake";
import { Logger, LogType } from "./logging/Logger";
import * as ErrorHelpers from "./ErrorHelpers";
import { concatErrorMsgAndStack } from "./util";
import { normalizeStatsPreset, Stats } from "./stats";

const hashDigestLength = 8;
const EMPTY_ASSET_INFO = {};

export type AssetInfo = Partial<JsAssetInfo> & Record<string, any>;
export type Assets = Record<string, Source>;
export interface LogEntry {
	type: string;
	args: any[];
	time: number;
	trace?: string[];
}

export interface KnownCreateStatsOptionsContext {
	forToString?: boolean;
}

type CreateStatsOptionsContext = KnownCreateStatsOptionsContext &
	Record<string, any>;

export class Compilation {
	#inner: JsCompilation;

	hooks: {
		processAssets: ReturnType<typeof createFakeProcessAssetsHook>;
		log: tapable.SyncBailHook<[string, LogEntry], true>;
		optimizeChunkModules: tapable.AsyncSeriesBailHook<
			[Iterable<JsChunk>, Iterable<JsModule>],
			undefined
		>;
	};
	options: RspackOptionsNormalized;
	outputOptions: OutputNormalized;
	compiler: Compiler;
	resolverFactory: ResolverFactory;
	inputFileSystem: any;
	logging: Map<string, LogEntry[]>;
	name?: string;

	constructor(compiler: Compiler, inner: JsCompilation) {
		this.name = undefined;
		this.hooks = {
			processAssets: createFakeProcessAssetsHook(this),
			log: new tapable.SyncBailHook(["origin", "logEntry"]),
			optimizeChunkModules: new tapable.AsyncSeriesBailHook([
				"chunks",
				"modules"
			])
		};
		this.compiler = compiler;
		this.resolverFactory = compiler.resolverFactory;
		this.inputFileSystem = compiler.inputFileSystem;
		this.options = compiler.options;
		this.outputOptions = compiler.options.output;
		this.logging = new Map();
		this.#inner = inner;
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
	get entrypoints(): ReadonlyMap<string, ChunkGroup> {
		return new Map(
			Object.entries(this.#inner.entrypoints).map(([n, e]) => [
				n,
				new ChunkGroup(e)
			])
		);
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
		options.reasons = optionOrLocalFallback(
			options.reasons,
			!context.forToString
		);
		options.entrypoints = optionOrLocalFallback(options.entrypoints, true);
		options.chunkGroups = optionOrLocalFallback(
			options.entrypoints,
			!context.forToString
		);
		options.errors = optionOrLocalFallback(options.errors, true);
		options.errorsCount = optionOrLocalFallback(options.errorsCount, true);
		options.warnings = optionOrLocalFallback(options.warnings, true);
		options.warningsCount = optionOrLocalFallback(options.warningsCount, true);
		options.hash = optionOrLocalFallback(options.hash, true);
		options.publicPath = optionOrLocalFallback(options.publicPath, true);
		options.outputPath = optionOrLocalFallback(
			options.outputPath,
			!context.forToString
		);

		return options;
	}

	/**
	 * Update an existing asset. Trying to update an asset that doesn't exist will throw an error.
	 *
	 * See: [Compilation.updateAsset](https://webpack.js.org/api/compilation-object/#updateasset)
	 * Source: [updateAsset](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4320)
	 *
	 * FIXME: *AssetInfo* may be undefined in update fn for webpack impl, but still not implemented in rspack
	 *
	 * @param {string} file file name
	 * @param {Source | function(Source): Source} newSourceOrFunction new asset source or function converting old to new
	 * @param {JsAssetInfo | function(JsAssetInfo): JsAssetInfo} assetInfoUpdateOrFunction new asset info or function converting old to new
	 */
	updateAsset(
		filename: string,
		newSourceOrFunction: Source | ((source: Source) => Source),
		assetInfoUpdateOrFunction:
			| JsAssetInfo
			| ((assetInfo: JsAssetInfo) => JsAssetInfo)
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
			assetInfoUpdateOrFunction
		);
	}

	/**
	 *
	 * @param moduleIdentifier moduleIdentifier of the module you want to modify
	 * @param source
	 * @returns true if the setting is success, false if failed.
	 */
	setNoneAstModuleSource(
		moduleIdentifier: string,
		source: JsCompatSource
	): boolean {
		return this.#inner.setNoneAstModuleSource(moduleIdentifier, source);
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
		const info = Object.assign(
			{
				minimized: false,
				development: false,
				hotModuleReplacement: false,
				related: {}
			},
			assetInfo
		);
		this.#inner.emitAsset(filename, createRawFromSource(source), info);
	}

	deleteAsset(filename: string) {
		this.#inner.deleteAsset(filename);
	}

	/**
	 * Get an array of Asset
	 *
	 * See: [Compilation.getAssets](https://webpack.js.org/api/compilation-object/#getassets)
	 * Source: [getAssets](https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L4448)
	 *
	 * @return {Readonly<JsAsset>[]}
	 */
	getAssets() {
		const assets = this.#inner.getAssets();

		return assets.map(asset => {
			// @ts-expect-error
			const source = createSourceFromRaw(asset.source);
			return {
				...asset,
				source
			};
		});
	}

	getAsset(name: string) {
		const asset = this.#inner.getAsset(name);
		if (!asset) {
			return;
		}
		return {
			...asset,
			// @ts-expect-error
			source: createSourceFromRaw(asset.source)
		};
	}

	pushDiagnostic(
		severity: "error" | "warning",
		title: string,
		message: string
	) {
		this.#inner.pushDiagnostic(severity, title, message);
	}

	get errors() {
		return {
			push: (err: Error) => {
				this.#inner.pushDiagnostic(
					"error",
					err.name,
					concatErrorMsgAndStack(err)
				);
			}
		};
	}

	get warnings() {
		return {
			push: (warn: Error) => {
				this.#inner.pushDiagnostic(
					"warning",
					warn.name,
					concatErrorMsgAndStack(warn)
				);
			}
		};
	}

	// TODO: full alignment
	getPath(filename: string, data: Record<string, any> = {}) {
		if (!data.hash) {
			data = {
				hash: this.hash,
				...data
			};
		}
		return this.getAssetPath(filename, data);
	}

	// TODO: full alignment
	// @ts-expect-error
	getAssetPath(filename, data) {
		return filename;
	}
	// @ts-expect-error
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
			childName => {
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

	get fileDependencies() {
		return createFakeCompilationDependencies(
			this.#inner.getFileDependencies(),
			d => this.#inner.addFileDependencies(d)
		);
	}

	get contextDependencies() {
		return createFakeCompilationDependencies(
			this.#inner.getContextDependencies(),
			d => this.#inner.addContextDependencies(d)
		);
	}

	get missingDependencies() {
		return createFakeCompilationDependencies(
			this.#inner.getMissingDependencies(),
			d => this.#inner.addMissingDependencies(d)
		);
	}

	get buildDependencies() {
		return createFakeCompilationDependencies(
			this.#inner.getBuildDependencies(),
			d => this.#inner.addBuildDependencies(d)
		);
	}

	getModules(): JsModule[] {
		return this.#inner.getModules();
	}
	getChunks(): JsChunk[] {
		return this.#inner.getChunks();
	}

	getStats() {
		return new Stats(this);
	}

	/**
	 * Get the `Source` of an given asset filename.
	 *
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__getAssetSource(filename: string): Source | null {
		const rawSource = this.#inner.getAssetSource(filename);
		if (!rawSource) {
			return null;
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
	static PROCESS_ASSETS_STAGE_NONE = 0;
	static PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE = 700;
	static PROCESS_ASSETS_STAGE_SUMMARIZE = 1000;
	static PROCESS_ASSETS_STAGE_REPORT = 5000;

	__internal_getProcessAssetsHookByStage(stage: number) {
		switch (stage) {
			case Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL:
				return this.hooks.processAssets.stageAdditional;
			case Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS:
				return this.hooks.processAssets.stagePreProcess;
			case Compilation.PROCESS_ASSETS_STAGE_NONE:
				return this.hooks.processAssets.stageNone;
			case Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE:
				return this.hooks.processAssets.stageOptimizeInline;
			case Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE:
				return this.hooks.processAssets.stageSummarize;
			case Compilation.PROCESS_ASSETS_STAGE_REPORT:
				return this.hooks.processAssets.stageReport;
			default:
				throw new Error(
					"processAssets hook uses custom stage number is not supported."
				);
		}
	}
}

export type { JsAssetInfo };
