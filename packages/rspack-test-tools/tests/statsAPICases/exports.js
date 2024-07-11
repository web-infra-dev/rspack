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
		"asset main.js 441 bytes [emitted] (name: main)
		Entrypoint main 441 bytes = main.js
		orphan modules 192 bytes [orphan] 4 modules
		runtime modules 677 bytes 3 modules
		./fixtures/esm/abc.js + 3 modules 192 bytes [code generated]
		  [no exports]
		  [no exports used]
		Rspack compiled successfully"
	`);
	}
};
