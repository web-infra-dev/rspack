const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { rspack } = require("@rspack/core");

describe("Watching dependency registration", () => {
	it("snapshots completed dependency deltas before the watch handler", async () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-watch-deps-"));
		const entry = path.join(root, "entry.js");
		const trackedFile = path.join(root, "tracked.js");
		const trackedContext = path.join(root, "tracked-context");
		const trackedMissing = path.join(root, "missing.js");
		fs.writeFileSync(entry, "export default true;\n");

		let registration;
		let resolveRegistration;
		const registered = new Promise(resolve => {
			resolveRegistration = resolve;
		});
		const compiler = rspack({
			context: root,
			entry: "./entry.js",
			mode: "development",
			output: { filename: "bundle.js", path: path.join(root, "dist") },
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.done.tap("watch-dependency-regression", stats => {
							for (const [key, value] of [
								["__internal__addedFileDependencies", trackedFile],
								["__internal__addedContextDependencies", trackedContext],
								["__internal__addedMissingDependencies", trackedMissing]
							]) {
								Object.defineProperty(stats.compilation, key, {
									configurable: true,
									value: [value]
								});
							}
						});
					}
				}
			]
		});
		compiler.watchFileSystem = {
			watch(files, contexts, missing) {
				registration = {
					file: [...(files.added ?? [])],
					context: [...(contexts.added ?? [])],
					missing: [...(missing.added ?? [])]
				};
				resolveRegistration();
				return {
					close() {},
					pause() {},
					getInfo() {
						return {
							changes: new Set(),
							removals: new Set(),
							fileTimeInfoEntries: new Map(),
							contextTimeInfoEntries: new Map()
						};
					}
				};
			}
		};

		let watching;
		try {
			await new Promise((resolve, reject) => {
				watching = compiler.watch({}, (error, stats) => {
					if (error || stats.hasErrors()) {
						reject(
							error ?? new Error(stats.toString({ all: false, errors: true }))
						);
						return;
					}
					for (const key of [
						"__internal__addedFileDependencies",
						"__internal__addedContextDependencies",
						"__internal__addedMissingDependencies"
					]) {
						Object.defineProperty(stats.compilation, key, {
							configurable: true,
							value: []
						});
					}
					resolve();
				});
			});
			await registered;
			expect(registration.file).toContain(trackedFile);
			expect(registration.context).toContain(trackedContext);
			expect(registration.missing).toContain(trackedMissing);
		} finally {
			if (watching) await new Promise(resolve => watching.close(resolve));
			fs.rmSync(root, { force: true, recursive: true });
		}
	});
});
