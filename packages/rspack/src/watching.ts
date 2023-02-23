import { Callback } from "tapable";
import type { Compilation, Compiler } from ".";
import { Stats } from ".";
import { WatchOptions } from "./config";
import { FileSystemInfoEntry, Watcher } from "./util/fs";

class Watching {
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
	#invalidReported: boolean;
	#closeCallbacks?: ((err?: Error) => void)[];
	#initial: boolean;
	#closed: boolean;
	#collectedChangedFiles?: Set<string>;
	#collectedRemovedFiles?: Set<string>;

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

		process.nextTick(() => {
			if (this.#initial) this.#invalidate();
			this.#initial = false;
		});
	}

	watch(
		files: Iterable<string>,
		dirs: Iterable<string>,
		missing: Iterable<string>
	) {
		// @ts-expect-error
		this.pausedWatcher = null;
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

		const finalCallback = (err?: Error) => {
			this.running = false;
			this.compiler.running = false;
			this.compiler.watching = undefined;
			this.compiler.watchMode = false;
			this.compiler.modifiedFiles = undefined;
			this.compiler.removedFiles = undefined;
			// this.compiler.fileTimestamps = undefined;
			// this.compiler.contextTimestamps = undefined;
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
			finalCallback();
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

	#invalidate(
		fileTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		contextTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		changedFiles?: Set<string>,
		removedFiles?: Set<string>
	) {
		// @ts-expect-error
		if (this.isBlocked() && (this.blocked = true)) {
			// @ts-expect-error
			this.#mergeWithCollected(changedFiles, removedFiles);
			return;
		}

		if (this.running) {
			// @ts-expect-error
			this.#mergeWithCollected(changedFiles, removedFiles);
			this.invalid = true;
			console.log("hit change but rebuild is not finished, pending files: ", [
				...(this.#collectedChangedFiles || new Set()),
				...(this.#collectedRemovedFiles || new Set())
			]);
			return;
		}
		this.#go(changedFiles, removedFiles);
	}

	#go(changedFiles?: ReadonlySet<string>, removedFiles?: ReadonlySet<string>) {
		this.running = true;
		const logger = this.compiler.getInfrastructureLogger("watcher");
		if (this.watcher) {
			this.pausedWatcher = this.watcher;
			this.lastWatcherStartTime = Date.now();
			this.watcher.pause();
			// @ts-expect-error
			this.watcher = null;
		} else if (!this.lastWatcherStartTime) {
			this.lastWatcherStartTime = Date.now();
		}
		const modifiedFiles = (this.compiler.modifiedFiles =
			this.#collectedChangedFiles);
		const deleteFiles = (this.compiler.removedFiles =
			this.#collectedRemovedFiles);
		this.#collectedChangedFiles = undefined;
		this.#collectedRemovedFiles = undefined;
		const begin = Date.now();
		this.invalid = false;
		this.#invalidReported = false;
		this.compiler.hooks.watchRun.callAsync(this.compiler, err => {
			if (err) return this._done(err);

			const isRebuild = this.compiler.options.devServer && !this.#initial;
			const print = isRebuild
				? () =>
						console.log("rebuild success, time cost", Date.now() - begin, "ms")
				: () =>
						console.log("build success, time cost", Date.now() - begin, "ms");

			const onBuild = (err: Error) => {
				if (err) return this._done(err);
				// if (this.invalid) return this._done(null);
				// @ts-expect-error
				this._done(null);
				if (!err && !this.#closed && !this.invalid) {
					print();
				}
			};

			if (isRebuild) {
				this.compiler.rebuild(modifiedFiles, deleteFiles, onBuild as any);
			} else {
				// @ts-expect-error
				this.compiler.build(onBuild);
			}
		});
	}

	/**
	 * The reason why this is _done instead of #done, is that in Webpack,
	 * it will rewrite this function to another function
	 */
	private _done(error?: Error) {
		this.running = false;
		let stats: Stats | null = null;
		const handleError = (err?: Error, cbs?: Callback<Error, void>[]) => {
			// @ts-expect-error
			this.compiler.hooks.failed.call(err);
			// this.compiler.cache.beginIdle();
			// this.compiler.idle = true;
			// @ts-expect-error
			this.handler(err, stats);
			if (!cbs) {
				cbs = this.callbacks;
				this.callbacks = [];
			}
			// @ts-expect-error
			for (const cb of cbs) cb(err);
		};

		if (error) {
			return handleError(error);
		}
		const cbs = this.callbacks;
		this.callbacks = [];

		stats = new Stats(this.compiler.compilation);
		this.compiler.hooks.done.callAsync(stats, err => {
			if (err) return handleError(err, cbs);
			// @ts-expect-error
			this.handler(null, stats);

			process.nextTick(() => {
				if (!this.#closed) {
					this.watch(
						this.compiler.compilation.fileDependencies,
						this.compiler.compilation.contextDependencies,
						this.compiler.compilation.missingDependencies
					);
				}
			});
			for (const cb of cbs) cb(null);
			// @ts-expect-error
			this.compiler.hooks.afterDone.call(stats);
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
}

export default Watching;
