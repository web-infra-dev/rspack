/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description:
		"should output the specified number of modules when set stats.modulesSpace",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc"
		};
	},
	async check(stats) {
		expect(
			stats?.toJson({
				all: true,
				timings: false,
				builtAt: false,
				version: false
			}).modules?.length
		).toBe(4);

		expect(
			stats?.toJson({
				all: true,
				timings: false,
				builtAt: false,
				version: false,
				modulesSpace: 3
			}).modules?.length
			// 2 = 3 - 1 = max - filteredChildrenLineReserved
		).toBe(2);
	}
};
