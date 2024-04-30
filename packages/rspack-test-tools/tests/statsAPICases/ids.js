/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have ids when ids is true",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a"
		};
	},
	async check(stats) {
		const options = {
			all: false,
			assets: true,
			modules: true,
			chunks: true,
			ids: true
		};
		expect(stats?.toJson(options)).toMatchSnapshot();
		expect(stats?.toString(options)).toMatchInlineSnapshot(`
		"asset main.js 211 bytes {909} [emitted] (name: main)
		chunk {909} main.js (main) [entry]
		./fixtures/a.js [585] {909}"
	`);
	}
};
