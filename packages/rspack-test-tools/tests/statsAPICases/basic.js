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
		asset main.js 211 bytes {909} [emitted] (name: main)
		Entrypoint main 211 bytes = main.js
		chunk {909} main.js (main) [entry]
		  ./fixtures/a.js [585] {909}
		    [no exports]
		    [no exports used]
		    Statement with side_effects in source code at ./fixtures/a.js:1:0-3:2
		    entry ./fixtures/a
		./fixtures/a.js [585] {909}
		  [no exports]
		  [no exports used]
		  Statement with side_effects in source code at ./fixtures/a.js:1:0-3:2
		  entry ./fixtures/a
		  
		Rspack compiled successfully (57e46af248a1c1fe076f)"
	`);
	}
};
