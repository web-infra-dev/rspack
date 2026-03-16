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
			experiments: {
				css: false
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			modules: true,
			runtimeModules: false,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const executedModules = statsJson.modules.filter(i => i.buildTimeExecuted);
		expect(executedModules.length).toBe(3);
		const executedModuleIds = executedModules.map(i => i.identifier);
		executedModuleIds.sort();
		expect(executedModuleIds).toMatchInlineSnapshot(`
			Array [
			  <HOME>/Library/pnpm/store/v10/links/css-loader/7.1.4/323015575a54040cbf40278ae679153ab56eb5e1420ff2b5c318a5bda78f1d0e/node_modules/css-loader/dist/cjs.js!<TEST_ROOT>/fixtures/css/style.css,
			  <HOME>/Library/pnpm/store/v10/links/css-loader/7.1.4/323015575a54040cbf40278ae679153ab56eb5e1420ff2b5c318a5bda78f1d0e/node_modules/css-loader/dist/runtime/api.js,
			  <HOME>/Library/pnpm/store/v10/links/css-loader/7.1.4/323015575a54040cbf40278ae679153ab56eb5e1420ff2b5c318a5bda78f1d0e/node_modules/css-loader/dist/runtime/noSourceMaps.js,
			]
		`);
	}
};
