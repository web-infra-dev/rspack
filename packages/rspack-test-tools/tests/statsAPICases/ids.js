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
		"asset main.js 206 bytes {909} [emitted] (name: main)
		chunk {909} (runtime: main) main.js (main) 55 bytes [entry] [rendered]
		./fixtures/a.js [585] 55 bytes {909} [built] [code generated]"
	`);
	}
};
