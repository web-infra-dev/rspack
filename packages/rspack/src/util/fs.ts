import { WatchOptions } from "../config";

export interface Watcher {
	close(): void; // closes the watcher and all underlying file watchers
	pause(): void; // pause closes the watcher, but keeps underlying file watchers alive until the next watch call
	getAggregatedChanges?(): Set<string>; // getAggregatedChanges get current aggregated changes that have not yet send to callback
	getAggregatedRemovals?(): Set<string>; // get current aggregated removals that have not yet send to callback
	getFileTimeInfoEntries?(): Map<string, FileSystemInfoEntry | "ignore">; // get info about files
	getContextTimeInfoEntries?(): Map<string, FileSystemInfoEntry | "ignore">; // get info about directories
	getInfo(): WatcherInfo; // get info about timestamps and changes
}

export interface WatcherInfo {
	changes: Set<string>; // get current aggregated changes that have not yet send to callback
	removals: Set<string>; // get current aggregated removals that have not yet send to callback
	fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">; // get info about files
	contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">; // get info about directories
}

export interface FileSystemInfoEntry {
	safeTime: number;
	timestamp?: number;
}

export interface WatchFileSystem {
	watch(
		files: Iterable<string>,
		directories: Iterable<string>,
		missing: Iterable<string>,
		startTime: number,
		options: WatchOptions,
		callback: (
			error: Error | null,
			fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			changedFiles: Set<string>,
			removedFiles: Set<string>
		) => void,
		callbackUndelayed: (fileName: string, changeTime: number) => void
	): Watcher;
}
