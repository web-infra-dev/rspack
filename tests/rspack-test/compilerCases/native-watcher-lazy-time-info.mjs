import path from "node:path";

const deps = paths => Object.assign([...paths], { added: paths, removed: [] });

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
const cases = [false, true].map(compact => ({
	description: compact
		? "should transfer absent native timestamps as paths and retain the snapshot after close"
		: "should materialize native timestamp Maps independently and only once",
	options() {
		return { experiments: { nativeWatcher: true } };
	},
	async build(context, compiler) {
		const fs = compiler.watchFileSystem;
		const file = path.join(compiler.context, "watched-file.txt");
		const missing = path.join(compiler.context, "lazy-time-info-missing.js");
		const missingDirectory = path.join(compiler.context, "lazy-time-info-missing-dir");
		const options = { aggregateTimeout: 10000 };
		const handle = fs.watch(
			deps([file]), deps([compiler.context, missingDirectory]), deps([missing]),
			Date.now(), options, () => {}, () => {}
		);
		const native = fs.getNativeWatcher(options);
		const collect = native.collectTimeInfoEntries.bind(native);
		try {
			for (let i = 0; i < 100; i++) {
				const entries = fs.getTimeInfoEntries();
				if (entries.has(missing) && entries.get(file)?.timestamp) break;
				await new Promise(resolve => setTimeout(resolve, 20));
			}
			if (compact) {
				const snapshot = collect();
				await native.close();
				expect(snapshot.fileTimestamps).toBe(snapshot.fileTimestamps);
				expect(snapshot.directoryTimestamps).toBe(snapshot.directoryTimestamps);
				expect(snapshot.fileTimestamps.missing).toContain(missing);
				expect(snapshot.directoryTimestamps.missing).toContain(missingDirectory);
				expect(snapshot.fileTimestamps.entries.find(row => row.path === file)).toMatchObject({
					path: file, safeTime: expect.any(Number), timestamp: expect.any(Number)
				});
				return;
			}
			let filesRead = 0;
			let directoriesRead = 0;
			native.collectTimeInfoEntries = () => {
				const snapshot = collect();
				return {
					get fileTimestamps() {
						filesRead++;
						return snapshot.fileTimestamps;
					},
					get directoryTimestamps() {
						directoriesRead++;
						return snapshot.directoryTimestamps;
					}
				};
			};
			const info = handle.getInfo();
			expect(filesRead).toBe(0);
			expect(directoriesRead).toBe(0);
			const files = info.fileTimeInfoEntries;
			expect(files.get(file)).toHaveProperty("timestamp");
			expect(files.get(missing)).toBeNull();
			expect(info.fileTimeInfoEntries).toBe(files);
			expect(filesRead).toBe(1);
			expect(directoriesRead).toBe(0);
			await native.close();
			const directories = info.contextTimeInfoEntries;
			expect(directories.get(compiler.context)).toHaveProperty("safeTime");
			expect(directories.get(missingDirectory)).toBeNull();
			expect(info.contextTimeInfoEntries).toBe(directories);
			expect(directoriesRead).toBe(1);
		} finally {
			native.collectTimeInfoEntries = collect;
			handle.close();
		}
	}
}));

cases.push({
	description: "should keep a discovered directory over an earlier missing timestamp for the same path",
	options() {
		return { experiments: { nativeWatcher: true } };
	},
	async build(context, compiler) {
		const fs = compiler.watchFileSystem;
		const root = compiler.context;
		const directory = path.join(root, "subdir");
		let observe;
		const observed = new Promise(resolve => { observe = resolve; });
		const handle = fs.watch(
			deps([]), deps([root]), deps([directory]), Date.now(),
			{
				aggregateTimeout: 10000,
				ignored(entry) {
					if (entry === path.join(directory, "watched-file.txt")) {
						observe(handle.getInfo().fileTimeInfoEntries.get(directory));
					}
					return false;
				}
			}, () => {}, () => {}
		);
		try {
			expect(await observed).toEqual({});
		} finally {
			handle.close();
		}
	}
});

export default cases;
