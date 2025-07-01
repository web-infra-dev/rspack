import * as binding from "@rspack/binding";
import type Watchpack from "watchpack";
import type {
	FileSystemInfoEntry,
	WatchFileSystem,
	Watcher,
	WatcherIncrementalDependencies
} from "./util/fs";

export default class NativeWatchFileSystem implements WatchFileSystem {
	#inner: binding.NativeWatcher | undefined;

	async watch(
		files: WatcherIncrementalDependencies,
		directories: WatcherIncrementalDependencies,
		missing: WatcherIncrementalDependencies,
		_startTime: number,
		options: Watchpack.WatchOptions,
		callback: (
			error: Error | null,
			fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			changedFiles: Set<string>,
			removedFiles: Set<string>
		) => void,
		callbackUndelayed: (fileName: string, changeTime: number) => void
	): Promise<Watcher> {
		if (
			(!files.added || typeof files.added[Symbol.iterator] !== "function") &&
			(!files.removed || typeof files.removed[Symbol.iterator] !== "function")
		) {
			throw new Error("Invalid arguments: 'files'");
		}

		if (
			(!directories.added ||
				typeof directories.added[Symbol.iterator] !== "function") &&
			(!directories.removed ||
				typeof directories.removed[Symbol.iterator] !== "function")
		) {
			throw new Error("Invalid arguments: 'directories'");
		}

		if (typeof callback !== "function") {
			throw new Error("Invalid arguments: 'callback'");
		}

		if (typeof options !== "object") {
			throw new Error("Invalid arguments: 'options'");
		}
		if (typeof callbackUndelayed !== "function" && callbackUndelayed) {
			throw new Error("Invalid arguments: 'callbackUndelayed'");
		}

		const nativeWatcher = this.getNativeWatcher(options);

		await nativeWatcher.watch(
			[Array.from(files.added), Array.from(files.removed)],
			[Array.from(directories.added), Array.from(directories.removed)],
			[Array.from(missing.added), Array.from(missing.removed)],
			(err: Error | null, changedFiles: string[], removedFiles: string[]) => {
				// TODO: add fileTimeInfoEntries and contextTimeInfoEntries
				callback(
					err,
					new Map(),
					new Map(),
					new Set(changedFiles),
					new Set(removedFiles)
				);
			},
			(fileName: string) => {
				// TODO: add real change time
				callbackUndelayed(fileName, Date.now());
			}
		);

		return {
			close: () => {
				nativeWatcher.close();
			},

			pause: () => {
				nativeWatcher.pause();
			},

			getInfo() {
				// This is a placeholder implementation.
				// TODO: The actual implementation should return the current state of the watcher.
				return {
					changes: new Set(),
					removals: new Set(),
					fileTimeInfoEntries: new Map(),
					contextTimeInfoEntries: new Map()
				};
			}
		};
	}

	getNativeWatcher(options: Watchpack.WatchOptions): binding.NativeWatcher {
		if (this.#inner) {
			return this.#inner;
		}

		const ignoredCallback = options.ignored
			? async (path: string): Promise<boolean> => {
					const ignored = options.ignored;
					if (Array.isArray(ignored)) {
						return ignored.some(item => path.includes(item));
					}
					if (typeof ignored === "string") {
						return path.includes(ignored);
					}
					if (ignored instanceof RegExp) {
						return ignored.test(path);
					}
					return false;
				}
			: undefined;
		const nativeWatcherOptions: binding.NativeWatcherOptions = {
			followSymlinks: options.followSymlinks,
			aggregateTimeout: options.aggregateTimeout,
			pollInterval: typeof options.poll === "boolean" ? 0 : options.poll,
			ignored: ignoredCallback
		};
		const nativeWatcher = new binding.NativeWatcher(nativeWatcherOptions);
		this.#inner = nativeWatcher;
		return nativeWatcher;
	}
}
