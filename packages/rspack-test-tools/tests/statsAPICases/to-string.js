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
		./fixtures/abc.js [built] [code generated]
		./fixtures/a.js [built] [code generated]
		./fixtures/b.js [built] [code generated]
		./fixtures/c.js [built] [code generated]
		Rspack compiled successfully"
	`);
	}
};
