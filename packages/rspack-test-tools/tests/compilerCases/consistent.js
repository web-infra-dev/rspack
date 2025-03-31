const stats = [];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should be called every compilation",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./split-chunks/src/index.js",
			optimization: {
				splitChunks: {
					chunks: 'all',
					minSize: 100 * 1024,
					maxSize: 200 * 1024,
				}
			},
			stats: {
				assets: true
			}
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run((err, stat) => {
				if (err) {
					throw err
				}
				stats.push(stat.toJson().assets)
				compiler.run((_, stat) => {
					stats.push(stat.toJson().assets)
					compiler.run((_, stat) => {
						stats.push(stat.toJson().assets)
						resolve();
					});
				});
			});
		});
	},
	async check() {
		const stats1 = stats[0].reduce((acc, curr) => acc + curr.name, '')
		const stats2 = stats[1].reduce((acc, curr) => acc + curr.name, '')
		const stats3 = stats[2].reduce((acc, curr) => acc + curr.name, '')
		expect(stats1 === stats2 === stats3);
	}
};
