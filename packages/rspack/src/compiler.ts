import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import { resolveWatchOption } from "./config/watch";
import type { Watch, ResolvedWatch } from "./config/watch";
import * as tapable from "tapable";

import { SyncHook, SyncBailHook, Callback } from "tapable";
import util from "util";
import fs from "fs";
import asyncLib from "neo-async";
import path from "path";
import {
	RspackOptions,
	RspackOptionsNormalized,
	getNormalizedRspackOptions
} from "./config";

import { Stats } from "./stats";
import { Asset, Compilation } from "./compilation";
import { mkdir } from "fs";

export type EmitAssetCallback = (options: {
	filename: string;
	asset: Asset;
}) => void;
class EntryPlugin {
	apply() {}
}
class HotModuleReplacementPlugin {
	apply() {}
}
type CompilationParams = Record<string, any>;
class Compiler {
	webpack: any;
	compilation: Compilation;
	infrastructureLogger: any;
	outputPath: string;
	name: string;
	inputFileSystem: any;
	outputFileSystem: any;
	hooks: {
		done: tapable.AsyncSeriesHook<Stats>;
		afterDone: tapable.SyncHook<Stats>;
		compilation: tapable.SyncHook<Compilation>;
		thisCompilation: tapable.SyncHook<[Compilation, CompilationParams]>;
		invalid: tapable.SyncHook<[string | null, number]>;
		compile: tapable.SyncHook<[any]>;
		initialize: tapable.SyncHook<[]>;
		infrastructureLog: tapable.SyncBailHook<[string, string, any[]], true>;
		beforeRun: tapable.AsyncSeriesHook<[Compiler]>;
		run: tapable.AsyncSeriesHook<[Compiler]>;
		failed: tapable.SyncHook<[Error]>;
	};
	options: RspackOptionsNormalized;

	constructor(context: string, options: RspackOptionsNormalized) {
		this.options = options;
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin // modernjs/server will auto inject this this plugin not set
		};
		this.hooks = {
			initialize: new SyncHook([]),
			done: new tapable.AsyncSeriesHook<Stats>(["stats"]),
			afterDone: new tapable.SyncHook<Stats>(["stats"]),
			beforeRun: new tapable.AsyncSeriesHook(["compiler"]),
			run: new tapable.AsyncSeriesHook(["compiler"]),
			thisCompilation: new tapable.SyncHook<[Compilation, CompilationParams]>([
				"compilation",
				"params"
			]),
			compilation: new tapable.SyncHook<Compilation>(["compilation"]),
			invalid: new SyncHook(["filename", "changeTime"]),
			compile: new SyncHook(["params"]),
			infrastructureLog: new SyncBailHook(["origin", "type", "args"]),
			failed: new SyncHook(["error"])
		};
	}
	/**
	 * Lazy initialize instance so it could access the changed options
	 */
	get #instance() {
		// @ts-ignored
		return new binding.Rspack(this.options, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#processAssets.bind(this)
		});
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
	#done(statsJson: binding.StatsCompilation) {}
	#processAssets(value: string, emitAsset: any) {
		return this.compilation.processAssets(value, emitAsset);
	}
	#newCompilation() {
		const compilation = new Compilation(this.options);
		this.compilation = compilation;
		this.hooks.compilation.call(compilation);
		return compilation;
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

					this.build((err, raw_stats) => {
						if (err) {
							return finalCallback(err);
						}
						const stats = new Stats(this.compilation, raw_stats);
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
	build(cb: Callback<Error, any>) {
		const compilation = this.#newCompilation();
		const build_cb = util.callbackify(
			this.#instance.build.bind(this.#instance)
		) as (cb: Callback<Error, any>) => void;
		build_cb((err, stats) => {
			if (err) {
				cb(err);
			} else {
				cb(null, stats);
			}
		});
	}
	rebuild(changedFiles: string[], cb) {
		const rebuild_cb = util.callbackify(
			this.#instance.rebuild.bind(this.#instance, changedFiles, [])
		) as (cb: Callback<Error, any>) => void;
		rebuild_cb((err, stats) => {
			if (err) {
				cb(err);
			} else {
				cb(null, stats);
			}
		});
	}

	async watch(watchOptions?: Watch): Promise<Watching> {
		const options = resolveWatchOption(watchOptions);

		const watcher = (await import("chokidar")).default.watch(
			this.options.context,
			{
				ignoreInitial: true,
				...options
			}
		);
		let stats = await util.promisify(this.build.bind(this))();

		// TODO: should use aggregated
		watcher.on("change", async path => {
			// TODO: only build because we lack the snapshot info of file.
			// TODO: it means there a lot of things to do....
			const begin = Date.now();
			console.log("hit change and start to build");
			const diffStats = await util.promisify(this.rebuild.bind(this))([path]);
			console.log(`build success, time cost ${(Date.now() - begin) / 1000}ms`);
		});

		return {
			async close() {
				await watcher.close();
			}
		};
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
		callback();
	}
	emitAssets(compilation: Compilation, callback) {
		const outputPath = compilation.getPath(this.outputPath, {});
		fs.mkdirSync(outputPath, { recursive: true });
		const assets = compilation.getAssets();
		compilation.assets = { ...compilation.assets };
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
}

export interface Watching {
	close(): Promise<void>;
}

export { Compiler };
