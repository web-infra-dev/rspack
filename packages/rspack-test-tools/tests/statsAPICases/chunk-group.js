/** @type {import('../..').TStatsAPICaseConfig} */
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
		expect(string).toContain(`Entrypoint main 13.6 KiB (15.7 KiB) = main.js 13.6 KiB (main.js.map 15.7 KiB)`);
		expect(string).toContain(`prefetch: chunk.js 812 bytes {919} (name: chunk) (chunk.js.map 514 bytes)`);

		// chunk groups
		expect(string).toContain(`Chunk Group chunk 812 bytes (514 bytes) = chunk.js 812 bytes (chunk.js.map 514 bytes)`);
		expect(string).toContain(`preload: chunk-b.js 134 bytes {805} (name: chunk-b)`);
		expect(string).toContain(`prefetch: chunk-c.js 134 bytes {212} (name: chunk-c), chunk-a.js 134 bytes {807} (name: chunk-a)`);
		expect(string).toContain(`Chunk Group chunk-a 134 bytes = chunk-a.js`);
		expect(string).toContain(`Chunk Group chunk-b 134 bytes = chunk-b.js`);
		expect(string).toContain(`Chunk Group chunk-c 134 bytes = chunk-c.js`);
	}
};
