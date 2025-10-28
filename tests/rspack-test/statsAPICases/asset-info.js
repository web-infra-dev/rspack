/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should generate asset info",
	options(context) {
		return {
			devtool: "source-map",
			context: context.getSource(),
			optimization: {
				minimize: false
			},
			entry: {
				main: "./fixtures/asset/index"
			},
			output: {},
			module: {
				rules: [
					{
						test: /\.png/,
						type: "asset/resource"
					}
				]
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
		const assets = stats?.toJson(statsOptions).assets;
		assets.sort((a, b) => a.name.localeCompare(b.name));
		const infos = assets.map(i => {
			i.info.fullhash = i.info.fullhash.map(i => "xxx");
			return i.info;
		});
		expect(infos).toEqual([{
			chunkhash: [],
			contenthash: [],
			fullhash: ["xxx"],
			immutable: true,
			isOverSizeLimit: false,
			related: {},
			sourceFilename: "fixtures/asset/image.png",
		}, {
			chunkhash: [],
			contenthash: [],
			fullhash: [],
			isOverSizeLimit: false,
			javascriptModule: false,
			related: {
				sourceMap: [
					"main.js.map",
				],
			},
		}]);
	}
};
