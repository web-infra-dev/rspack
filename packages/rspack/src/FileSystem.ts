import util from "node:util";
import type { NodeFsStats, ThreadsafeNodeFS } from "@rspack/binding";

import {
	type IStats,
	type IntermediateFileSystem,
	type OutputFileSystem,
	mkdirp,
	rmrf
} from "./util/fs";
import { memoizeFn } from "./util/memoize";

const BUFFER_SIZE = 1000;

const ASYNC_NOOP = async () => {};

const NOOP_FILESYSTEM: ThreadsafeNodeFS = {
	writeFile: ASYNC_NOOP,
	removeFile: ASYNC_NOOP,
	mkdir: ASYNC_NOOP,
	mkdirp: ASYNC_NOOP,
	removeDirAll: ASYNC_NOOP,
	readDir: ASYNC_NOOP,
	readFile: ASYNC_NOOP,
	stat: ASYNC_NOOP,
	lstat: ASYNC_NOOP,
	open: ASYNC_NOOP,
	rename: ASYNC_NOOP,
	close: ASYNC_NOOP,
	write: ASYNC_NOOP,
	writeAll: ASYNC_NOOP,
	read: ASYNC_NOOP,
	readUntil: ASYNC_NOOP,
	readToEnd: ASYNC_NOOP
};

class ThreadsafeOutputNodeFS implements ThreadsafeNodeFS {
	writeFile!: (name: string, content: Buffer) => Promise<void>;
	removeFile!: (name: string) => Promise<void>;
	mkdir!: (name: string) => Promise<void>;
	mkdirp!: (name: string) => Promise<string | void>;
	removeDirAll!: (name: string) => Promise<string | void>;
	readDir!: (name: string) => Promise<string[] | void>;
	readFile!: (name: string) => Promise<Buffer | string | void>;
	stat!: (name: string) => Promise<NodeFsStats | void>;
	lstat!: (name: string) => Promise<NodeFsStats | void>;
	open!: (name: string, flags: string) => Promise<number | void>;
	rename!: (from: string, to: string) => Promise<void>;
	close!: (fd: number) => Promise<void>;
	write!: (
		fd: number,
		content: Buffer,
		position: number
	) => Promise<number | void>;
	writeAll!: (fd: number, content: Buffer) => Promise<number | void>;
	read!: (
		fd: number,
		length: number,
		position: number
	) => Promise<Buffer | void>;
	readUntil!: (
		fd: number,
		code: number,
		position: number
	) => Promise<Buffer | void>;
	readToEnd!: (fd: number, position: number) => Promise<Buffer | void>;

	constructor(fs?: OutputFileSystem) {
		Object.assign(this, NOOP_FILESYSTEM);
		if (!fs) {
			return;
		}
		this.writeFile = memoizeFn(() => util.promisify(fs.writeFile.bind(fs)));
		this.removeFile = memoizeFn(() => util.promisify(fs.unlink.bind(fs)));
		this.mkdir = memoizeFn(() => util.promisify(fs.mkdir.bind(fs)));
		this.mkdirp = memoizeFn(() => util.promisify(mkdirp.bind(null, fs)));
		this.removeDirAll = memoizeFn(() => util.promisify(rmrf.bind(null, fs)));
		this.readDir = memoizeFn(() => {
			const readDirFn = util.promisify(fs.readdir.bind(fs));
			return async (filePath: string) => {
				const res = await readDirFn(filePath);
				return res as string[];
			};
		});
		this.readFile = memoizeFn(() => util.promisify(fs.readFile.bind(fs)));
		this.stat = memoizeFn(() => {
			const statFn = util.promisify(fs.stat.bind(fs));
			return async (filePath: string) => {
				const res = await statFn(filePath);
				return res && ThreadsafeOutputNodeFS.__to_binding_stat(res);
			};
		});
		this.lstat = memoizeFn(() => {
			const statFn = util.promisify((fs.lstat || fs.stat).bind(fs));
			return async (filePath: string) => {
				const res = await statFn(filePath);
				return res && ThreadsafeOutputNodeFS.__to_binding_stat(res);
			};
		});
	}

	static __to_binding(fs?: OutputFileSystem) {
		return new this(fs);
	}

	static __to_binding_stat(stat: IStats): NodeFsStats {
		return {
			isFile: stat.isFile(),
			isDirectory: stat.isDirectory(),
			isSymlink: stat.isSymbolicLink(),
			atimeMs: stat.atimeMs,
			mtimeMs: stat.atimeMs,
			ctimeMs: stat.atimeMs,
			birthtimeMs: stat.birthtimeMs,
			size: stat.size
		};
	}
}

class ThreadsafeIntermediateNodeFS extends ThreadsafeOutputNodeFS {
	constructor(fs?: IntermediateFileSystem) {
		super(fs);
		if (!fs) {
			return;
		}
		this.open = memoizeFn(() => util.promisify(fs.open.bind(fs)));
		this.rename = memoizeFn(() => util.promisify(fs.rename.bind(fs)));
		this.close = memoizeFn(() => util.promisify(fs.close.bind(fs)));
		this.write = memoizeFn(() => {
			const writeFn = util.promisify(fs.write.bind(fs));
			return async (fd: number, content: Buffer, position: number) => {
				return await writeFn(fd, content, {
					position
				});
			};
		});
		this.writeAll = memoizeFn(() => {
			const writeFn = util.promisify(fs.writeFile.bind(fs));
			return async (fd: number, content: Buffer) => {
				return await writeFn(fd, content);
			};
		});
		this.read = memoizeFn(() => {
			const readFn = fs.read.bind(fs);
			return async (fd: number, length: number, position: number) => {
				new Promise(resolve => {
					readFn(
						fd,
						{
							position,
							length
						},
						(err, bytesRead, buffer) => {
							if (err) {
								resolve(err);
							} else {
								resolve(buffer);
							}
						}
					);
				});
			};
		});
		this.readUntil = memoizeFn(() => {
			return async (fd: number, delim: number, position: number) => {
				const res: Buffer[] = [];
				let current_position = position;
				while (true) {
					const buffer = await this.read(fd, BUFFER_SIZE, current_position);
					if (!buffer || buffer.length === 0) {
						break;
					}
					const pos = buffer.indexOf(delim);
					if (pos >= 0) {
						res.push(buffer.slice(0, pos));
						break;
					}
					res.push(buffer);
					current_position += buffer.length;
				}
				return Buffer.concat(res);
			};
		});
		this.readToEnd = memoizeFn(() => {
			return async (fd: number, position: number) => {
				const res: Buffer[] = [];
				let current_position = position;
				while (true) {
					const buffer = await this.read(fd, BUFFER_SIZE, current_position);
					if (!buffer || buffer.length === 0) {
						break;
					}
					res.push(buffer);
					current_position += buffer.length;
				}
				return Buffer.concat(res);
			};
		});
	}

	static __to_binding(fs?: IntermediateFileSystem) {
		return new this(fs);
	}
}

export { ThreadsafeOutputNodeFS, ThreadsafeIntermediateNodeFS };
