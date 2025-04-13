/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/node/NodeWatchFileSystem.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import util from "node:util";
import Watchpack from "watchpack";

import type {
	FileSystemInfoEntry,
	InputFileSystem,
	WatchFileSystem,
	Watcher
} from "../util/fs";

export default class NodeWatchFileSystem implements WatchFileSystem {
	inputFileSystem: InputFileSystem;
	watcherOptions: Watchpack.WatchOptions;
	watcher: Watchpack;

	constructor(inputFileSystem: InputFileSystem) {
		this.inputFileSystem = inputFileSystem;
		this.watcherOptions = {
			aggregateTimeout: 0
		};
		this.watcher = new Watchpack(this.watcherOptions);
	}

	watch(
		files: Iterable<string>,
		directories: Iterable<string>,
		missing: Iterable<string>,
		startTime: number,
		options: Watchpack.WatchOptions,
		callback: (
			error: Error | null,
			fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			changedFiles: Set<string>,
			removedFiles: Set<string>
		) => void,
		callbackUndelayed: (fileName: string, changeTime: number) => void
	): Watcher {
		if (!files || typeof files[Symbol.iterator] !== "function") {
			throw new Error("Invalid arguments: 'files'");
		}
		if (!directories || typeof directories[Symbol.iterator] !== "function") {
			throw new Error("Invalid arguments: 'directories'");
		}
		if (!missing || typeof missing[Symbol.iterator] !== "function") {
			throw new Error("Invalid arguments: 'missing'");
		}
		if (typeof callback !== "function") {
			throw new Error("Invalid arguments: 'callback'");
		}
		if (typeof startTime !== "number" && startTime) {
			throw new Error("Invalid arguments: 'startTime'");
		}
		if (typeof options !== "object") {
			throw new Error("Invalid arguments: 'options'");
		}
		if (typeof callbackUndelayed !== "function" && callbackUndelayed) {
			throw new Error("Invalid arguments: 'callbackUndelayed'");
		}
		const oldWatcher = this.watcher;
		this.watcher = new Watchpack(options);

		if (callbackUndelayed) {
			this.watcher.once("change", callbackUndelayed);
		}

		const fetchTimeInfo = () => {
			const fileTimeInfoEntries = new Map();
			const contextTimeInfoEntries = new Map();
			if (this.watcher) {
				this.watcher.collectTimeInfoEntries(
					fileTimeInfoEntries,
					contextTimeInfoEntries
				);
			}
			return { fileTimeInfoEntries, contextTimeInfoEntries };
		};
		this.watcher.once("aggregated", (changes, removals) => {
			// pause emitting events (avoids clearing aggregated changes and removals on timeout)
			this.watcher.pause();

			if (this.inputFileSystem?.purge) {
				const fs = this.inputFileSystem;
				for (const item of changes) {
					fs.purge?.(item);
				}
				for (const item of removals) {
					fs.purge?.(item);
				}
			}
			const { fileTimeInfoEntries, contextTimeInfoEntries } = fetchTimeInfo();

			callback(
				null,
				fileTimeInfoEntries,
				contextTimeInfoEntries,
				changes,
				removals
			);
		});

		this.watcher.watch({ files, directories, missing, startTime });

		if (oldWatcher) {
			oldWatcher.close();
			/**
			 * NodeEnvironmentPlugin.ts sets `compiler.inputFileSystem` to a `CachedInputFileSystem`,
			 * which caches the content for 60s, unless `purge` is called.
			 *
			 * NodeWatchFileSystem will purge the cached content before read when watched files change.
			 * However, this doesn't cover some edge cases, which can sometimes lead to stale content being read.
			 * e.g. packages/rspack-test-tools/tests/watchCases/build-chunk-graph/chunk-modify/test.config.js
			 *
			 * > TLDR; in the 2nd step, only `index.js` is changed, cache of `dyn-2.js` is not purged,
			 * > new `this.watcher` doesn't watch `dyn-2.js`.
			 * > In the 3rd step, `index.js` and `dyn-2.js` are changed together. Changes to `index.js`
			 * > is detected, and the cached will be purged. But changes to `dyn-2.js` won't, ...
			 *
			 * Currently, Rspack can pass this test case, because the binding doesn't read js inputFileSystem,
			 * and it doesn't cach the file content.
			 *
			 * **This is a short term solution**
			 */
			this.inputFileSystem.purge?.();
		}
		return {
			close: () => {
				if (this.watcher) {
					this.watcher.close();
					this.watcher = null as any;
				}
			},
			pause: () => {
				if (this.watcher) {
					this.watcher.pause();
				}
			},
			getAggregatedRemovals: util.deprecate(
				() => {
					const items = this.watcher?.aggregatedRemovals;
					if (items && this.inputFileSystem?.purge) {
						const fs = this.inputFileSystem;
						for (const item of items) {
							fs.purge?.(item);
						}
					}
					return items;
				},
				"Watcher.getAggregatedRemovals is deprecated in favor of Watcher.getInfo since that's more performant.",
				"DEP_WEBPACK_WATCHER_GET_AGGREGATED_REMOVALS"
			),
			getAggregatedChanges: util.deprecate(
				() => {
					const items = this.watcher?.aggregatedChanges;
					if (items && this.inputFileSystem?.purge) {
						const fs = this.inputFileSystem;
						for (const item of items) {
							fs.purge?.(item);
						}
					}
					return items;
				},
				"Watcher.getAggregatedChanges is deprecated in favor of Watcher.getInfo since that's more performant.",
				"DEP_WEBPACK_WATCHER_GET_AGGREGATED_CHANGES"
			),
			getFileTimeInfoEntries: util.deprecate(
				() => {
					return fetchTimeInfo().fileTimeInfoEntries;
				},
				"Watcher.getFileTimeInfoEntries is deprecated in favor of Watcher.getInfo since that's more performant.",
				"DEP_WEBPACK_WATCHER_FILE_TIME_INFO_ENTRIES"
			),
			getContextTimeInfoEntries: util.deprecate(
				() => {
					return fetchTimeInfo().contextTimeInfoEntries;
				},
				"Watcher.getContextTimeInfoEntries is deprecated in favor of Watcher.getInfo since that's more performant.",
				"DEP_WEBPACK_WATCHER_CONTEXT_TIME_INFO_ENTRIES"
			),
			getInfo: () => {
				const removals = this.watcher?.aggregatedRemovals;
				const changes = this.watcher?.aggregatedChanges;
				if (this.inputFileSystem?.purge) {
					const fs = this.inputFileSystem;
					if (removals) {
						for (const item of removals) {
							fs.purge?.(item);
						}
					}
					if (changes) {
						for (const item of changes) {
							fs.purge?.(item);
						}
					}
				}
				const { fileTimeInfoEntries, contextTimeInfoEntries } = fetchTimeInfo();
				return {
					changes,
					removals,
					fileTimeInfoEntries,
					contextTimeInfoEntries
				};
			}
		};
	}
}
