/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should generate chunk group asset",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/order/index"
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

		const string = stats.toString(statsOptions);

		// entrypoints
		expect(string).toContain(`Entrypoint main 13.4 KiB (15.6 KiB) = main.js 13.4 KiB (main.js.map 15.6 KiB)`);
		expect(string).toContain(`prefetch: chunk.js 827 bytes {411} (name: chunk) (chunk.js.map 510 bytes)`);

		// chunk groups
		expect(string).toContain(`Chunk Group chunk 827 bytes (510 bytes) = chunk.js 827 bytes (chunk.js.map 510 bytes)`);
		expect(string).toContain(`preload: chunk-b.js 126 bytes {276} (name: chunk-b)`);
		expect(string).toContain(`prefetch: chunk-c.js 125 bytes {467} (name: chunk-c), chunk-a.js 126 bytes {181} (name: chunk-a)`);
		expect(string).toContain(`Chunk Group chunk-a 126 bytes = chunk-a.js`);
		expect(string).toContain(`Chunk Group chunk-b 126 bytes = chunk-b.js`);
		expect(string).toContain(`Chunk Group chunk-c 125 bytes = chunk-c.js`);
	}
};
