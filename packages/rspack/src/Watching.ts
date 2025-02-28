/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Watching.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import assert from "node:assert";
import type { Callback } from "@rspack/lite-tapable";

import type { Compilation, Compiler } from ".";
import { Stats } from ".";
import type { WatchOptions } from "./config";
import type { FileSystemInfoEntry, Watcher } from "./util/fs";

export class Watching {
	watcher?: Watcher;
	pausedWatcher?: Watcher;
	compiler: Compiler;
	handler: Callback<Error, Stats>;
	callbacks: Callback<Error, void>[];
	watchOptions: WatchOptions;
	// @ts-expect-error  lastWatcherStartTime will be assigned with Date.now() during initialization
	lastWatcherStartTime: number;
	running: boolean;
	blocked: boolean;
	isBlocked: () => boolean;
	onChange: () => void;
	onInvalid: () => void;
	invalid: boolean;
	startTime?: number;
	#invalidReported: boolean;
	#closeCallbacks?: ((err?: Error | null) => void)[];
	#initial: boolean;
	#closed: boolean;
	#collectedChangedFiles?: Set<string>;
	#collectedRemovedFiles?: Set<string>;
	suspended: boolean;

	constructor(
		compiler: Compiler,
		watchOptions: WatchOptions,
		handler: Callback<Error, Stats>
	) {
		this.callbacks = [];
		this.invalid = false;
		this.#invalidReported = true;
		this.blocked = false;
		this.isBlocked = () => false;
		this.onChange = () => {};
		this.onInvalid = () => {};
		this.compiler = compiler;
		this.running = false;
		this.#initial = true;
		this.#closed = false;
		this.watchOptions = watchOptions;
		this.handler = handler;
		this.suspended = false;

		// The default aggregateTimeout of watchpack is 200ms,
		// using smaller values can improve HMR performance
		if (typeof this.watchOptions.aggregateTimeout !== "number") {
			this.watchOptions.aggregateTimeout = 5;
		}
		// Ignore watching files in node_modules to reduce memory usage and make startup faster
		if (this.watchOptions.ignored === undefined) {
			this.watchOptions.ignored = /[\\/](?:\.git|node_modules)[\\/]/;
		}

		process.nextTick(() => {
			if (this.#initial) this.#invalidate();
		});
	}

	watch(
		files: Iterable<string>,
		dirs: Iterable<string>,
		missing: Iterable<string>
	) {
		this.pausedWatcher = undefined;
		// SAFETY: `watchFileSystem` is expected to be initialized.
		this.watcher = this.compiler.watchFileSystem!.watch(
			files,
			dirs,
			missing,
			this.lastWatcherStartTime,
			this.watchOptions,
			(
				err,
				fileTimeInfoEntries,
				contextTimeInfoEntries,
				changedFiles,
				removedFiles
			) => {
				if (err) {
					this.compiler.fileTimestamps = undefined;
					this.compiler.contextTimestamps = undefined;
					this.compiler.modifiedFiles = undefined;
					this.compiler.removedFiles = undefined;
					return this.handler(err);
				}
				this.#invalidate(
					fileTimeInfoEntries,
					contextTimeInfoEntries,
					changedFiles,
					removedFiles
				);
				this.onChange();
			},
			(fileName, changeTime) => {
				if (!this.#invalidReported) {
					this.#invalidReported = true;
					this.compiler.hooks.invalid.call(fileName, changeTime);
				}
				this.onInvalid();
			}
		);
	}

	close(callback?: () => void) {
		if (this.#closeCallbacks) {
			if (callback) {
				this.#closeCallbacks.push(callback);
			}
			return;
		}

		const finalCallback = (err: Error | null) => {
			this.running = false;
			this.compiler.running = false;
			this.compiler.watching = undefined;
			this.compiler.watchMode = false;
			this.compiler.modifiedFiles = undefined;
			this.compiler.removedFiles = undefined;
			this.compiler.fileTimestamps = undefined;
			this.compiler.contextTimestamps = undefined;
			// this.compiler.fsStartTime = undefined;
			const shutdown = (err: Error | null) => {
				this.compiler.hooks.watchClose.call();
				const closeCallbacks = this.#closeCallbacks!;
				this.#closeCallbacks = undefined;
				for (const cb of closeCallbacks) cb(err);
			};
			// TODO: compilation parameter support
			// if (compilation) {
			// 	const logger = compilation.getLogger("webpack.Watching");
			// 	logger.time("storeBuildDependencies");
			// 	this.compiler.cache.storeBuildDependencies(
			// 		compilation.buildDependencies,
			// 		err2 => {
			// 			logger.timeEnd("storeBuildDependencies");
			// 			shutdown(err || err2);
			// 		}
			// 	);
			// } else {
			// 	shutdown(err);
			// }
			shutdown(err);
		};

		this.#closed = true;
		if (this.watcher) {
			this.watcher.close();
			this.watcher = undefined;
		}
		if (this.pausedWatcher) {
			this.pausedWatcher.close();
			this.pausedWatcher = undefined;
		}
		this.compiler.watching = undefined;
		this.compiler.watchMode = false;
		this.#closeCallbacks = [];
		if (callback) {
			this.#closeCallbacks.push(callback);
		}
		if (this.running) {
			this.invalid = true;

			this._done = finalCallback;
		} else {
			finalCallback(null);
		}
	}

	invalidate(callback?: Callback<Error, void>) {
		if (callback) {
			this.callbacks.push(callback);
		}
		if (!this.#invalidReported) {
			this.#invalidReported = true;
			this.compiler.hooks.invalid.call(null, Date.now());
		}
		this.onChange();
		this.#invalidate();
	}

	lazyCompilationInvalidate(files: Set<string>) {
		this.#invalidate(new Map(), new Map(), files, new Set());
	}

	#invalidate(
		fileTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		contextTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		changedFiles?: Set<string>,
		removedFiles?: Set<string>
	) {
		this.#mergeWithCollected(changedFiles, removedFiles);
		if (this.suspended || (this.isBlocked() && (this.blocked = true))) {
			return;
		}

		if (this.running) {
			this.invalid = true;
			return;
		}

		this.#go(
			fileTimeInfoEntries,
			contextTimeInfoEntries,
			changedFiles,
			removedFiles
		);
	}

	#go(
		fileTimeInfoEntries?: ReadonlyMap<string, FileSystemInfoEntry | "ignore">,
		contextTimeInfoEntries?: ReadonlyMap<
			string,
			FileSystemInfoEntry | "ignore"
		>,
		changedFiles?: ReadonlySet<string>,
		removedFiles?: ReadonlySet<string>
	) {
		this.#initial = false;
		if (this.startTime === undefined) this.startTime = Date.now();
		this.running = true;
		if (this.watcher) {
			this.pausedWatcher = this.watcher;
			this.lastWatcherStartTime = Date.now();
			this.watcher.pause();
			this.watcher = undefined;
		} else if (!this.lastWatcherStartTime) {
			this.lastWatcherStartTime = Date.now();
		}

		if (
			fileTimeInfoEntries &&
			contextTimeInfoEntries &&
			changedFiles &&
			removedFiles
		) {
			this.#mergeWithCollected(changedFiles, removedFiles);
			this.compiler.fileTimestamps = fileTimeInfoEntries;
			this.compiler.contextTimestamps = contextTimeInfoEntries;
		} else if (this.pausedWatcher) {
			const { changes, removals, fileTimeInfoEntries, contextTimeInfoEntries } =
				this.pausedWatcher.getInfo();
			this.#mergeWithCollected(changes, removals);
			this.compiler.fileTimestamps = fileTimeInfoEntries;
			this.compiler.contextTimestamps = contextTimeInfoEntries;
		}

		this.compiler.modifiedFiles = this.#collectedChangedFiles;
		this.compiler.removedFiles = this.#collectedRemovedFiles;
		this.#collectedChangedFiles = undefined;
		this.#collectedRemovedFiles = undefined;
		this.invalid = false;
		this.#invalidReported = false;
		this.compiler.hooks.watchRun.callAsync(this.compiler, err => {
			if (err) return this._done(err);

			const onCompiled = (
				err: Error | null,
				_compilation: Compilation | undefined
			) => {
				if (err) return this._done(err);

				const compilation = _compilation!;

				const needAdditionalPass = compilation.hooks.needAdditionalPass.call();
				if (needAdditionalPass) {
					compilation.needAdditionalPass = true;

					compilation.startTime = this.startTime;
					compilation.endTime = Date.now();
					const stats = new Stats(compilation);
					this.compiler.hooks.done.callAsync(stats, err => {
						if (err) return this._done(err, compilation);

						this.compiler.hooks.additionalPass.callAsync(err => {
							if (err) return this._done(err, compilation);
							this.compiler.compile(onCompiled);
						});
					});
					return;
				}
				this._done(null, this.compiler._lastCompilation!);
			};

			this.compiler.compile(onCompiled);
		});
	}

	/**
	 * The reason why this is _done instead of #done, is that in Webpack,
	 * it will rewrite this function to another function
	 */
	private _done(error: Error | null, compilation?: Compilation) {
		this.running = false;
		let stats: undefined | Stats = undefined;

		const handleError = (err: Error, cbs?: Callback<Error, void>[]) => {
			this.compiler.hooks.failed.call(err);
			// this.compiler.cache.beginIdle();
			// this.compiler.idle = true;
			this.handler(err, stats);

			const callbacksToExecute = cbs || this.callbacks.splice(0);
			for (const cb of callbacksToExecute) {
				cb(err);
			}
		};

		if (error) {
			return handleError(error);
		}
		assert(compilation);

		stats = new Stats(compilation);

		if (
			this.invalid &&
			!this.suspended &&
			!this.blocked &&
			!(this.isBlocked() && (this.blocked = true))
		) {
			this.#go();
			return;
		}

		const startTime = this.startTime; // store last startTime for compilation
		// reset startTime for next compilation, before throwing error
		this.startTime = undefined;
		compilation.startTime = startTime;
		compilation.endTime = Date.now();
		const cbs = this.callbacks;
		this.callbacks = [];
		const fileDependencies = new Set([...compilation.fileDependencies]);
		const contextDependencies = new Set([...compilation.contextDependencies]);
		const missingDependencies = new Set([...compilation.missingDependencies]);

		this.compiler.hooks.done.callAsync(stats, err => {
			if (err) return handleError(err, cbs);
			this.handler(null, stats);

			process.nextTick(() => {
				if (!this.#closed) {
					this.watch(
						fileDependencies,
						contextDependencies,
						missingDependencies
					);
				}
			});
			for (const cb of cbs) cb(null);
			this.compiler.hooks.afterDone.call(stats!);
		});
	}

	#mergeWithCollected(
		changedFiles?: ReadonlySet<string>,
		removedFiles?: ReadonlySet<string>
	) {
		if (!changedFiles) return;
		if (!removedFiles) return;
		if (!this.#collectedChangedFiles || !this.#collectedRemovedFiles) {
			this.#collectedChangedFiles = new Set(changedFiles);
			this.#collectedRemovedFiles = new Set(removedFiles);
		} else {
			for (const file of changedFiles) {
				this.#collectedChangedFiles.add(file);
				this.#collectedRemovedFiles.delete(file);
			}
			for (const file of removedFiles) {
				this.#collectedChangedFiles.delete(file);
				this.#collectedRemovedFiles.add(file);
			}
		}
	}

	suspend() {
		this.suspended = true;
	}

	resume() {
		if (this.suspended) {
			this.suspended = false;
			this.#invalidate();
		}
	}
}
