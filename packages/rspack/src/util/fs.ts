import { WatchOptions } from "../config";
import type * as FsModule from "node:fs";
import { Buffer } from "buffer";
import { Stats, StatsBase, StatsFsBase } from "node:fs";

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

type IOException = null | NodeJS.ErrnoException;
type IOData = string | Buffer;

export interface IDirent
	extends Pick<
		StatsBase<unknown>,
		| "isFile"
		| "isDirectory"
		| "isBlockDevice"
		| "isCharacterDevice"
		| "isSymbolicLink"
		| "isFIFO"
		| "isSocket"
	> {
	name: string | Buffer;
}

export interface IStats extends StatsBase<number | bigint> {}

export interface InputFileSystem {
	readFile: (
		filepath: string,
		callback: (error?: IOException, data?: IOData) => void
	) => void;
	readJson?: (
		filepath: string,
		callback: (error?: IOException | Error, data?: any) => void
	) => void;
	readlink: (
		filepath: string,
		callback: (error?: IOException, data?: IOData) => void
	) => void;
	readdir: (
		path: string,
		callback: (error?: IOException, stats?: IOData[] | IDirent[]) => void
	) => void;
	stat: (
		path: string,
		callback: (error?: IOException, stats?: IStats) => void
	) => void;
	lstat?: (
		path: string,
		callback: (error?: IOException, stats?: IStats) => void
	) => void;
	realpath?: (
		path: string,
		callback: (error?: IOException, data?: IOData) => void
	) => void;
	purge?: (path?: string) => void;
	join?: (p0: string, p1: string) => string;
	relative?: (from: string, to: string) => string;
	dirname?: (path: string) => string;
}
