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
		PublicPath: auto
		asset main.js 204 bytes {889} [emitted] (name: main)
		Entrypoint main 204 bytes = main.js
		chunk {889} (runtime: main) main.js (main) 55 bytes [entry] [rendered]
		  > ./fixtures/a main
		  ./fixtures/a.js [195] 55 bytes {889} [depth 0] [built] [code generated]
		    [used exports unknown]
		    Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>
		    ModuleConcatenation bailout: Module is not an ECMAScript module
		    entry ./fixtures/a
		    cjs self exports reference self [195] ./fixtures/a.js
		./fixtures/a.js [195] 55 bytes {889} [depth 0] [built] [code generated]
		  [used exports unknown]
		  Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>
		  ModuleConcatenation bailout: Module is not an ECMAScript module
		  entry ./fixtures/a
		  cjs self exports reference self [195] ./fixtures/a.js
		  
		Rspack compiled successfully (74404557098e37ce)
	`);
	}
};
