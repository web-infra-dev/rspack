/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should look not bad for default stats toString",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc"
		};
	},
	async check(stats) {
		expect(
			stats?.toString({ timings: false, version: false })
		).toMatchInlineSnapshot(`undefined`);
	}
};
