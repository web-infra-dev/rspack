import path from "node:path";

const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
const deps = paths => Object.assign([...paths], { added: paths, removed: [] });

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should keep the caller's entries over native absent or existence-only ones in collectTimeInfoEntries",
	options() {
		return { experiments: { nativeWatcher: true } };
	},
	async build(context, compiler) {
		const fs = compiler.watchFileSystem;
		const missingFile = path.join(compiler.context, "time-info-missing.js");
		const missingContext = path.join(compiler.context, "time-info-missing-dir");
		const handle = fs.watch(
			deps([]),
			deps([missingContext]),
			deps([missingFile]),
			Date.now(),
			{ aggregateTimeout: 10000 },
			() => {},
			() => {}
		);
		try {
			const own = new Map();
			for (let i = 0; i < 100 && !own.has(missingFile); i++) {
				await wait(20);
				fs.collectTimeInfoEntries(own, new Map());
			}
			expect(own.get(missingFile)).toBeNull();

			const kept = { safeTime: 1 };
			const files = new Map([[missingFile, kept]]);
			const directories = new Map([[missingContext, kept]]);
			fs.collectTimeInfoEntries(files, directories);
			expect(files.get(missingFile)).toBe(kept);
			expect(directories.get(missingContext)).toBe(kept);
		} finally {
			handle.close();
		}
	}
};
