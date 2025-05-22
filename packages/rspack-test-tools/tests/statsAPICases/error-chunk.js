/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
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
		const string = stats.toString({
			errors: true
		});
		expect(string).toContain(`ERROR in × Entrypoints 'b' and 'a' use 'dependOn' to depend on each other in a circular way.`);
	}
};
