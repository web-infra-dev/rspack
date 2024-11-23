/** @type {import('../../dist').TStatsAPICaseConfig} */
module.exports = {
	description: "should output the chunks",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/chunk-b"
		};
	},
	async check(stats) {
		expect(
			stats?.toJson({
				chunks: true,
				timings: false,
				builtAt: false,
				version: false,
				modulesSpace: 3
			}).chunks
		).toMatchInlineSnapshot(`undefined`);
	}
};
