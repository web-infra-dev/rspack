/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have nested modules",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/esm/abc",
			optimization: {
				concatenateModules: true
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			modules: true,
			nestedModules: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const concatedModule = statsJson.modules.find(
			m => m.name === "./fixtures/esm/abc.js + 3 modules"
		);
		expect(concatedModule).toBeTruthy();
		expect(stats?.toString(statsOptions).replace(/\d+ ms/g, "X ms"))
			.toMatchInlineSnapshot(`
		asset main.js 412 bytes [emitted] (name: main)
		orphan modules 192 bytes [orphan] 4 modules
		runtime modules 647 bytes 3 modules
		./fixtures/esm/abc.js + 3 modules 192 bytes [code generated]
		  | orphan modules 192 bytes [orphan] 4 modules
		Rspack compiled successfully
	`);
	}
};
