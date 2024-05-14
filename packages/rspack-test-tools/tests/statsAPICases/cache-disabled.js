const { Stats } = require("@rspack/core");

/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should not have any cache hits log when cache is disabled",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/abc",
			cache: false
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
		expect(stats).not.toContain("module build cache");
		expect(stats).not.toContain("module factorize cache");
		expect(stats).not.toContain("module code generation cache");
	}
};
