defineStatsAPICase(Utils.basename(__filename), {
	description: "should have module profile when profile is true",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./abc",
			profile: true
		};
	},
	async check(stats) {
		expect(
			stats?.toString({ all: false, modules: true }).replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		./abc.js 83 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./a.js 55 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./b.js 94 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./c.js 72 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
	`);
	}
});
