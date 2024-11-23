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
		).toMatchInlineSnapshot(`undefined`);
	}
};
