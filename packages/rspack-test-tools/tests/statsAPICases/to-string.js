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
		expect(stats?.toString({ timings: false, version: false }))
			.toMatchInlineSnapshot(`
		"asset main.js 353 bytes [emitted] (name: main)
		Entrypoint main 353 bytes = main.js
		./fixtures/abc.js 83 bytes [built] [code generated]
		./fixtures/a.js 55 bytes [built] [code generated]
		./fixtures/b.js 94 bytes [built] [code generated]
		./fixtures/c.js 72 bytes [built] [code generated]
		Rspack compiled successfully"
	`);
	}
};
