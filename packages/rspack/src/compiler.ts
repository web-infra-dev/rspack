import path from "path";
import fs from "fs";
import * as tapable from "tapable";
import { SyncHook, SyncBailHook, Callback } from "tapable";
import asyncLib from "neo-async";
import type { WatchOptions } from "watchpack";
import Watching from "./watching";
import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import { RspackOptionsNormalized } from "./config";
import { Stats } from "./stats";
import { Compilation } from "./compilation";
import ResolverFactory from "./ResolverFactory";
import { WatchFileSystem } from "./util/fs";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import { getRawOptions } from "./config/adapter";
import {
	createThreadsafeNodeFSFromRaw,
	ThreadsafeWritableNodeFS
} from "./fileSystem";

class EntryPlugin {
	apply() {}
}
class HotModuleReplacementPlugin {
	apply() {}
}

class Compiler {
	// @ts-expect-error
	#_instance: binding.Rspack;

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
		thisCompilation: tapable.SyncHook<[Compilation]>;
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
			thisCompilation: new tapable.SyncHook<
				[
					Compilation
					// CompilationParams
				]
			>([
				"compilation"
				// "params"
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
	}
	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	get #instance() {
		const options = getRawOptions(this.options, this);
		this.#_instance =
			this.#_instance ||
			new binding.Rspack(
				options,
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
					optimizeChunkModule: this.#optimize_chunk_modules.bind(this)
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

	async #processAssets(stage: number) {
		await this.compilation
			.__internal_getProcessAssetsHookByStage(stage)
			.promise(this.compilation.assets);
	}

	async #optimize_chunk_modules() {
		await this.compilation.hooks.optimizeChunkModules.promise(
			this.compilation.getModules()
		);
	}
	async #make() {
		await this.hooks.make.promise(this.compilation);
	}
	async #emit() {
		await this.hooks.emit.promise(this.compilation);
	}

	async #afterEmit() {
		await this.hooks.afterEmit.promise(this.compilation);
	}

	#compilation(native: binding.JsCompilation) {
		// TODO: implement this based on the child compiler impl.
		this.hooks.compilation.call(this.compilation);
	}

	#newCompilation(native: binding.JsCompilation) {
		const compilation = new Compilation(this, native);
		compilation.name = this.name;
		this.compilation = compilation;
		this.hooks.thisCompilation.call(this.compilation);
	}

	run(callback: Callback<Error, Stats>) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}
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
				// @ts-expect-error
				cb(null);
			}
		});
	}
	// Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	rebuild(
		modifiedFiles: ReadonlySet<string> | undefined,
		removedFiles: ReadonlySet<string> | undefined,
		cb: ((error?: Error) => void) | undefined
	) {
		const unsafe_rebuild = this.#instance.unsafe_rebuild;
		const rebuild_cb = unsafe_rebuild.bind(
			this.#instance
		) as typeof unsafe_rebuild;
		rebuild_cb([...(modifiedFiles ?? [])], [...(removedFiles ?? [])], err => {
			if (err) {
				// @ts-expect-error
				cb(err);
			} else {
				// @ts-expect-error
				cb(null);
			}
		});
	}

	watch(
		watchOptions: WatchOptions,
		handler: (error: Error, stats: Stats) => void
	): Watching {
		if (this.running) {
			return handler(
				new ConcurrentCompilationError(),
				// @ts-expect-error
				null
			) as unknown as Watching;
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

	/**
	 * @todo
	 */
	// @ts-expect-error
	close(callback) {
		// @ts-expect-error
		this.#_instance = null;
		if (this.watching) {
			// When there is still an active watching, close this first
			this.watching.close(() => {
				this.close(callback);
			});
			return;
		}
		callback();
	}
	// @ts-expect-error
	emitAssets(compilation: Compilation, callback) {
		const outputPath = compilation.getPath(this.outputPath, {});
		fs.mkdirSync(outputPath, { recursive: true });
		const assets = compilation.getAssets();
		asyncLib.forEachLimit(
			assets,
			15,
			({ name: file, source, info }, callback) => {
				let targetFile = file;
				const absPath = path.resolve(outputPath, targetFile);
				const getContent = () => {
					if (typeof source.buffer === "function") {
						return source.buffer();
					} else {
						const bufferOrString = source.source();
						if (Buffer.isBuffer(bufferOrString)) {
							return bufferOrString;
						} else {
							return Buffer.from(bufferOrString as string, "utf-8");
						}
					}
				};
				// @ts-expect-error
				const doWrite = content => {
					this.outputFileSystem.writeFile(absPath, content, callback);
				};
				let content = getContent();
				doWrite(content);
			}
		);
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
