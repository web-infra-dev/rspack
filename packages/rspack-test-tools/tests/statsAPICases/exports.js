/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have usedExports and providedExports stats",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/esm/abc"
			},
			optimization: {
				usedExports: true,
				providedExports: true
			},
			experiments: {
				rspackFuture: {
					newTreeshaking: true
				}
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			usedExports: true,
			providedExports: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"PublicPath: auto
		asset main.js 784 bytes [emitted] (name: main)
		Entrypoint main 784 bytes = main.js
		runtime modules 3 modules
		./fixtures/esm/a.js
		  [exports: a, default]
		  [only some exports used: a]
		./fixtures/esm/b.js
		  [exports: b, default]
		  [only some exports used: default]
		./fixtures/esm/c.js
		  [exports: c, default]
		./fixtures/esm/abc.js
		  [no exports]
		  [no exports used]
		Rspack compiled successfully (d58eb30bef2d7c671efc)"
	`);
	}
};
