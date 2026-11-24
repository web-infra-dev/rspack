/** @type {import("@rspack/test-tools").TStatsAPICaseConfig} */
export default {
	description: "should identify the ESM library plugin when a split chunk name already exists",
	options(context) {
		return {
			context: context.getSource(),
			mode: "production",
			entry: "./fixtures/a",
			output: { module: true, library: { type: "modern-module" } },
			optimization: {
				splitChunks: { cacheGroups: { shared: { test: /a\.js$/, name: "main" } } }
			}
		};
	},
	check(stats) {
		expect(stats.hasErrors()).toBe(false);
		expect(stats.toString({ all: false, warnings: true, colors: false })).toContain(
			"WARNING in ⚠ EsmLibraryPlugin: Merge modules into an existing chunk: main"
		);
	}
};
