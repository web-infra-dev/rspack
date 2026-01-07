/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should inject bundler info runtime modules",
	options(context) {
		return {
			context: context.getSource(),
			output: {
				bundlerInfo: {
					force: true
				}
			},
			entry: "./fixtures/index",
		};
	},
	async check(stats) {
		const statsOptions = {
			runtimeModules: true
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const runtimeModules = statsJson.modules.filter(m => m.identifier.startsWith("webpack/runtime/")).map(i => i.identifier).filter(Boolean);
		expect(runtimeModules).toContain("webpack/runtime/rspack_unique_id");
		expect(runtimeModules).toContain("webpack/runtime/rspack_version");
	}
};
