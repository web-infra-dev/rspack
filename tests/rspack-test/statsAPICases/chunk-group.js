/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should generate chunk group asset",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/order/index"
			},
			output: {
				environment: {
					methodShorthand: false
				}
			},
			optimization: {
				minimize: false
			},
			devtool: "source-map"
		};
	},
	async check(stats) {
		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};

		const string = stats.toString(statsOptions).replace(/[\d.]+ (KiB|bytes)/g, "X.X $1");

		// entrypoints
		expect(string).toContain(`Entrypoint main X.X KiB (X.X KiB) = main.js X.X KiB (main.js.map X.X KiB)`);
		expect(string).toContain(`prefetch: chunk.js X.X bytes {411} (name: chunk) (chunk.js.map X.X bytes)`);

		// chunk groups
		expect(string).toContain(`Chunk Group chunk X.X bytes (X.X bytes) = chunk.js X.X bytes (chunk.js.map X.X bytes)`);
		expect(string).toContain(`preload: chunk-b.js X.X bytes {276} (name: chunk-b)`);
		expect(string).toContain(`prefetch: chunk-c.js X.X bytes {467} (name: chunk-c), chunk-a.js X.X bytes {181} (name: chunk-a)`);
		expect(string).toContain(`Chunk Group chunk-a X.X bytes = chunk-a.js`);
		expect(string).toContain(`Chunk Group chunk-b X.X bytes = chunk-b.js`);
		expect(string).toContain(`Chunk Group chunk-c X.X bytes = chunk-c.js`);
	}
};
