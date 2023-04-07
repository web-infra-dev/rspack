export interface ThreadsafeWritableNodeFS {
	writeFile: (...args: any[]) => any;
	mkdir: (...args: any[]) => any;
	mkdirp: (...args: any[]) => any;
	remove_dir_all: (...args: any[]) => any;
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
		remove_dir_all: dir =>
			fs.rmSync(dir, {
				recursive: true
			})
	};
}

export { createThreadsafeNodeFSFromRaw };
