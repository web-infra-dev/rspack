/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/fs.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import assert from "assert";
import path from "path";

import type { WatchOptions } from "../config";

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

interface IDirent {
	isFile: () => boolean;
	isDirectory: () => boolean;
	isBlockDevice: () => boolean;
	isCharacterDevice: () => boolean;
	isSymbolicLink: () => boolean;
	isFIFO: () => boolean;
	isSocket: () => boolean;
	name: string | Buffer;
}

interface IStats {
	isFile: () => boolean;
	isDirectory: () => boolean;
	isBlockDevice: () => boolean;
	isCharacterDevice: () => boolean;
	isSymbolicLink: () => boolean;
	isFIFO: () => boolean;
	isSocket: () => boolean;
	dev: number | bigint;
	ino: number | bigint;
	mode: number | bigint;
	nlink: number | bigint;
	uid: number | bigint;
	gid: number | bigint;
	rdev: number | bigint;
	size: number | bigint;
	blksize: number | bigint;
	blocks: number | bigint;
	atimeMs: number | bigint;
	mtimeMs: number | bigint;
	ctimeMs: number | bigint;
	birthtimeMs: number | bigint;
	atime: Date;
	mtime: Date;
	ctime: Date;
	birthtime: Date;
}

export interface OutputFileSystem {
	writeFile: (
		arg0: string,
		arg1: string | Buffer,
		arg2: (arg0?: null | NodeJS.ErrnoException) => void
	) => void;
	mkdir: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException) => void
	) => void;
	readdir: (
		arg0: string,
		arg1: (
			arg0?: null | NodeJS.ErrnoException,
			arg1?: (string | Buffer)[] | IDirent[]
		) => void
	) => void;
	rmdir: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException) => void
	) => void;
	unlink: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException) => void
	) => void;
	stat: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException, arg1?: IStats) => void
	) => void;
	lstat?: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException, arg1?: IStats) => void
	) => void;
	readFile: (
		arg0: string,
		arg1: (arg0?: null | NodeJS.ErrnoException, arg1?: string | Buffer) => void
	) => void;
	join?: (arg0: string, arg1: string) => string;
	relative?: (arg0: string, arg1: string) => string;
	dirname?: (arg0: string) => string;
}

export function rmrf(
	fs: OutputFileSystem,
	p: string,
	callback: (err?: Error | null) => void
) {
	fs.stat(p, (err, stats) => {
		if (err) {
			if (err.code === "ENOENT") {
				return callback();
			}
			return callback(err);
		}
		if (stats!.isDirectory()) {
			fs.readdir(p, (err, files) => {
				if (err) {
					return callback(err);
				}
				let count = files!.length;
				if (count === 0) {
					fs.rmdir(p, callback);
				} else {
					files!.forEach(file => {
						assert(typeof file === "string");
						const fullPath = join(fs, p, file);
						rmrf(fs, fullPath, err => {
							if (err) {
								return callback(err);
							}
							count--;
							if (count === 0) {
								fs.rmdir(p, callback);
							}
						});
					});
				}
			});
		} else {
			fs.unlink(p, callback);
		}
	});
}

const join = (fs: OutputFileSystem, rootPath: string, filename: string) => {
	if (fs && fs.join) {
		return fs.join(rootPath, filename);
	} else if (path.posix.isAbsolute(rootPath)) {
		return path.posix.join(rootPath, filename);
	} else if (path.win32.isAbsolute(rootPath)) {
		return path.win32.join(rootPath, filename);
	} else {
		throw new Error(
			`${rootPath} is neither a posix nor a windows path, and there is no 'join' method defined in the file system`
		);
	}
};

const dirname = (fs: OutputFileSystem, absPath: string) => {
	if (fs && fs.dirname) {
		return fs.dirname(absPath);
	} else if (path.posix.isAbsolute(absPath)) {
		return path.posix.dirname(absPath);
	} else if (path.win32.isAbsolute(absPath)) {
		return path.win32.dirname(absPath);
	} else {
		throw new Error(
			`${absPath} is neither a posix nor a windows path, and there is no 'dirname' method defined in the file system`
		);
	}
};

export const mkdirp = (
	fs: OutputFileSystem,
	p: string,
	callback: (error?: Error) => void
) => {
	fs.mkdir(p, err => {
		if (err) {
			if (err.code === "ENOENT") {
				const dir = dirname(fs, p);
				if (dir === p) {
					callback(err);
					return;
				}
				mkdirp(fs, dir, err => {
					if (err) {
						callback(err);
						return;
					}
					fs.mkdir(p, err => {
						if (err) {
							if (err.code === "EEXIST") {
								callback();
								return;
							}
							callback(err);
							return;
						}
						callback();
					});
				});
				return;
			} else if (err.code === "EEXIST") {
				callback();
				return;
			}
			callback(err);
			return;
		}
		callback();
	});
};

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
