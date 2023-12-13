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
		writeFile: (file, data) =>
			util.promisify(fs.writeFile.bind(fs))(file, data),
		removeFile: file => util.promisify(fs.unlink.bind(fs))(file),
		mkdir: dir => util.promisify(fs.mkdir.bind(fs))(dir),
		mkdirp: dir =>
			util.promisify(fs.mkdir.bind(fs))(dir, {
				recursive: true
			}),
		removeDirAll: dir => {
			// memfs don't support rmSync
			return rmrfBuild(fs)(dir);
		}
	};
}

const rmrfBuild = (fs: typeof import("fs")) => {
	async function exists(path: string) {
		try {
			await fs.promises.access(path);
			return true;
		} catch {
			return false;
		}
	}
	const rmrf = async (dir: string) => {
		if (await exists(dir)) {
			const files = await fs.promises.readdir(dir);
			for (const file of files) {
				const filePath = join(dir, file);
				if ((await fs.promises.lstat(filePath)).isDirectory()) {
					await rmrf(filePath);
				} else {
					await fs.promises.unlink(filePath);
				}
			}
			await fs.promises.rmdir(dir);
		}
	};
	return rmrf;
};

export { createThreadsafeNodeFSFromRaw };
