import util from "util";
import type { ThreadsafeNodeFS } from "@rspack/binding";

import { type OutputFileSystem, mkdirp, rmrf } from "./util/fs";
import { memoizeFn } from "./util/memoize";

const NOOP_FILESYSTEM: ThreadsafeNodeFS = {
	writeFile() {},
	removeFile() {},
	mkdir() {},
	mkdirp() {},
	removeDirAll() {}
};

class ThreadsafeWritableNodeFS implements ThreadsafeNodeFS {
	writeFile!: (name: string, content: Buffer) => Promise<void> | void;
	removeFile!: (name: string) => Promise<void> | void;
	mkdir!: (name: string) => Promise<void> | void;
	mkdirp!: (name: string) => Promise<string | void> | string | void;
	removeDirAll!: (name: string) => Promise<string | void> | string | void;

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
	}

	static __to_binding(fs?: OutputFileSystem) {
		return new this(fs);
	}
}

export { ThreadsafeWritableNodeFS };
