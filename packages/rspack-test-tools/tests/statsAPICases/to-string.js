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
		asset main.js 726 bytes [emitted] (name: main)
		Entrypoint main 726 bytes = main.js
		./fixtures/a.js
		./fixtures/b.js
		./fixtures/c.js
		./fixtures/abc.js

		ERROR in ./fixtures/b.js
		  × Module parse failed:
		  ╰─▶   × JavaScript parsing error: Return statement is not allowed here
		         ╭─[4:1]
		       4 │
		       5 │ // Test CJS top-level return
		       6 │ return;
		         · ───────
		         ╰────
		      
		  help: 
		        You may need an appropriate loader to handle this file type.

		Rspack compiled with 1 error (3fa2fc6e23dccdd9a108)"
	`);
	}
};
