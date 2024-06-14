/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have stats",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/a"
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"PublicPath: auto
		asset main.js 207 bytes {909} [emitted] (name: main)
		Entrypoint main 207 bytes = main.js
		chunk {909} main.js (main) [entry]
		  ./fixtures/a.js [585] {909} [depth 0] [built] [code generated]
		    [no exports]
		    [used exports unknown]
		    Statement with side_effects in source code at ./fixtures/a.js:1:0-3:2
		    entry ./fixtures/a
		    cjs self exports reference self [585]
		./fixtures/a.js [585] {909} [depth 0] [built] [code generated]
		  [no exports]
		  [used exports unknown]
		  Statement with side_effects in source code at ./fixtures/a.js:1:0-3:2
		  entry ./fixtures/a
		  cjs self exports reference self [585]
		  
		Rspack compiled successfully (99b766bb1908ecab2ebb)"
	`);
	}
};
