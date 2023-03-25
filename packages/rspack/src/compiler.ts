/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Compiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import fs from "fs";
import * as tapable from "tapable";
import { SyncHook, SyncBailHook, Callback } from "tapable";
import type { WatchOptions } from "watchpack";
import Watching from "./watching";
import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import { RspackOptionsNormalized } from "./config";
import { Stats } from "./stats";
import { Compilation, CompilationParams } from "./compilation";
import ResolverFactory from "./ResolverFactory";
import { WatchFileSystem } from "./util/fs";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import { getRawOptions } from "./config/adapter";
import { createThreadsafeNodeFSFromRaw } from "./fileSystem";
import { NormalModuleFactory } from "./normalModuleFactory";

class EntryPlugin {
	apply() {}
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
	// @ts-expect-error
	watchFileSystem: WatchFileSystem;
	intermediateFileSystem: any;
	// @ts-expect-error
	watchMode: boolean;
	context: string;
	modifiedFiles?: ReadonlySet<string>;
	removedFiles?: ReadonlySet<string>;
	hooks: {
		done: tapable.AsyncSeriesHook<Stats>;
		afterDone: tapable.SyncHook<Stats>;
		// TODO: CompilationParams
		compilation: tapable.SyncHook<Compilation>;
		// TODO: CompilationParams
		thisCompilation: tapable.SyncHook<[Compilation, CompilationParams]>;
		invalid: tapable.SyncHook<[string | null, number]>;
		compile: tapable.SyncHook<[any]>;
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
	};
	options: RspackOptionsNormalized;
	#disabledHooks: string[];

	constructor(context: string, options: RspackOptionsNormalized) {
		this.outputFileSystem = fs;
		this.options = options;
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin, // modernjs/server will auto inject this this plugin not set
			get sources(): typeof import("webpack-sources") {
				return require("webpack-sources");
			},
			Compilation,
			get version() {
				return "5.75.0"; // this is a hack to be compatible with plugin which detect webpack's version
			},
			get rspackVersion() {
				return require("../package.json").version;
			}
		};
		this.root = this;
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
			compilation: new tapable.SyncHook<Compilation>(["compilation"]),
			invalid: new SyncHook(["filename", "changeTime"]),
			compile: new SyncHook(["params"]),
			infrastructureLog: new SyncBailHook(["origin", "type", "args"]),
			failed: new SyncHook(["error"]),
			watchRun: new tapable.AsyncSeriesHook(["compiler"]),
			watchClose: new tapable.SyncHook([]),
			environment: new tapable.SyncHook([]),
			afterEnvironment: new tapable.SyncHook([]),
			afterPlugins: new tapable.SyncHook(["compiler"]),
			afterResolvers: new tapable.SyncHook(["compiler"]),
			make: new tapable.AsyncParallelHook(["compilation"])
		};
		this.modifiedFiles = undefined;
		this.removedFiles = undefined;
		this.#disabledHooks = [];
	}

	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	get #instance() {
		this.#_instance =
			this.#_instance ??
			new binding.Rspack(
				getRawOptions(this.options, this),
				{
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
					optimizeChunkModule: this.#optimize_chunk_modules.bind(this),
					finishModules: this.#finish_modules.bind(this),
					normalModuleFactoryResolveForScheme:
						this.#normalModuleFactoryResolveForScheme.bind(this)
				},
				createThreadsafeNodeFSFromRaw(this.outputFileSystem)
			);

		return this.#_instance;
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
			finishModules: this.compilation.hooks.finishModules
			// normalModuleFactoryResolveForScheme: this.#
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

	async #processAssets(stage: number) {
		await this.compilation
			.__internal_getProcessAssetsHookByStage(stage)
			.promise(this.compilation.assets);
		this.#updateDisabledHooks();
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
		this.hooks.compilation.call(this.compilation);

		this.#updateDisabledHooks();
	}

	#newCompilation(native: binding.JsCompilation) {
		const compilation = new Compilation(this, native);
		compilation.name = this.name;
		this.compilation = compilation;
		// reset normalModuleFactory when create new compilation
		let normalModuleFactory = new NormalModuleFactory();
		this.compilation.normalModuleFactory = normalModuleFactory;
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

	watch(
		watchOptions: WatchOptions,
		handler: (error: Error, stats?: Stats) => Watching
	): Watching {
		if (this.running) {
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
