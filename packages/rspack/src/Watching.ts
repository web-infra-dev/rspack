/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Watching.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import { Callback } from "tapable";
import type { Compilation, Compiler } from ".";
import { Stats } from ".";
import { WatchOptions } from "./config";
import { FileSystemInfoEntry, Watcher } from "./util/fs";
import assert from "assert";

export class Watching {
	watcher?: Watcher;
	pausedWatcher?: Watcher;
	compiler: Compiler;
	handler: (error?: Error, stats?: Stats) => void;
	callbacks: Callback<Error, void>[];
	watchOptions: WatchOptions;
	// @ts-expect-error
	lastWatcherStartTime: number;
	running: boolean;
	blocked: boolean;
	isBlocked?: () => boolean;
	onChange?: () => void;
	onInvalid?: () => void;
	invalid: boolean;
	startTime?: number;
	#invalidReported: boolean;
	#closeCallbacks?: ((err?: Error) => void)[];
	#initial: boolean;
	#closed: boolean;
	#collectedChangedFiles?: Set<string>;
	#collectedRemovedFiles?: Set<string>;
	suspended: boolean;

	constructor(
		compiler: Compiler,
		watchOptions: WatchOptions,
		handler: (error?: Error, stats?: Stats) => void
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

		// The default aggregateTimeout of WatchPack is 200ms,
		// using smaller values can improve hmr performance
		if (typeof this.watchOptions.aggregateTimeout !== "number") {
			this.watchOptions.aggregateTimeout = 5;
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
		this.watcher = this.compiler.watchFileSystem.watch(
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
				// @ts-expect-error
				this.onChange();
			},
			(fileName, changeTime) => {
				if (!this.#invalidReported) {
					this.#invalidReported = true;
					this.compiler.hooks.invalid.call(fileName, changeTime);
				}
				// @ts-expect-error
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
			const shutdown = (err: Error) => {
				this.compiler.hooks.watchClose.call();
				const closeCallbacks = this.#closeCallbacks;
				this.#closeCallbacks = undefined;
				// @ts-expect-error
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
			// @ts-expect-error
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
		// @ts-expect-error
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
		// @ts-expect-error
		this.#mergeWithCollected(changedFiles, removedFiles);
		// @ts-expect-error
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

		const modifiedFiles = (this.compiler.modifiedFiles =
			this.#collectedChangedFiles);
		const deleteFiles = (this.compiler.removedFiles =
			this.#collectedRemovedFiles);
		this.#collectedChangedFiles = undefined;
		this.#collectedRemovedFiles = undefined;
		this.invalid = false;
		this.#invalidReported = false;
		this.compiler.hooks.watchRun.callAsync(this.compiler, err => {
			if (err) return this._done(err, null);

			const canRebuild =
				!this.#initial && (modifiedFiles?.size || deleteFiles?.size);

			const onCompile = (err: Error | null) => {
				if (err) return this._done(err, null);
				// if (this.invalid) return this._done(null);
				this._done(null, this.compiler.compilation!);
			};

			this.compiler.compile(onCompile);
			if (!canRebuild) {
				this.#initial = false;
			}
		});
	}

	/**
	 * The reason why this is _done instead of #done, is that in Webpack,
	 * it will rewrite this function to another function
	 */
	private _done(error: Error, compilation: null): void;
	private _done(error: null, compilation: Compilation): void;
	private _done(error: Error | null, compilation: Compilation | null) {
		this.running = false;
		let stats: undefined | Stats = undefined;

		const handleError = (err: Error, cbs?: Callback<Error, void>[]) => {
			this.compiler.hooks.failed.call(err);
			// this.compiler.cache.beginIdle();
			// this.compiler.idle = true;
			this.handler(err, stats);
			if (!cbs) {
				cbs = this.callbacks;
				this.callbacks = [];
			}
			for (const cb of cbs) cb(err);
		};

		const cbs = this.callbacks;
		this.callbacks = [];
		const startTime = this.startTime; // store last startTime for compilation
		// reset startTime for next compilation, before throwing error
		this.startTime = undefined;
		if (error) {
			return handleError(error);
		}
		assert(compilation);

		compilation.startTime = startTime;
		compilation.endTime = Date.now();
		stats = new Stats(compilation);

		this.compiler.hooks.done.callAsync(stats, err => {
			if (err) return handleError(err, cbs);
			// @ts-expect-error
			this.handler(null, stats);

			process.nextTick(() => {
				if (!this.#closed) {
					this.watch(
						compilation.fileDependencies,
						compilation.contextDependencies,
						compilation.missingDependencies
					);
				}
			});
			for (const cb of cbs) cb(null);
			this.compiler.hooks.afterDone.call(stats!);
		});
	}

	#mergeWithCollected(
		changedFiles: ReadonlySet<string>,
		removedFiles: ReadonlySet<string>
	) {
		if (!changedFiles) return;
		if (!this.#collectedChangedFiles) {
			this.#collectedChangedFiles = new Set(changedFiles);
			this.#collectedRemovedFiles = new Set(removedFiles);
		} else {
			for (const file of changedFiles) {
				this.#collectedChangedFiles.add(file);
				// @ts-expect-error
				this.#collectedRemovedFiles.delete(file);
			}
			for (const file of removedFiles) {
				this.#collectedChangedFiles.delete(file);
				// @ts-expect-error
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
