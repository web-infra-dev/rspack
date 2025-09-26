defineStatsAPICase(Utils.basename(__filename), {
	description: "should look not bad for default stats toString",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./abc"
		};
	},
	async check(stats) {
		expect(stats?.toString({ timings: false, version: false }))
			.toMatchInlineSnapshot(`
		asset main.js 344 bytes [emitted] (name: main)
		./abc.js 83 bytes [built] [code generated]
		./a.js 55 bytes [built] [code generated]
		./b.js 94 bytes [built] [code generated]
		./c.js 72 bytes [built] [code generated]
		Rspack compiled successfully
	`);
	}
});
