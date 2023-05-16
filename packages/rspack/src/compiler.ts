/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import * as binding from "@rspack/binding";
import fs from "fs";
import path from "path";
import * as tapable from "tapable";
import { Callback, SyncBailHook, SyncHook } from "tapable";
import type { WatchOptions } from "watchpack";
import {
	OutputNormalized,
	RspackOptionsNormalized,
	RspackPluginInstance
} from "./config";
import { RuleSetCompiler } from "./RuleSetCompiler";
import { Stats } from "./stats";
import { Compilation, CompilationParams } from "./compilation";
import { ContextModuleFactory } from "./ContextModuleFactory";
import ResolverFactory from "./ResolverFactory";
import { getRawOptions } from "./config/adapter";
import { LoaderContext, LoaderResult } from "./config/adapter-rule-use";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import { createThreadsafeNodeFSFromRaw } from "./fileSystem";
import Cache from "./lib/Cache";
import { makePathsRelative } from "./util/identifier";
import CacheFacade from "./lib/CacheFacade";
import { runLoader } from "./loader-runner";
import { Logger } from "./logging/Logger";
import { NormalModuleFactory } from "./normalModuleFactory";
import { WatchFileSystem } from "./util/fs";
import { getScheme } from "./util/scheme";
import Watching from "./watching";
import { NormalModule } from "./normalModule";

class EntryPlugin {
	constructor(
		public context: string,
		public entry: string,
		public options: {
			name?: string;
			runtime?: string;
		} = {}
	) {}
	apply(compiler: Compiler) {
		const entry = this.context
			? path.resolve(this.context, this.entry)
			: this.entry;

		compiler.options.entry = {
			[this.options.name || "main"]: {
				import: [entry],
				runtime: this.options.runtime
			}
		};
	}
}

class NodeTargetPlugin {
	apply() {}
}

class NodeTemplatePlugin {
	apply() {}
}

class EnableLibraryPlugin {
	constructor(private libraryType: string) {}
	apply(compiler: Compiler) {
		compiler.options.output.enabledLibraryTypes = [this.libraryType];
	}
}
class HotModuleReplacementPlugin {
	apply() {}
}

class Compiler {
	#_instance?: binding.Rspack;

	webpack: any;
	// @ts-expect-error
	compilation: Compilation;
	root: Compiler;
	running: boolean;
	resolverFactory: ResolverFactory;
	infrastructureLogger: any;
	watching?: Watching;
	outputPath!: string;
	name?: string;
	inputFileSystem: any;
	outputFileSystem: typeof import("fs");
	ruleSet: RuleSetCompiler;
	// @ts-expect-error
	watchFileSystem: WatchFileSystem;
	intermediateFileSystem: any;
	// @ts-expect-error
	watchMode: boolean;
	context: string;
	modifiedFiles?: ReadonlySet<string>;
	cache: Cache;
	compilerPath: string;
	removedFiles?: ReadonlySet<string>;
	hooks: {
		done: tapable.AsyncSeriesHook<Stats>;
		afterDone: tapable.SyncHook<Stats>;
		// TODO: CompilationParams
		compilation: tapable.SyncHook<[Compilation, CompilationParams]>;
		// TODO: CompilationParams
		thisCompilation: tapable.SyncHook<[Compilation, CompilationParams]>;
		invalid: tapable.SyncHook<[string | null, number]>;
		compile: tapable.SyncHook<[any]>;
		normalModuleFactory: tapable.SyncHook<NormalModuleFactory>;
		contextModuleFactory: tapable.SyncHook<ContextModuleFactory>;
		initialize: tapable.SyncHook<[]>;
		infrastructureLog: tapable.SyncBailHook<[string, string, any[]], true>;
		beforeRun: tapable.AsyncSeriesHook<[Compiler]>;
		run: tapable.AsyncSeriesHook<[Compiler]>;
		emit: tapable.AsyncSeriesHook<[Compilation]>;
		afterEmit: tapable.AsyncSeriesHook<[Compilation]>;
		failed: tapable.SyncHook<[Error]>;
		watchRun: tapable.AsyncSeriesHook<[Compiler]>;
		watchClose: tapable.SyncHook<[]>;
		environment: tapable.SyncHook<[]>;
		afterEnvironment: tapable.SyncHook<[]>;
		afterPlugins: tapable.SyncHook<[Compiler]>;
		afterResolvers: tapable.SyncHook<[Compiler]>;
		make: tapable.AsyncParallelHook<[Compilation]>;
		beforeCompile: tapable.AsyncSeriesHook<any>;
		finishModules: tapable.AsyncSeriesHook<[any]>;
	};
	options: RspackOptionsNormalized;
	#disabledHooks: string[];
	parentCompilation?: Compilation;
	constructor(context: string, options: RspackOptionsNormalized) {
		this.outputFileSystem = fs;
		this.options = options;
		this.cache = new Cache();
		this.compilerPath = "";
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin, // modernjs/server will auto inject this plugin not set
			NormalModule,
			get sources(): typeof import("webpack-sources") {
				return require("webpack-sources");
			},
			Compilation,
			get version() {
				return require("../package.json").webpackVersion; // this is a hack to be compatible with plugin which detect webpack's version
			},
			get rspackVersion() {
				return require("../package.json").version;
			},
			WebpackError: Error,
			node: {
				NodeTargetPlugin,
				NodeTemplatePlugin
			},
			library: {
				EnableLibraryPlugin
			},
			util: {
				get createHash() {
					return require("./util/createHash").createHash;
				},
				get cleverMerge() {
					return require("./util/cleverMerge").cachedCleverMerge;
				}
				// get comparators() {
				// 	return require("./util/comparators");
				// },
				// get runtime() {
				// 	return require("./util/runtime");
				// },
				// get serialization() {
				// 	return require("./util/serialization");
				// },
				// get LazySet() {
				// 	return require("./util/LazySet");
				// }
			}
		};
		this.root = this;
		this.ruleSet = new RuleSetCompiler();
		this.running = false;
		this.context = context;
		this.resolverFactory = new ResolverFactory();
		this.modifiedFiles = undefined;
		this.removedFiles = undefined;
		this.hooks = {
			initialize: new SyncHook([]),
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new tapable.SyncHook<Stats>(["stats"]),
			beforeRun: new tapable.AsyncSeriesHook(["compiler"]),
			run: new tapable.AsyncSeriesHook(["compiler"]),
			emit: new tapable.AsyncSeriesHook(["compilation"]),
			afterEmit: new tapable.AsyncSeriesHook(["compilation"]),
			thisCompilation: new tapable.SyncHook<[Compilation, CompilationParams]>([
				"compilation",
				"params"
			]),
			compilation: new tapable.SyncHook<[Compilation, CompilationParams]>([
				"compilation",
				"params"
			]),
			invalid: new SyncHook(["filename", "changeTime"]),
			compile: new SyncHook(["params"]),
			infrastructureLog: new SyncBailHook(["origin", "type", "args"]),
			failed: new SyncHook(["error"]),
			normalModuleFactory: new tapable.SyncHook<NormalModuleFactory>([
				"normalModuleFactory"
			]),
			contextModuleFactory: new tapable.SyncHook<ContextModuleFactory>([
				"contextModuleFactory"
			]),
			watchRun: new tapable.AsyncSeriesHook(["compiler"]),
			watchClose: new tapable.SyncHook([]),
			environment: new tapable.SyncHook([]),
			afterEnvironment: new tapable.SyncHook([]),
			afterPlugins: new tapable.SyncHook(["compiler"]),
			afterResolvers: new tapable.SyncHook(["compiler"]),
			make: new tapable.AsyncParallelHook(["compilation"]),
			beforeCompile: new tapable.AsyncSeriesHook(["params"]),
			finishModules: new tapable.AsyncSeriesHook(["modules"])
		};
		this.modifiedFiles = undefined;
		this.removedFiles = undefined;
		this.#disabledHooks = [];
	}

	/**
	 * @param {string} name cache name
	 * @returns {CacheFacade} the cache facade instance
	 */
	getCache(name: string): CacheFacade {
		return new CacheFacade(
			this.cache,
			`${this.compilerPath}${name}`,
			this.options.output.hashFunction
		);
	}

	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	get #instance() {
		const processResource = (
			loaderContext: LoaderContext,
			resourcePath: string,
			callback: any
		) => {
			const resource = loaderContext.resource;
			const scheme = getScheme(resource);
			this.compilation
				.currentNormalModuleHooks()
				.readResource.for(scheme)
				.callAsync(loaderContext, (err: any, result: LoaderResult) => {
					if (err) return callback(err);
					if (typeof result !== "string" && !result) {
						return callback(new Error(`Unhandled ${scheme} resource`));
					}
					return callback(null, result);
				});
		};
		const options = getRawOptions(this.options, this, processResource);

		this.#_instance =
			this.#_instance ??
			new binding.Rspack(
				options,
				{
					beforeCompile: this.#beforeCompile.bind(this),
					make: this.#make.bind(this),
					emit: this.#emit.bind(this),
					afterEmit: this.#afterEmit.bind(this),
					processAssetsStageAdditional: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
					),
					processAssetsStagePreProcess: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS
					),
					processAssetsStageAdditions: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_ADDITIONS
					),
					processAssetsStageNone: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_NONE
					),
					processAssetsStageOptimizeInline: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE
					),
					processAssetsStageSummarize: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
					),
					processAssetsStageOptimizeHash: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH
					),
					processAssetsStageReport: this.#processAssets.bind(
						this,
						Compilation.PROCESS_ASSETS_STAGE_REPORT
					),
					// `Compilation` should be created with hook `thisCompilation`, and here is the reason:
					// We know that the hook `thisCompilation` will not be called from a child compiler(it doesn't matter whether the child compiler is created on the Rust or the Node side).
					// See webpack's API: https://webpack.js.org/api/compiler-hooks/#thiscompilation
					// So it is safe to create a new compilation here.
					thisCompilation: this.#newCompilation.bind(this),
					// The hook `Compilation` should be called whenever it's a call from the child compiler or normal compiler and
					// still it does not matter where the child compiler is created(Rust or Node) as calling the hook `compilation` is a required task.
					// No matter how it will be implemented, it will be copied to the child compiler.
					compilation: this.#compilation.bind(this),
					optimizeModules: this.#optimize_modules.bind(this),
					optimizeChunkModule: this.#optimize_chunk_modules.bind(this),
					finishModules: this.#finish_modules.bind(this),
					normalModuleFactoryResolveForScheme:
						this.#normalModuleFactoryResolveForScheme.bind(this),
					chunkAsset: this.#chunkAsset.bind(this),
					beforeResolve: this.#beforeResolve.bind(this),
					contextModuleBeforeResolve:
						this.#contextModuleBeforeResolve.bind(this)
				},
				createThreadsafeNodeFSFromRaw(this.outputFileSystem),
				loaderContext => runLoader(loaderContext, this)
			);

		return this.#_instance;
	}
	createChildCompiler(
		compilation: Compilation,
		compilerName: string,
		compilerIndex: number,
		outputOptions: OutputNormalized,
		plugins: RspackPluginInstance[]
	) {
		const childCompiler = new Compiler(this.context, {
			...this.options,
			output: {
				...this.options.output,
				...outputOptions
			},
			// TODO: check why we need to have builtins otherwise this.#instance will fail to initialize Rspack
			builtins: {
				...this.options.builtins,
				html: undefined
			}
		});
		childCompiler.name = compilerName;
		childCompiler.outputPath = this.outputPath;
		childCompiler.inputFileSystem = this.inputFileSystem;
		// childCompiler.outputFileSystem = null;
		childCompiler.resolverFactory = this.resolverFactory;
		childCompiler.modifiedFiles = this.modifiedFiles;
		childCompiler.removedFiles = this.removedFiles;
		// childCompiler.fileTimestamps = this.fileTimestamps;
		// childCompiler.contextTimestamps = this.contextTimestamps;
		// childCompiler.fsStartTime = this.fsStartTime;
		childCompiler.cache = this.cache;
		childCompiler.compilerPath = `${this.compilerPath}${compilerName}|${compilerIndex}|`;
		// childCompiler._backCompat = this._backCompat;

		const relativeCompilerName = makePathsRelative(
			this.context,
			compilerName,
			this.root
		);
		// if (!this.records[relativeCompilerName]) {
		// 	this.records[relativeCompilerName] = [];
		// }
		// if (this.records[relativeCompilerName][compilerIndex]) {
		// 	childCompiler.records = this.records[relativeCompilerName][compilerIndex];
		// } else {
		// 	this.records[relativeCompilerName].push((childCompiler.records = {}));
		// }

		childCompiler.parentCompilation = compilation;
		childCompiler.root = this.root;
		if (Array.isArray(plugins)) {
			for (const plugin of plugins) {
				plugin.apply(childCompiler);
			}
		}
		for (const name in this.hooks) {
			if (
				![
					"make",
					"compile",
					"emit",
					"afterEmit",
					"invalid",
					"done",
					"thisCompilation"
				].includes(name)
			) {
				//@ts-ignore
				if (childCompiler.hooks[name]) {
					//@ts-ignore
					childCompiler.hooks[name].taps = this.hooks[name].taps.slice();
				}
			}
		}

		// compilation.hooks.childCompiler.call(
		// 	childCompiler,
		// 	compilerName,
		// 	compilerIndex
		// );

		return childCompiler;
	}

	runAsChild(callback: any) {
		const startTime = Date.now();

		const finalCallback = (
			err: Error | null,
			entries?: any,
			compilation?: Compilation
		) => {
			try {
				callback(err, entries, compilation);
			} catch (e) {
				const err = new Error(`compiler.runAsChild callback error: ${e}`);
				// err.details = e.stack;
				this.parentCompilation!.errors.push(err);
				// TODO: remove once this works
				console.log(e);
			}
		};

		this.run((err, stats) => {
			if (err) return finalCallback(err);
			const compilation: Compilation = stats!.compilation;

			this.parentCompilation!.children.push(compilation);
			for (const { name, source, info } of compilation.getAssets()) {
				this.parentCompilation!.emitAsset(name, source, info);
			}

			const entries = [];
			for (const ep of compilation.entrypoints.values()) {
				entries.push(...ep.getFiles());
			}

			// compilation.startTime = startTime;
			// compilation.endTime = Date.now();

			return finalCallback(null, entries, compilation);
		});
	}
	isChild() {
		const isRoot = this.root === this;
		return !isRoot;
	}
	getInfrastructureLogger(name: string | Function) {
		if (!name) {
			throw new TypeError(
				"Compiler.getInfrastructureLogger(name) called without a name"
			);
		}
		return new Logger(
			(type, args) => {
				if (typeof name === "function") {
					name = name();
					if (!name) {
						throw new TypeError(
							"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
						);
					}
				} else {
					if (
						// @ts-expect-error
						this.hooks.infrastructureLog.call(name, type, args) === undefined
					) {
						if (this.infrastructureLogger !== undefined) {
							this.infrastructureLogger(name, type, args);
						}
					}
				}
			},
			(childName): any => {
				if (typeof name === "function") {
					if (typeof childName === "function") {
						// @ts-expect-error
						return this.getInfrastructureLogger(_ => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
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
						return this.getInfrastructureLogger(() => {
							if (typeof name === "function") {
								name = name();
								if (!name) {
									throw new TypeError(
										"Compiler.getInfrastructureLogger(name) called with a function not returning a name"
									);
								}
							}
							return `${name}/${childName}`;
						});
					}
				} else {
					if (typeof childName === "function") {
						return this.getInfrastructureLogger(() => {
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
						return this.getInfrastructureLogger(`${name}/${childName}`);
					}
				}
			}
		);
	}

	#updateDisabledHooks() {
		const disabledHooks = [];
		const hookMap = {
			make: this.hooks.make,
			beforeCompile: this.hooks.beforeCompile,
			emit: this.hooks.emit,
			afterEmit: this.hooks.afterEmit,
			processAssetsStageAdditional:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
				),
			processAssetsStagePreProcess:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS
				),
			processAssetsStageNone:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_NONE
				),
			processAssetsStageOptimizeInline:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE
				),
			processAssetsStageSummarize:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
				),
			processAssetsStageReport:
				this.compilation.__internal_getProcessAssetsHookByStage(
					Compilation.PROCESS_ASSETS_STAGE_REPORT
				),
			compilation: this.hooks.compilation,
			optimizeChunkModules: this.compilation.hooks.optimizeChunkModules,
			finishModules: this.compilation.hooks.finishModules,
			optimizeModules: this.compilation.hooks.optimizeModules
		};
		for (const [name, hook] of Object.entries(hookMap)) {
			if (hook.taps.length === 0) {
				disabledHooks.push(name);
			}
		}

		// disabledHooks is in order
		if (this.#disabledHooks.join() !== disabledHooks.join()) {
			this.#instance.unsafe_set_disabled_hooks(disabledHooks);
			this.#disabledHooks = disabledHooks;
		}
	}

	async #beforeCompile() {
		await this.hooks.beforeCompile.promise();
		// compilation is not created yet, so this will fail
		// this.#updateDisabledHooks();
	}

	async #finishModules() {
		await this.compilation.hooks.finishModules.promise(
			this.compilation.getModules()
		);
		this.#updateDisabledHooks();
	}
	async #processAssets(stage: number) {
		await this.compilation
			.__internal_getProcessAssetsHookByStage(stage)
			.promise(this.compilation.assets);
		this.#updateDisabledHooks();
	}

	async #beforeResolve(resourceData: binding.BeforeResolveData) {
		let res =
			await this.compilation.normalModuleFactory?.hooks.beforeResolve.promise(
				resourceData
			);

		this.#updateDisabledHooks();
		return res;
	}
	async #contextModuleBeforeResolve(resourceData: binding.BeforeResolveData) {
		let res =
			await this.compilation.contextModuleFactory?.hooks.beforeResolve.promise(
				resourceData
			);

		this.#updateDisabledHooks();
		return res;
	}

	async #normalModuleFactoryResolveForScheme(
		resourceData: binding.SchemeAndJsResourceData
	) {
		await this.compilation.normalModuleFactory?.hooks.resolveForScheme
			.for(resourceData.scheme)
			.promise(resourceData.resourceData);
		return resourceData.resourceData;
	}

	async #optimize_chunk_modules() {
		await this.compilation.hooks.optimizeChunkModules.promise(
			this.compilation.getChunks(),
			this.compilation.getModules()
		);
		this.#updateDisabledHooks();
	}
	async #optimize_modules() {
		await this.compilation.hooks.optimizeModules.promise(
			this.compilation.getModules()
		);
		this.#updateDisabledHooks();
	}

	#chunkAsset(assetArg: binding.JsChunkAssetArgs) {
		this.compilation.hooks.chunkAsset.call(assetArg.chunk, assetArg.filename);
		this.#updateDisabledHooks();
	}

	async #finish_modules() {
		await this.compilation.hooks.finishModules.promise(
			this.compilation.getModules()
		);
		this.#updateDisabledHooks();
	}

	async #make() {
		await this.hooks.make.promise(this.compilation);
		this.#updateDisabledHooks();
	}
	async #emit() {
		await this.hooks.emit.promise(this.compilation);
		this.#updateDisabledHooks();
	}

	async #afterEmit() {
		await this.hooks.afterEmit.promise(this.compilation);
		this.#updateDisabledHooks();
	}

	#compilation(native: binding.JsCompilation) {
		// TODO: implement this based on the child compiler impl.
		this.hooks.compilation.call(this.compilation, {
			normalModuleFactory: this.compilation.normalModuleFactory!
		});

		this.#updateDisabledHooks();
	}

	#newCompilation(native: binding.JsCompilation) {
		const compilation = new Compilation(this, native);
		compilation.name = this.name;
		this.compilation = compilation;
		// reset normalModuleFactory when create new compilation
		let normalModuleFactory = new NormalModuleFactory();
		let contextModuleFactory = new ContextModuleFactory();
		this.compilation.normalModuleFactory = normalModuleFactory;
		this.hooks.normalModuleFactory.call(normalModuleFactory);
		this.compilation.contextModuleFactory = contextModuleFactory;
		this.hooks.contextModuleFactory.call(normalModuleFactory);
		this.hooks.thisCompilation.call(this.compilation, {
			normalModuleFactory: normalModuleFactory
		});
		this.#updateDisabledHooks();
	}

	run(callback: Callback<Error, Stats>) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}
		const startTime = Date.now();
		this.running = true;
		const doRun = () => {
			// @ts-expect-error
			const finalCallback = (err, stats?) => {
				this.running = false;
				if (err) {
					this.hooks.failed.call(err);
				}
				if (callback) {
					callback(err, stats);
				}
				this.hooks.afterDone.call(stats);
			};
			this.hooks.beforeRun.callAsync(this, err => {
				if (err) {
					return finalCallback(err);
				}
				this.hooks.run.callAsync(this, err => {
					if (err) {
						return finalCallback(err);
					}

					this.build(err => {
						if (err) {
							return finalCallback(err);
						}
						this.compilation.startTime = startTime;
						this.compilation.endTime = Date.now();
						const stats = new Stats(this.compilation);
						this.hooks.done.callAsync(stats, err => {
							if (err) {
								return finalCallback(err);
							} else {
								return finalCallback(null, stats);
							}
						});
					});
				});
			});
		};
		doRun();
	}
	// Safety: This method is only valid to call if the previous build task is finished, or there will be data races.
	build(cb: (error?: Error) => void) {
		const unsafe_build = this.#instance.unsafe_build;
		const build_cb = unsafe_build.bind(this.#instance) as typeof unsafe_build;
		build_cb(err => {
			if (err) {
				cb(err);
			} else {
				cb(undefined);
			}
		});
	}
	// Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	rebuild(
		modifiedFiles?: ReadonlySet<string>,
		removedFiles?: ReadonlySet<string>,
		cb?: (error?: Error) => void
	) {
		const unsafe_rebuild = this.#instance.unsafe_rebuild;
		const rebuild_cb = unsafe_rebuild.bind(
			this.#instance
		) as typeof unsafe_rebuild;
		rebuild_cb([...(modifiedFiles ?? [])], [...(removedFiles ?? [])], err => {
			if (err) {
				cb && cb(err);
			} else {
				cb && cb(undefined);
			}
		});
	}

	watch(watchOptions: WatchOptions, handler: Callback<Error, Stats>): Watching {
		if (this.running) {
			// @ts-expect-error
			return handler(new ConcurrentCompilationError());
		}
		this.running = true;
		this.watchMode = true;
		// @ts-expect-error
		this.watching = new Watching(this, watchOptions, handler);
		return this.watching;
	}

	purgeInputFileSystem() {
		if (this.inputFileSystem && this.inputFileSystem.purge) {
			this.inputFileSystem.purge();
		}
	}

	close(callback: () => void) {
		// WARNING: Arbitrarily dropping the instance is not safe, as it may still be in use by the background thread.
		// A hint is necessary for the compiler to know when it is safe to drop the instance.
		// For example: register a callback to the background thread, and drop the instance when the callback is called (calling the `close` method queues the signal)
		// See: https://github.com/webpack/webpack/blob/4ba225225b1348c8776ca5b5fe53468519413bc0/lib/Compiler.js#L1218
		if (!this.running) {
			// Manually drop the instance.
			// this.#_instance = undefined;
		}

		if (this.watching) {
			// When there is still an active watching, close this first
			this.watching.close(() => {
				this.close(callback);
			});
			return;
		}
		callback();
	}

	getAsset(name: string) {
		let source = this.compilation.__internal__getAssetSource(name);
		if (!source) {
			return null;
		}
		return source.buffer();
	}
}

export { Compiler };
