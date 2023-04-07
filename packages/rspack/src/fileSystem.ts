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
		rm: dir =>
			fs.rmSync(dir, {
				recursive: true
			})
	};
}

export { createThreadsafeNodeFSFromRaw };
