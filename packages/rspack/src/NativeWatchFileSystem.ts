import * as binding from "@rspack/binding";
import type { WatchOptions } from "./exports";
import type {
	FileSystemInfoEntry,
	WatchFileSystem,
	Watcher,
	WatcherDependencies
} from "./util/fs";

export default class NativeWatchFileSystem implements WatchFileSystem {
	#inner: binding.NativeWatcher | undefined;

	async watch(
		files: WatcherDependencies,
		directories: WatcherDependencies,
		missing: WatcherDependencies,
		_startTime: number,
		options: WatchOptions,
		callback: (
			error: Error | null,
			fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			changedFiles: Set<string>,
			removedFiles: Set<string>
		) => void,
		callbackUndelayed: (fileName: string, changeTime: number) => void
	): Promise<Watcher> {
		if (!this.#inner) {
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
			this.#inner = new binding.NativeWatcher(nativeWatcherOptions);
		}

		await this.#inner.watch(
			[files.add, files.remove],
			[directories.add, directories.remove],
			[missing.add, missing.remove],
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
				if (this.#inner) {
					this.#inner.close();
				}
			},

			pause: () => {
				if (this.#inner) {
					this.#inner.pause();
				}
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
}
