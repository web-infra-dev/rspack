import * as binding from "@rspack/binding";
import type Watchpack from "watchpack";
import type { FileSystemInfoEntry, Watcher, WatchFileSystem } from "./util/fs";

/**
 * The following code is modified based on
 * https://github.com/webpack/watchpack/blob/332b55016b7c32dab4134f793ca71a5141bd10c1/lib/watchpack.js#L33-L57
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/watchpack/blob/main/LICENSE
 */
const stringToRegexp = (ignored: string) => {
	if (ignored.length === 0) {
		return;
	}
	const globToRegExp: typeof import("glob-to-regexp") = require("glob-to-regexp");
	const { source } = globToRegExp(ignored, { globstar: true, extended: true });
	return `${source.slice(0, -1)}(?:$|\\/)`;
};

const ignoredToFunction = (ignored: Watchpack.WatchOptions["ignored"]) => {
	if (Array.isArray(ignored)) {
		const stringRegexps = ignored.map(i => stringToRegexp(i)).filter(Boolean);
		if (stringRegexps.length === 0) {
			return () => false;
		}
		const regexp = new RegExp(stringRegexps.join("|"));
		return (item: string) => regexp.test(item.replace(/\\/g, "/"));
	}
	if (typeof ignored === "string") {
		const stringRegexp = stringToRegexp(ignored);
		if (!stringRegexp) {
			return () => false;
		}
		const regexp = new RegExp(stringRegexp);
		return (item: string) => regexp.test(item.replace(/\\/g, "/"));
	}
	if (ignored instanceof RegExp) {
		return (item: string) => ignored.test(item.replace(/\\/g, "/"));
	}
	if (typeof ignored === "function") {
		return (item: string) => ignored(item);
	}
	if (ignored) {
		throw new Error(`Invalid option for 'ignored': ${ignored}`);
	}
	return undefined;
};

export default class NativeWatchFileSystem implements WatchFileSystem {
	#inner: binding.NativeWatcher | undefined;

	watch(
		files: Iterable<string> & {
			added?: Iterable<string>;
			removed?: Iterable<string>;
		},
		directories: Iterable<string> & {
			added?: Iterable<string>;
			removed?: Iterable<string>;
		},
		missing: Iterable<string> & {
			added?: Iterable<string>;
			removed?: Iterable<string>;
		},
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
	): Watcher {
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

		nativeWatcher.watch(
			[Array.from(files.added!), Array.from(files.removed!)],
			[Array.from(directories.added!), Array.from(directories.removed!)],
			[Array.from(missing.added!), Array.from(missing.removed!)],
			(err: Error | null, result) => {
				const { changedFiles, removedFiles } = result;
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

		const nativeWatcherOptions: binding.NativeWatcherOptions = {
			followSymlinks: options.followSymlinks,
			aggregateTimeout: options.aggregateTimeout,
			pollInterval: typeof options.poll === "boolean" ? 0 : options.poll,
			ignored: ignoredToFunction(options.ignored)
		};
		const nativeWatcher = new binding.NativeWatcher(nativeWatcherOptions);
		this.#inner = nativeWatcher;
		return nativeWatcher;
	}
}
