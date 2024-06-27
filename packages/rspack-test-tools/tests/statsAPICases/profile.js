/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have module profile when profile is true",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc",
			profile: true
		};
	},
	async check(stats) {
		expect(
			stats?.toString({ all: false, modules: true }).replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		"./fixtures/abc.js 83 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./fixtures/a.js 55 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./fixtures/b.js 94 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)
		./fixtures/c.js 72 bytes [built] [code generated]
		  X ms (resolving: X ms, building: X ms)"
	`);
	}
};
