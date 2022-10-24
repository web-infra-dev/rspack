import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import { resolveWatchOption } from "./config/watch";
import type { Watch, ResolvedWatch } from "./config/watch";
import * as tapable from "tapable";
import { SyncHook, SyncBailHook } from "tapable";
import util from "util";
import {
	RspackOptions,
	RspackOptionsNormalized,
	Asset,
	getNormalizedRspackOptions
} from "./config";

import { Stats } from "./stats";
import { Compilation } from "./compilation";

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
	#plugins: RspackOptions["plugins"];
	#instance: binding.Rspack;
	compilation: Compilation;
	infrastructureLogger: any;
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
	};
	options: RspackOptionsNormalized;

	constructor(context: string, options: RspackOptionsNormalized) {
		this.options = options;
		// to workaround some plugin access webpack, we may change dev-server to avoid this hack in the future
		this.webpack = {
			EntryPlugin, // modernjs/server use this to inject dev-client
			HotModuleReplacementPlugin // modernjs/server will auto inject this this plugin not set
		};
		// @ts-ignored
		this.#instance = new binding.Rspack(this.options, {
			doneCallback: this.#done.bind(this),
			processAssetsCallback: this.#processAssets.bind(this)
		});
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
			infrastructureLog: new SyncBailHook(["origin", "type", "args"])
		};
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
	async #done(statsJson: binding.StatsCompilation) {
		const stats = new Stats(this.compilation, statsJson);
		await this.hooks.done.promise(stats);
	}
	async #processAssets(value: string, emitAsset: any) {
		return this.compilation.processAssets(value, emitAsset);
	}
	#newCompilation() {
		const compilation = new Compilation();
		this.compilation = compilation;
		this.hooks.compilation.call(compilation);
		return compilation;
	}
	async run(callback) {
		const doRun = async () => {
			await this.hooks.beforeRun.promise(this);
			await this.hooks.run.promise(this);
			const raw_stats = await this.build();
			const stats = new Stats(this.compilation, raw_stats);
			await this.hooks.done.promise(stats);
			return stats;
		};
		if (callback) {
			util.callbackify(doRun)(callback);
		} else {
			return doRun();
		}
	}
	async build() {
		const compilation = this.#newCompilation();
		const stats = await this.#instance.build();
		return stats;
	}
	async rebuild(changedFiles: string[]) {
		const stats = await this.#instance.rebuild(changedFiles, []);
		return stats.inner;
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
		let stats = await this.build();

		// TODO: should use aggregated
		watcher.on("change", async path => {
			// TODO: only build because we lack the snapshot info of file.
			// TODO: it means there a lot of things to do....
			const begin = Date.now();
			console.log("hit change and start to build");
			const diffStats = await this.rebuild([path]);
			console.log("build success, time cost", Date.now() - begin);
		});

		return {
			async close() {
				await watcher.close();
			}
		};
	}
	/**
	 * @todo
	 */
	close(callback) {
		callback();
	}
}

export interface Watching {
	close(): Promise<void>;
}

export { Compiler };
