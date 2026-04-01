/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should preserve providedExports for cyclic reexports",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/flag-dependency-exports/entry",
			optimization: {
				providedExports: true,
				usedExports: false,
				inlineExports: false,
				concatenateModules: false
			}
		};
	},
	async check(stats) {
		const json = stats.toJson({
			all: false,
			modules: true,
			providedExports: true,
			nestedModules: false
		});
		const modules = new Map(
			(json.modules || []).map(module => [module.name, module.providedExports])
		);

		expect(modules.get("./fixtures/flag-dependency-exports/entry.js")).toEqual(["a", "b"]);
		expect(modules.get("./fixtures/flag-dependency-exports/a.js")).toEqual(["a", "b"]);
		expect(modules.get("./fixtures/flag-dependency-exports/b.js")).toEqual(["a", "b"]);
	}
};
