const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have build time executed",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/css/index",
			module: {
				rules: [
					{
						type: "javascript/auto",
						test: /\.css$/,
						use: [CssExtractRspackPlugin.loader, "css-loader"]
					}
				]
			},
			plugins: [
				new CssExtractRspackPlugin({
					filename: "[name].css"
				})
			],
		};
	},
	async check(stats) {
		const statsOptions = {
			modules: true,
			runtimeModules: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const runtimeModules = statsJson.modules.filter(
			m => m.buildTimeExecuted && m.identifier.startsWith("webpack/runtime/")
		);
		expect(runtimeModules.length).toBe(4);
		const runtimeModuleIds = runtimeModules.map(i => i.identifier);
		runtimeModuleIds.sort();
		expect(runtimeModuleIds).toMatchInlineSnapshot(`
		Array [
		  webpack/runtime/compat_get_default_export,
		  webpack/runtime/define_property_getters,
		  webpack/runtime/has_own_property,
		  webpack/runtime/make_namespace_object,
		]
		`);
	}
};
