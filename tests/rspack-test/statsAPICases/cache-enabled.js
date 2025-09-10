const { Stats } = require("@rspack/core");

/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description:
		"should have any cache hits log of modules in incremental rebuild mode",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc",
			cache: true
		};
	},
	async build(context, compiler) {
		await new Promise((resolve, reject) => {
			compiler.run(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		await new Promise((resolve, reject) => {
			compiler.__internal__rebuild(
				new Set([context.getSource("./fixtures/a")]),
				new Set(),
				err => {
					if (err) {
						return reject(err);
					}
					resolve();
				}
			);
		});
	},
	async check(_, compiler) {
		const stats = new Stats(compiler._lastCompilation).toString({
			all: false,
			logging: "verbose"
		});
		expect(stats).toContain("module code generation cache: 100.0% (4/4)");
	}
};
