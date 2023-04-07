import { join } from "path";

export interface ThreadsafeWritableNodeFS {
	writeFile: (...args: any[]) => any;
	mkdir: (...args: any[]) => any;
	mkdirp: (...args: any[]) => any;
	rm: (...args: any[]) => any;
}

function createThreadsafeNodeFSFromRaw(
	fs: typeof import("fs")
): ThreadsafeWritableNodeFS {
	return {
		writeFile: (file, data) => fs.writeFileSync(file, data),
		mkdir: dir => fs.mkdirSync(dir),
		mkdirp: dir =>
			fs.mkdirSync(dir, {
				recursive: true
			}),
		rm: dir => {
			// memfs don't support rmSync
			rmrfBuild(fs)(dir);
		}
	};
}

const rmrfBuild = (fs: typeof import("fs")) => {
	const rmrf = (dir: string) => {
		if (fs.existsSync(dir)) {
			const files = fs.readdirSync(dir);
			files.forEach(file => {
				const filePath = join(dir, file);
				if (fs.lstatSync(filePath).isDirectory()) {
					rmrf(filePath);
				} else {
					fs.unlinkSync(filePath);
				}
			});
			fs.rmdirSync(dir);
		}
	};
	return rmrf;
};

export { createThreadsafeNodeFSFromRaw };
