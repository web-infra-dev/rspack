import path from "path";
import fs from "fs";

import * as tapable from "tapable";
import { SyncHook, SyncBailHook, Callback } from "tapable";
import asyncLib from "neo-async";

import Watching from "./watching";
import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import type { WatchOptions } from "./config/watch";
import { RspackOptionsNormalized } from "./config";
import { Stats } from "./stats";
import { Compilation } from "./compilation";
import ResolverFactory from "./ResolverFactory";
import { WatchFileSystem } from "./util/fs";

class EntryPlugin {
	apply() {}
}
class HotModuleReplacementPlugin {
	apply() {}
}
type CompilationParams = Record<string, any>;
class Compiler {
	#_instance: binding.Rspack;

	webpack: any;
	compilation: Compilation;
	root: Compiler;
	resolverFactory: ResolverFactory;
	infrastructureLogger: any;
	watching: Watching;
	outputPath: string;
	name: string;
	inputFileSystem: any;
	outputFileSystem: any;
	watchFileSystem: WatchFileSystem;
	intermediateFileSystem: any;
	watchMode: boolean;
	context: string;
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
		failed: tapable.SyncHook<[Error]>;
		watchRun: tapable.AsyncSeriesHook<[Compiler]>;
	};
	options: RspackOptionsNormalized;

	constructor(context: string, options: RspackOptionsNormalized) {
		this.options = options;
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin, // modernjs/server will auto inject this this plugin not set
			get sources(): typeof import("webpack-sources") {
				return require("webpack-sources");
			}
		};
		this.root = this;
		this.context = context;
		this.resolverFactory = new ResolverFactory();
		this.hooks = {
			initialize: new SyncHook([]),
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new tapable.SyncHook<Stats>(["stats"]),
			beforeRun: new tapable.AsyncSeriesHook(["compiler"]),
			run: new tapable.AsyncSeriesHook(["compiler"]),
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
			watchRun: new tapable.AsyncSeriesHook(["compiler"])
		};
	}
	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	get #instance() {
		const options: binding.RawOptions = this.options;

		this.#_instance =
			this.#_instance ||
			new binding.Rspack(options, {
				// TODO: add it back
				// done: this.#done.bind(this),
				processAssets: this.#processAssets.bind(this),
				// `Compilation` should be created with hook `thisCompilation`, and here is the reason:
				// We know that the hook `thisCompilation` will not be called from a child compiler(it doesn't matter whether the child compiler is created on the Rust or the Node side).
				// See webpack's API: https://webpack.js.org/api/compiler-hooks/#thiscompilation
				// So it is safe to create a new compilation here.
				thisCompilation: this.#newCompilation.bind(this),
				// The hook `Compilation` should be called whenever it's a call from the child compiler or normal compiler and
				// still it does not matter where the child compiler is created(Rust or Node) as calling the hook `compilation` is a required task.
				// No matter how it will be implemented, it will be copied to the child compiler.
				compilation: this.#compilation.bind(this)
			});

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
						this.hooks.infrastructureLog.call(name, type, args) === undefined
					) {
						if (this.infrastructureLogger !== undefined) {
							this.infrastructureLogger(name, type, args);
						}
					}
				}
			},
			childName => {
				if (typeof name === "function") {
					if (typeof childName === "function") {
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
	/**
	 * @todo remove it in the future
	 * @param err
	 * @param value
	 * @returns
	 */
	#done(statsJson: binding.JsStatsCompilation) {}

	async #processAssets() {
		await this.compilation.hooks.processAssets.promise(
			new Proxy(
				{},
				{
					get: (_, property) => {
						return this.compilation.__internal__getAssetSource(
							property as string
						);
					},
					has: (_, property) => {
						return this.compilation.__internal__hasAsset(property as string);
					},
					ownKeys: _ => {
						return this.compilation.__internal__getAssetFilenames();
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
			)
		);
	}

	#compilation(native: binding.JsCompilation) {
		// TODO: implement this based on the child compiler impl.
		this.hooks.compilation.call(this.compilation);
	}

	#newCompilation(native: binding.JsCompilation) {
		const compilation = new Compilation(this, native);
		this.compilation = compilation;
		this.hooks.thisCompilation.call(this.compilation);
	}

	run(callback) {
		const doRun = () => {
			const finalCallback = (err, stats?) => {
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

					this.build((err) => {
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
		build_cb((err) => {
			if (err) {
				cb(err);
			} else {
				cb(null);
			}
		});
	}
	// Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	rebuild(
		modifiedFiles: ReadonlySet<string>,
		removedFiles: ReadonlySet<string>,
		cb: (error?: Error) => void
	) {
		const unsafe_rebuild = this.#instance.unsafe_rebuild;
		const rebuild_cb = unsafe_rebuild.bind(this.#instance) as typeof unsafe_rebuild;
		rebuild_cb([...modifiedFiles], [...removedFiles], (err) => {
			if (err) {
				cb(err);
			} else {
				cb(null);
			}
		});
	}

	watch(
		watchOptions: WatchOptions,
		handler: (error: Error, stats: Stats) => void
	): Watching {
		this.watchMode = true;
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
	close(callback) {
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
