/** @type {import('../../dist').TStatsAPICaseConfig} */
module.exports = {
	description: "should output error chunk info",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				a: {
					import: "./fixtures/a",
					dependOn: "b"
				},
				b: {
					import: "./fixtures/b",
					dependOn: "a"
				}
			}
		};
	},
	async check(stats) {
		expect(
			stats?.toJson({
				errors: true
			}).errors
		).toMatchInlineSnapshot(`undefined`);
	}
};
