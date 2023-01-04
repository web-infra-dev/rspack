import type { Compilation, Compiler } from ".";
import { Stats } from ".";
import { WatchOptions } from "./config/watch";
import { FileSystemInfoEntry, Watcher } from "./util/fs";

class Watching {
	watcher?: Watcher;
	pausedWatcher?: Watcher;
	compiler: Compiler;
	handler: (error?: Error, stats?: Stats) => void;
	watchOptions: WatchOptions;
	lastWatcherStartTime: number;
	running: boolean;
	#initial: boolean;
	#closed: boolean;
	#collectedChangedFiles?: Set<string>;
	#collectedRemovedFiles?: Set<string>;

	constructor(
		compiler: Compiler,
		watchOptions: WatchOptions,
		handler: (error?: Error, stats?: Stats) => void
	) {
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
					return this.handler(err);
				}
				this.#invalidate(
					fileTimeInfoEntries,
					contextTimeInfoEntries,
					changedFiles,
					removedFiles
				);
			},
			() => {}
		);
	}

	close(callback?: () => void) {
		this.#closed = true;
		if (this.watcher) {
			this.watcher.close();
			this.watcher = null;
		}
		if (this.pausedWatcher) {
			this.pausedWatcher.close();
			this.pausedWatcher = null;
		}
		this.compiler.watching = undefined;
		this.compiler.watchMode = false;
		callback();
	}

	invalidate() {
		this.#invalidate();
	}

	#invalidate(
		fileTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		contextTimeInfoEntries?: Map<string, FileSystemInfoEntry | "ignore">,
		changedFiles?: Set<string>,
		removedFiles?: Set<string>
	) {
		if (this.running) {
			this.#mergeWithCollected(changedFiles, removedFiles);
			console.log("hit change but rebuild is not finished, pending files: ", [
				...this.#collectedChangedFiles,
				...this.#collectedRemovedFiles
			]);
			return;
		}
		this.#go(changedFiles, removedFiles);
	}

	#go(changedFiles?: ReadonlySet<string>, removedFiles?: ReadonlySet<string>) {
		this.running = true;
		if (this.watcher) {
			this.pausedWatcher = this.watcher;
			this.lastWatcherStartTime = Date.now();
			this.watcher.pause();
			this.watcher = null;
		} else if (!this.lastWatcherStartTime) {
			this.lastWatcherStartTime = Date.now();
		}
		const compile =
			this.compiler.options.devServer && !this.#initial
				? (changes, removals, cb) =>
						this.compiler.rebuild(changes, removals, cb)
				: (_a, _b, cb) => this.compiler.build(cb);
		const begin = Date.now();
		this.compiler.hooks.watchRun.callAsync(this.compiler, err => {
			if (err) this.#done(err);
			compile(changedFiles, removedFiles, (err) => {
				this.#done(err, this.compiler.compilation);
				console.log("rebuild success, time cost", Date.now() - begin, "ms");
			});
		});
	}

	#done(error?: Error, compilation?: Compilation) {
		this.running = false;
		if (error) {
			return this.handler(error);
		}
		console.time('get stats old');
		const rawStats = this.compiler.compilation.getStatsOld();
		console.timeEnd('get stats old');
		console.time('get stats new');
		const newStats = this.compiler.compilation.getStats();
		console.timeEnd('get stats new');
		const stats = new Stats(this.compiler.compilation);
		this.handler(undefined, stats);
		this.compiler.hooks.done.callAsync(stats, () => {
			const hasPending =
				this.#collectedChangedFiles || this.#collectedRemovedFiles;
			// If we have any pending task left, we should rebuild again with the pending files
			if (hasPending) {
				const pendingChengedFiles = this.#collectedChangedFiles;
				const pendingRemovedFiles = this.#collectedRemovedFiles;
				this.#collectedChangedFiles = undefined;
				this.#collectedRemovedFiles = undefined;
				this.#go(pendingChengedFiles, pendingRemovedFiles);
			}
			process.nextTick(() => {
				if (!this.#closed) {
					this.watch(
						this.compiler.compilation.fileDependencies,
						this.compiler.compilation.contextDependencies,
						this.compiler.compilation.missingDependencies
					);
				}
			});
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
				this.#collectedRemovedFiles.delete(file);
			}
			for (const file of removedFiles) {
				this.#collectedChangedFiles.delete(file);
				this.#collectedRemovedFiles.add(file);
			}
		}
	}
}

export default Watching;
