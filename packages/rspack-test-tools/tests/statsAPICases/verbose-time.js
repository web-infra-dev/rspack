/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have time log when logging verbose",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc"
		};
	},
	async check(stats) {
		expect(
			stats
				?.toString({ all: false, logging: "verbose" })
				.replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`undefined`);
	}
};
