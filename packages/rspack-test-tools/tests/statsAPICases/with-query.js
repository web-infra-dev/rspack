module.exports = {
	description: "should output stats with query",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc-query"
		};
	},
	async check(stats) {
		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
	}
};
