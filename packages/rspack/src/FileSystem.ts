import util from "node:util";
import type { NodeFsStats, ThreadsafeNodeFS } from "@rspack/binding";

import { type IStats, type OutputFileSystem, mkdirp, rmrf } from "./util/fs";
import { memoizeFn } from "./util/memoize";

const NOOP_FILESYSTEM: ThreadsafeNodeFS = {
	writeFile() {},
	removeFile() {},
	mkdir() {},
	mkdirp() {},
	removeDirAll() {},
	readDir: () => {},
	readFile: () => {},
	stat: () => {},
	lstat: () => {}
};

class ThreadsafeWritableNodeFS implements ThreadsafeNodeFS {
	writeFile!: (name: string, content: Buffer) => Promise<void> | void;
	removeFile!: (name: string) => Promise<void> | void;
	mkdir!: (name: string) => Promise<void> | void;
	mkdirp!: (name: string) => Promise<string | void> | string | void;
	removeDirAll!: (name: string) => Promise<string | void> | string | void;
	readDir!: (name: string) => Promise<string[] | void> | string[] | void;
	readFile!: (
		name: string
	) => Promise<Buffer | string | void> | Buffer | string | void;
	stat!: (name: string) => Promise<NodeFsStats | void> | NodeFsStats | void;
	lstat!: (name: string) => Promise<NodeFsStats | void> | NodeFsStats | void;

	constructor(fs?: OutputFileSystem) {
		if (!fs) {
			// This happens when located in a child compiler.
			Object.assign(this, NOOP_FILESYSTEM);
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
				return (
					res && {
						isFile: res.isFile(),
						isDirectory: res.isDirectory(),
						atimeMs: res.atimeMs,
						mtimeMs: res.atimeMs,
						ctimeMs: res.atimeMs,
						birthtimeMs: res.birthtimeMs,
						size: res.size
					}
				);
			};
		});
		this.lstat = memoizeFn(() => {
			const statFn = util.promisify((fs.lstat || fs.stat).bind(fs));
			return async (filePath: string) => {
				const res = await statFn(filePath);
				return res && ThreadsafeWritableNodeFS.__to_binding_stat(res);
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
			atimeMs: stat.atimeMs,
			mtimeMs: stat.atimeMs,
			ctimeMs: stat.atimeMs,
			birthtimeMs: stat.birthtimeMs,
			size: stat.size
		};
	}
}

export { ThreadsafeWritableNodeFS };
