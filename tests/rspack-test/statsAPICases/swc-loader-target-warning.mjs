/** @type {import("@rspack/test-tools").TStatsAPICaseConfig} */
export default {
	description: "should identify the SWC loader when target options conflict",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			module: {
				rules: [
					{
						test: /\.js$/,
						loader: "builtin:swc-loader",
						options: { env: { targets: "chrome 90" }, jsc: { target: "es2020" } }
					}
				]
			}
		};
	},
	check(stats) {
		const { warnings } = stats.toJson({ all: false, warnings: true });
		expect(warnings).toMatchObject([
			{
				code: "builtin:swc-loader",
				message: expect.stringContaining(
					"⚠ builtin:swc-loader: `env` and `jsc.target` cannot be set together. Remove one of them."
				)
			}
		]);
	}
};
