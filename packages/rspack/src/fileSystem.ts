import util from "util";
import { join } from "path";
import { memoizeFn } from "./util/memoize";
import { InputFileSystem } from "./util/fs";

export interface ThreadsafeWritableNodeFS {
	writeFile: (...args: any[]) => any;
	removeFile: (...args: any[]) => any;
	mkdir: (...args: any[]) => any;
	mkdirp: (...args: any[]) => any;
	removeDirAll: (...args: any[]) => any;
}

export interface ThreadsafeReadableNodeFS {
	read(path: string): PromiseLike<Buffer>;
}

export function createThreadsafeReadableNodeFS(
	inputFileSystem: Pick<InputFileSystem, "readFile">
): ThreadsafeReadableNodeFS {
	const asyncFS = {
		readFile: memoizeFn(() =>
			util.promisify(inputFileSystem.readFile.bind(inputFileSystem))
		)
	};
	return {
		read: (path: string) =>
			asyncFS.readFile(path).then(a => Buffer.from(a || ""))
	};
}

function createThreadsafeNodeFSFromRaw(
	fs: typeof import("fs")
): ThreadsafeWritableNodeFS {
	let writeFile = memoizeFn(() => util.promisify(fs.writeFile.bind(fs)));
	let removeFile = memoizeFn(() => util.promisify(fs.unlink.bind(fs)));
	let mkdir = memoizeFn(() => util.promisify(fs.mkdir.bind(fs)));
	return {
		writeFile,
		removeFile,
		mkdir,
		mkdirp: dir => {
			return mkdir(dir, {
				recursive: true
			});
		},
		removeDirAll: dir => {
			// memfs don't support rmSync
			return rmrfBuild(fs)(dir);
		}
	};
}

const rmrfBuild = (fs: typeof import("fs")) => {
	async function exists(path: string) {
		try {
			await util.promisify(fs.access.bind(fs))(path);
			return true;
		} catch {
			return false;
		}
	}
	const rmrf = async (dir: string) => {
		if (await exists(dir)) {
			const files = await util.promisify(fs.readdir.bind(fs))(dir);
			await Promise.all(
				files
					.map(f => join(dir, f))
					.map(async filePath => {
						if (
							(await util.promisify(fs.lstat.bind(fs))(filePath)).isDirectory()
						) {
							await rmrf(filePath);
						} else {
							await util.promisify(fs.unlink.bind(fs))(filePath);
						}
					})
			);
			await util.promisify(fs.rmdir.bind(fs))(dir);
		}
	};
	return rmrf;
};

export { createThreadsafeNodeFSFromRaw };
