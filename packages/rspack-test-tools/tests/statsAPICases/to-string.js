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
		"PublicPath: auto
		asset main.js 353 bytes [emitted] (name: main)
		Entrypoint main 353 bytes = main.js
		./fixtures/a.js
		./fixtures/b.js
		./fixtures/c.js
		./fixtures/abc.js
		Rspack compiled successfully (96a323a46758bf7c0e24)"
	`);
	}
};
