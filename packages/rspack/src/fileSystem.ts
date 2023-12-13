import util from "util";
import { join } from "path";

export interface ThreadsafeWritableNodeFS {
	writeFile: (...args: any[]) => any;
	removeFile: (...args: any[]) => any;
	mkdir: (...args: any[]) => any;
	mkdirp: (...args: any[]) => any;
	removeDirAll: (...args: any[]) => any;
}

function createThreadsafeNodeFSFromRaw(
	fs: typeof import("fs")
): ThreadsafeWritableNodeFS {
	return {
		writeFile: (file, data) => {
			return util.promisify(fs.writeFile.bind(fs))(file, data);
		},
		removeFile: file => {
			return util.promisify(fs.unlink.bind(fs))(file);
		},
		mkdir: dir => {
			return util.promisify(fs.mkdir.bind(fs))(dir);
		},
		mkdirp: dir => {
			return util.promisify(fs.mkdir.bind(fs))(dir, {
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
			for (const file of files) {
				const filePath = join(dir, file);
				if ((await util.promisify(fs.lstat.bind(fs))(filePath)).isDirectory()) {
					await rmrf(filePath);
				} else {
					await util.promisify(fs.unlink.bind(fs))(filePath);
				}
			}
			await util.promisify(fs.rmdir.bind(fs))(dir);
		}
	};
	return rmrf;
};

export { createThreadsafeNodeFSFromRaw };
