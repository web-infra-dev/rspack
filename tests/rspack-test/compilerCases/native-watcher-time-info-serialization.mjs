import path from "node:path";

const deps = paths => Object.assign([...paths], { added: paths, removed: [] });

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "should preserve escaped paths and optional fields in native timestamp snapshots",
	options() {
		return { experiments: { nativeWatcher: true } };
	},
	async build(context, compiler) {
		const fs = compiler.watchFileSystem;
		const file = path.join(compiler.context, "watched-file.txt");
		const missing = path.join(compiler.context, 'missing-中文-"-\\-\n.js');
		const options = { aggregateTimeout: 10000 };
		const handle = fs.watch(
			deps([file]), deps([compiler.context]), deps([missing]),
			Date.now(), options, () => {}, () => {}
		);
		try {
			for (let i = 0; i < 100; i++) {
				const entries = fs.getTimeInfoEntries();
				if (entries.has(missing) && entries.get(file)?.timestamp) break;
				await new Promise(resolve => setTimeout(resolve, 20));
			}
			const snapshot = fs.getNativeWatcher(options).collectTimeInfoEntries();
			const fileRow = snapshot.fileTimestamps.entries.find(row => row.path === file);
			expect(fileRow).toEqual({
				path: file,
				existenceOnly: false,
				safeTime: expect.any(Number),
				timestamp: expect.any(Number),
				accuracy: expect.any(Number)
			});
			expect(Object.keys(fileRow)).toEqual([
				"path", "existenceOnly", "safeTime", "timestamp", "accuracy"
			]);
			const existenceRow = snapshot.fileTimestamps.entries.find(row => row.path === compiler.context);
			expect(existenceRow).toEqual({ path: compiler.context, existenceOnly: true });
			const directoryRow = snapshot.directoryTimestamps.entries.find(row => row.path === compiler.context);
			expect(directoryRow).toEqual({
				path: compiler.context, existenceOnly: false, safeTime: expect.any(Number)
			});
			expect(snapshot.fileTimestamps.missing).toContain(missing);
			const info = handle.getInfo();
			expect(info.fileTimeInfoEntries.has(missing)).toBe(true);
			expect(info.fileTimeInfoEntries.get(missing)).toBeNull();
			expect(info.fileTimeInfoEntries.get(file)).toEqual({
				safeTime: fileRow.safeTime, timestamp: fileRow.timestamp, accuracy: fileRow.accuracy
			});
		} finally {
			handle.close();
		}
	}
};
