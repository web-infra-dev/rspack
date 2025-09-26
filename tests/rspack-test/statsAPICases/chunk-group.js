defineStatsAPICase(Utils.basename(__filename), {
	description: "should generate chunk group asset",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./order/index"
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
		expect(string).toContain(`Entrypoint main 13.4 KiB (15.2 KiB) = main.js 13.4 KiB (main.js.map 15.2 KiB)`);
		expect(string).toContain(`prefetch: chunk.js 807 bytes {411} (name: chunk) (chunk.js.map 488 bytes)`);

		// chunk groups
		expect(string).toContain(`Chunk Group chunk 807 bytes (488 bytes) = chunk.js 807 bytes (chunk.js.map 488 bytes)`);
		expect(string).toContain(`preload: chunk-b.js 100 bytes {276} (name: chunk-b)`);
		expect(string).toContain(`prefetch: chunk-c.js 100 bytes {467} (name: chunk-c), chunk-a.js 100 bytes {181} (name: chunk-a)`);
		expect(string).toContain(`Chunk Group chunk-a 100 bytes = chunk-a.js`);
		expect(string).toContain(`Chunk Group chunk-b 100 bytes = chunk-b.js`);
		expect(string).toContain(`Chunk Group chunk-c 100 bytes = chunk-c.js`);
	}
});
