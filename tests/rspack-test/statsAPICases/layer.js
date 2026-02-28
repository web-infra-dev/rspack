/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have module layer",
	options(context) {
		return {
			context: context.getSource(),
			mode: "development",
			entry: {
				main: {
					import: "./fixtures/abc",
					layer: "test"
				}
			},
		};
	},
	async check(stats) {
		const options = {
			all: false,
			modules: true,
		};
		const json = stats?.toJson(options);
		const jsModules = json.modules.filter(
			m => m.moduleType === "javascript/auto"
		);
		expect(jsModules).toHaveLength(4);
		expect(jsModules.every(m => m.layer === "test")).toBe(true);
		const string = stats?.toString(options);
		expect(string).toMatchInlineSnapshot(`
		./fixtures/abc.js (in test) 83 bytes [built] [code generated]
		./fixtures/a.js (in test) 55 bytes [built] [code generated]
		./fixtures/b.js (in test) 94 bytes [built] [code generated]
		./fixtures/c.js (in test) 72 bytes [built] [code generated]
	`);
	}
};
