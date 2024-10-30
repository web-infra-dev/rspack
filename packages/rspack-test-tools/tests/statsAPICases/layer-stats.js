/** @type {import('../../dist').TStatsAPICaseConfig} */
module.exports = {
	description: "should have module layer",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: {
					import: "./fixtures/abc",
					layer: "test"
				},
				legacy: {
					import: "./fixtures/abc",
					layer: "legacy"
				},
			},
			experiments: {
				layers: true
			}
		};
	},
	async check(stats) {
		const options = {
			all: false,
			modules: true,
			groupModulesByLayer: true
		};
		expect(stats?.toJson(options)).toMatchSnapshot();
	}
};
