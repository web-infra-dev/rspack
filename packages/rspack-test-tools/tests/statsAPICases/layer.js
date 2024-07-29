/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have module layer",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: {
					import: "./fixtures/abc",
					layer: "test"
				}
			},
			experiments: {
				layers: true
			}
		};
	},
	async check(stats) {
		const options = {
			all: false,
			modules: true
		};
		expect(stats?.toJson(options)).toMatchSnapshot();
		expect(stats?.toString(options)).toMatchInlineSnapshot(`
		"./fixtures/abc.js (in test) 83 bytes [built] [code generated]
		./fixtures/a.js (in test) 55 bytes [built] [code generated]
		./fixtures/b.js (in test) 94 bytes [built] [code generated]
		./fixtures/c.js (in test) 72 bytes [built] [code generated]"
	`);
	}
};
