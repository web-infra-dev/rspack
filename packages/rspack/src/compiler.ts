import * as binding from "@rspack/binding";
import { Logger } from "./logging/Logger";
import { resolveWatchOption } from "./config/watch";
import type { Watch } from "./config/watch";
import * as tapable from "tapable";
import { SyncHook, SyncBailHook, Callback } from "tapable";
import util from "util";
import fs from "fs";
import asyncLib from "neo-async";
import path from "path";
import { RspackOptionsNormalized } from "./config";
import { Stats } from "./stats";
import { Asset, Compilation } from "./compilation";

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
		this._instance =
			// @ts-ignored
			this._instance ||
			// @ts-ignored
			new binding.Rspack(this.options, {
				doneCallback: this.#done.bind(this),
				processAssetsCallback: this.#processAssets.bind(this)
			});
		// @ts-ignored
		return this._instance;
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

					this.unsafe_build((err, raw_stats) => {
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
	// Safety: This method is only valid to call if the previous build task is finished, or there will be data races.
	unsafe_build(cb: Callback<Error, binding.StatsCompilation>) {
		const compilation = this.#newCompilation();
		const build_cb = this.#instance.unsafe_build.bind(this.#instance) as (
			cb: Callback<Error, binding.StatsCompilation>
		) => void;
		build_cb((err, stats) => {
			if (err) {
				cb(err);
			} else {
				cb(null, stats);
			}
		});
	}
	// Safety: This method is only valid to call if the previous rebuild task is finished, or there will be data races.
	unsafe_rebuild(
		changedFiles: string[],
		cb: (error?: Error, stats?: binding.DiffStat) => void
	) {
		const rebuild_cb = this.#instance.unsafe_rebuild.bind(this.#instance) as (
			changed: string[],
			removed: string[],
			cb: Callback<Error, any>
		) => void;
		rebuild_cb(changedFiles, [], (err, stats) => {
			if (err) {
				cb(err);
			} else {
				cb(null, stats);
			}
		});
	}

	// TODO: use ws to send message to client temporary.
	// TODO: we should use `Stats` which got from `hooks.done`
	// TODO: in `dev-server`
	async watch(watchOptions?: Watch, ws?: any): Promise<Watching> {
		const options = resolveWatchOption(watchOptions);

		const watcher = (await import("chokidar")).default.watch(
			this.options.context,
			{
				ignoreInitial: true,
				...options
			}
		);
		const begin = Date.now();
		let stats = await util.promisify(this.unsafe_build.bind(this))();
		console.log("build success, time cost", Date.now() - begin, "ms");

		let pendingChangedFilepaths = new Set<string>();
		let isBuildFinished = true;

		// TODO: should use aggregated
		watcher.on("change", async changedFilepath => {
			// TODO: only build because we lack the snapshot info of file.
			// TODO: it means there a lot of things to do....

			// store the changed file path, it may or may not be consumed right now
			if (!isBuildFinished) {
				pendingChangedFilepaths.add(changedFilepath);
				console.log(
					"hit change but rebuild is not finished, caching files: ",
					pendingChangedFilepaths
				);
				return;
			}

			const rebuildWithFilepaths = (changedFilepath: string[]) => {
				// Rebuild finished, we can start to rebuild again
				isBuildFinished = false;
				console.log("hit change and start to build");

				const begin = Date.now();
				this.unsafe_rebuild(changedFilepath, (error: any, diffStats) => {
					isBuildFinished = true;

					const hasPending = Boolean(pendingChangedFilepaths.size);

					// If we have any pending task left, we should rebuild again with the pending files
					if (hasPending) {
						const pending = [...pendingChangedFilepaths];
						pendingChangedFilepaths.clear();
						rebuildWithFilepaths(pending);
					}
					if (error) {
						throw error;
					}

					for (const [uri, stats] of Object.entries(diffStats)) {
						let relativePath = path.relative(this.options.context, uri);
						if (
							!(relativePath.startsWith("../") || relativePath.startsWith("./"))
						) {
							relativePath = "./" + relativePath;
						}

						// send Message
						if (ws) {
							const data = {
								uri: relativePath,
								content: stats.content
							};
							if (/\.[less|css|sass|scss]$/.test(data.uri)) {
								const cssOutput = fs
									.readFileSync(
										path.resolve(this.options.output.path, "main.css")
									)
									.toString("utf-8");
								// TODO: need support more
								data.content = [
									`var cssStyleTag = document.querySelector("style[id='hot-css']")`,
									`if (cssStyleTag) {`,
									`  cssStyleTag.innerText = \`${cssOutput}\``,
									`} else {`,
									`  var newCSStyleTag = document.createElement('style')`,
									`  newCSStyleTag.setAttribute('id', 'hot-css')`,
									`  newCSStyleTag.innerText = \`${cssOutput}\``,
									`  document.head.appendChild(newCSStyleTag)`,
									`}`,
									``,
									`var outdataCSSLinkTag = document.querySelector("link[href='main.css']")`,
									`outdataCSSLinkTag && outdataCSSLinkTag.parentNode && outdataCSSLinkTag.parentNode.removeChild(outdataCSSLinkTag)`
								].join("\n");
							}

							for (const client of ws.clients) {
								// the type of "ok" means rebuild success.
								// the data should deleted after we had hash in stats.
								client.send(
									JSON.stringify({ type: "ok", data: JSON.stringify(data) })
								);
							}
						}
					}
					console.log("rebuild success, time cost", Date.now() - begin);
				});
			};

			rebuildWithFilepaths([...pendingChangedFilepaths, changedFilepath]);
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
