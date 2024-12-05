/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/fs.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import assert from "node:assert";
import type { Abortable } from "node:events";
import path from "node:path";

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

export type IStatsBase<T> = {
	isFile: () => boolean;
	isDirectory: () => boolean;
	isBlockDevice: () => boolean;
	isCharacterDevice: () => boolean;
	isSymbolicLink: () => boolean;
	isFIFO: () => boolean;
	isSocket: () => boolean;
	dev: T;
	ino: T;
	mode: T;
	nlink: T;
	uid: T;
	gid: T;
	rdev: T;
	size: T;
	blksize: T;
	blocks: T;
	atimeMs: T;
	mtimeMs: T;
	ctimeMs: T;
	birthtimeMs: T;
	atime: Date;
	mtime: Date;
	ctime: Date;
	birthtime: Date;
};

export type IStats = IStatsBase<number>;

export type IBigIntStats = IStatsBase<bigint> & {
	atimeNs: bigint;
	mtimeNs: bigint;
	ctimeNs: bigint;
	birthtimeNs: bigint;
};

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

export interface OutputFileSystem {
	writeFile: (
		arg0: string | number,
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

export type JsonPrimitive = string | number | boolean | null;
export type JsonArray = JsonValue[];
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;
export type JsonObject = { [Key in string]: JsonValue } & {
	[Key in string]?: JsonValue | undefined;
};

export type NoParamCallback = (err: NodeJS.ErrnoException | null) => void;
export type StringCallback = (
	err: NodeJS.ErrnoException | null,
	data?: string
) => void;
export type BufferCallback = (
	err: NodeJS.ErrnoException | null,
	data?: Buffer
) => void;
export type StringOrBufferCallback = (
	err: NodeJS.ErrnoException | null,
	data?: string | Buffer
) => void;
export type ReaddirStringCallback = (
	err: NodeJS.ErrnoException | null,
	files?: string[]
) => void;
export type ReaddirBufferCallback = (
	err: NodeJS.ErrnoException | null,
	files?: Buffer[]
) => void;
export type ReaddirStringOrBufferCallback = (
	err: NodeJS.ErrnoException | null,
	files?: string[] | Buffer[]
) => void;
export type ReaddirDirentCallback = (
	err: NodeJS.ErrnoException | null,
	files?: IDirent[]
) => void;
export type StatsCallback = (
	err: NodeJS.ErrnoException | null,
	stats?: IStats
) => void;
export type BigIntStatsCallback = (
	err: NodeJS.ErrnoException | null,
	stats?: IBigIntStats
) => void;
export type StatsOrBigIntStatsCallback = (
	err: NodeJS.ErrnoException | null,
	stats?: IStats | IBigIntStats
) => void;
export type NumberCallback = (
	err: NodeJS.ErrnoException | null,
	data?: number
) => void;
export type ReadJsonCallback = (
	err: NodeJS.ErrnoException | Error | null,
	data?: JsonObject
) => void;

export type PathLike = string | Buffer | URL;
export type PathOrFileDescriptor = PathLike | number;

export type ObjectEncodingOptions = {
	encoding?: BufferEncoding | null;
};

export type ReadFile = {
	(
		path: PathOrFileDescriptor,
		options:
			| ({
					encoding: null | undefined;
					flag?: string;
			  } & Abortable)
			| null
			| undefined,
		callback: BufferCallback
	): void;
	(
		path: PathOrFileDescriptor,
		options:
			| ({ encoding: BufferEncoding; flag?: string } & Abortable)
			| BufferEncoding,
		callback: StringCallback
	): void;
	(
		path: PathOrFileDescriptor,
		options:
			| (ObjectEncodingOptions & { flag?: string } & Abortable)
			| BufferEncoding
			| null
			| undefined,
		callback: StringOrBufferCallback
	): void;
	(path: PathOrFileDescriptor, callback: BufferCallback): void;
};

export type ReadFileSync = {
	(
		path: PathOrFileDescriptor,
		options: {
			encoding: null | undefined;
			flag?: string;
		} | null
	): Buffer;
	(
		path: PathOrFileDescriptor,
		options: { encoding: BufferEncoding; flag?: string } | BufferEncoding
	): string;
	(
		path: PathOrFileDescriptor,
		options: (ObjectEncodingOptions & { flag?: string }) | BufferEncoding | null
	): string | Buffer;
};

export type EncodingOption =
	| ObjectEncodingOptions
	| BufferEncoding
	| undefined
	| null;

export type BufferEncodingOption = "buffer" | { encoding: "buffer" };

export type StatOptions = {
	bigint?: boolean;
};

export type StatSyncOptions = {
	bigint?: boolean;
	throwIfNoEntry?: boolean;
};

export type Readlink = {
	(path: PathLike, options: EncodingOption, callback: StringCallback): void;
	(
		path: PathLike,
		options: BufferEncodingOption,
		callback: BufferCallback
	): void;
	(
		path: PathLike,
		options: EncodingOption,
		callback: StringOrBufferCallback
	): void;
	(path: PathLike, callback: StringCallback): void;
};

export type ReadlinkSync = {
	(path: PathLike, options: EncodingOption): string;
	(path: PathLike, options: BufferEncodingOption): Buffer;
	(path: PathLike, options: EncodingOption): string | Buffer;
};

export type Readdir = {
	(
		path: PathLike,
		options:
			| {
					encoding: BufferEncoding | null;
					withFileTypes?: false;
					recursive?: boolean;
			  }
			| BufferEncoding
			| null
			| undefined,
		callback: ReaddirStringCallback
	): void;
	(
		path: PathLike,
		options:
			| { encoding: "buffer"; withFileTypes?: false; recursive?: boolean }
			| "buffer",
		callback: ReaddirBufferCallback
	): void;
	(path: PathLike, callback: ReaddirStringCallback): void;
	(
		path: PathLike,
		options:
			| (ObjectEncodingOptions & { withFileTypes: true; recursive?: boolean })
			| BufferEncoding
			| null
			| undefined,
		callback: ReaddirStringOrBufferCallback
	): void;
	(
		path: PathLike,
		options: ObjectEncodingOptions & {
			withFileTypes: true;
			recursive?: boolean;
		},
		callback: ReaddirDirentCallback
	): void;
};

export type ReaddirSync = {
	(
		path: PathLike,
		options:
			| {
					encoding: BufferEncoding | null;
					withFileTypes?: false;
					recursive?: boolean;
			  }
			| BufferEncoding
			| null
	): string[];
	(
		path: PathLike,
		options:
			| { encoding: "buffer"; withFileTypes?: false; recursive?: boolean }
			| "buffer"
	): Buffer[];
	(
		path: PathLike,
		options:
			| (ObjectEncodingOptions & { withFileTypes?: false; recursive?: boolean })
			| BufferEncoding
			| null
	): string[] | Buffer[];
	(
		path: PathLike,
		options: ObjectEncodingOptions & {
			withFileTypes: true;
			recursive?: boolean;
		}
	): IDirent[];
};

export type Stat = {
	(path: PathLike, callback: StatsCallback): void;
	(
		path: PathLike,
		options: (StatOptions & { bigint?: false }) | undefined,
		callback: StatsCallback
	): void;
	(
		path: PathLike,
		options: StatOptions & { bigint: true },
		callback: BigIntStatsCallback
	): void;
	(
		path: PathLike,
		options: StatOptions | undefined,
		callback: StatsOrBigIntStatsCallback
	): void;
};

export type StatSync = {
	(path: PathLike, options?: undefined): IStats;
	(
		path: PathLike,
		options?: StatSyncOptions & { bigint?: false; throwIfNoEntry: false }
	): IStats | undefined;
	(
		path: PathLike,
		options: StatSyncOptions & { bigint: true; throwIfNoEntry: false }
	): IBigIntStats | undefined;
	(path: PathLike, options?: StatSyncOptions & { bigint?: false }): IStats;
	(path: PathLike, options: StatSyncOptions & { bigint: true }): IBigIntStats;
	(
		path: PathLike,
		options: StatSyncOptions & { bigint: boolean; throwIfNoEntry?: false }
	): IStats | IBigIntStats;
	(
		path: PathLike,
		options?: StatSyncOptions
	): IStats | IBigIntStats | undefined;
};

export type LStat = {
	(path: PathLike, callback: StatsCallback): void;
	(
		path: PathLike,
		options: (StatOptions & { bigint?: false }) | undefined,
		callback: StatsCallback
	): void;
	(
		path: PathLike,
		options: StatOptions & { bigint: true },
		callback: BigIntStatsCallback
	): void;
	(
		path: PathLike,
		options: StatOptions | undefined,
		callback: StatsOrBigIntStatsCallback
	): void;
};

export type LStatSync = {
	(path: PathLike, options?: undefined): IStats;
	(
		path: PathLike,
		options?: StatSyncOptions & { bigint?: false; throwIfNoEntry: false }
	): IStats | undefined;
	(
		path: PathLike,
		options: StatSyncOptions & { bigint: true; throwIfNoEntry: false }
	): IBigIntStats | undefined;
	(path: PathLike, options?: StatSyncOptions & { bigint?: false }): IStats;
	(path: PathLike, options: StatSyncOptions & { bigint: true }): IBigIntStats;
	(
		path: PathLike,
		options: StatSyncOptions & { bigint: boolean; throwIfNoEntry?: false }
	): IStats | IBigIntStats;
	(
		path: PathLike,
		options?: StatSyncOptions
	): IStats | IBigIntStats | undefined;
};

export type RealPath = {
	(path: PathLike, options: EncodingOption, callback: StringCallback): void;
	(
		path: PathLike,
		options: BufferEncodingOption,
		callback: BufferCallback
	): void;
	(
		path: PathLike,
		options: EncodingOption,
		callback: StringOrBufferCallback
	): void;
	(path: PathLike, callback: StringCallback): void;
};

export type RealPathSync = {
	(path: PathLike, options?: EncodingOption): string;
	(path: PathLike, options: BufferEncodingOption): Buffer;
	(path: PathLike, options?: EncodingOption): string | Buffer;
};

export type ReadJson = (
	path: PathOrFileDescriptor,
	callback: ReadJsonCallback
) => void;

export type ReadJsonSync = (path: PathOrFileDescriptor) => JsonObject;

export type Purge = (files?: string | string[] | Set<string>) => void;

export type InputFileSystem = {
	readFile: ReadFile;
	readFileSync?: ReadFileSync;
	readlink: Readlink;
	readlinkSync?: ReadlinkSync;
	readdir: Readdir;
	readdirSync?: ReaddirSync;
	stat: Stat;
	statSync?: StatSync;
	lstat?: LStat;
	lstatSync?: LStatSync;
	realpath?: RealPath;
	realpathSync?: RealPathSync;
	readJson?: ReadJson;
	readJsonSync?: ReadJsonSync;
	purge?: Purge;
	join?: (path1: string, path2: string) => string;
	relative?: (from: string, to: string) => string;
	dirname?: (path: string) => string;
};

export type IntermediateFileSystem = InputFileSystem &
	OutputFileSystem &
	IntermediateFileSystemExtras;

export type WriteStreamOptions = {
	flags?: string;
	encoding?:
		| "ascii"
		| "utf8"
		| "utf-8"
		| "utf16le"
		| "utf-16le"
		| "ucs2"
		| "ucs-2"
		| "latin1"
		| "binary"
		| "base64"
		| "base64url"
		| "hex";
	fd?: any;
	mode?: number;
};

export type MakeDirectoryOptions = {
	recursive?: boolean;
	mode?: string | number;
};

export type MkdirSync = (
	path: PathLike,
	options: MakeDirectoryOptions
) => undefined | string;

export type ReadAsyncOptions<TBuffer extends ArrayBufferView = Buffer> = {
	offset?: number;
	length?: number;
	position?: null | number | bigint;
	buffer?: TBuffer;
};

export type Read<TBuffer extends ArrayBufferView = Buffer> = (
	fd: number,
	options: ReadAsyncOptions<TBuffer>,
	callback: (
		err: null | NodeJS.ErrnoException,
		bytesRead: number,
		buffer: TBuffer
	) => void
) => void;

export type WriteAsyncOptions<TBuffer extends ArrayBufferView = Buffer> = {
	offset?: number;
	length?: number;
	position?: null | number | bigint;
	buffer?: TBuffer;
};

export type Write<TBuffer extends ArrayBufferView = Buffer> = (
	fd: number,
	content: Buffer,
	options: WriteAsyncOptions<TBuffer>,
	callback: (
		err: null | NodeJS.ErrnoException,
		bytesWrite: number,
		buffer: TBuffer
	) => void
) => void;

export type Open = (
	file: PathLike,
	flags: undefined | string | number,
	callback: (arg0: null | NodeJS.ErrnoException, arg1?: number) => void
) => void;

export type IntermediateFileSystemExtras = {
	rename: (
		arg0: PathLike,
		arg1: PathLike,
		arg2: (arg0: null | NodeJS.ErrnoException) => void
	) => void;
	mkdirSync: MkdirSync;
	write: Write<Buffer>;
	open: Open;
	read: Read<Buffer>;
	close: (
		arg0: number,
		arg1: (arg0: null | NodeJS.ErrnoException) => void
	) => void;
};

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
					for (const file of files!) {
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
					}
				}
			});
		} else {
			fs.unlink(p, callback);
		}
	});
}

const join = (fs: OutputFileSystem, rootPath: string, filename: string) => {
	if (fs?.join) {
		return fs.join(rootPath, filename);
	}
	if (path.posix.isAbsolute(rootPath)) {
		return path.posix.join(rootPath, filename);
	}
	if (path.win32.isAbsolute(rootPath)) {
		return path.win32.join(rootPath, filename);
	}
	throw new Error(
		`${rootPath} is neither a posix nor a windows path, and there is no 'join' method defined in the file system`
	);
};

const dirname = (fs: OutputFileSystem, absPath: string) => {
	if (fs?.dirname) {
		return fs.dirname(absPath);
	}
	if (path.posix.isAbsolute(absPath)) {
		return path.posix.dirname(absPath);
	}
	if (path.win32.isAbsolute(absPath)) {
		return path.win32.dirname(absPath);
	}
	throw new Error(
		`${absPath} is neither a posix nor a windows path, and there is no 'dirname' method defined in the file system`
	);
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
			}
			if (err.code === "EEXIST") {
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
