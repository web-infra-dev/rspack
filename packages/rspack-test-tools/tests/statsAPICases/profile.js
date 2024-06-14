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
		"./fixtures/abc.js [built] [code generated]
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/a.js [built] [code generated]
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/b.js [built] [code generated]
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/c.js [built] [code generated]
		  X ms (resolving: X ms, integration: X ms, building: X ms)"
	`);
	}
};
