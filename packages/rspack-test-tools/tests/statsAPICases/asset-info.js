/** @type {import('../..').TStatsAPICaseConfig} */
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
		expect(stats?.toJson(statsOptions).assets).toMatchInlineSnapshot(`
		Array [
		  Object {
		    auxiliaryChunkIdHints: Array [],
		    auxiliaryChunkNames: Array [
		      main,
		    ],
		    auxiliaryChunks: Array [
		      909,
		    ],
		    cached: false,
		    chunkIdHints: Array [],
		    chunkNames: Array [],
		    chunks: Array [],
		    emitted: true,
		    filteredRelated: 0,
		    info: Object {
		      chunkhash: Array [],
		      contenthash: Array [],
		      fullhash: Array [
		        c560fa876f51d750,
		      ],
		      immutable: true,
		      isOverSizeLimit: false,
		      related: Object {},
		      sourceFilename: fixtures/asset/image.png,
		    },
		    isOverSizeLimit: false,
		    name: c560fa876f51d750.png,
		    related: Array [],
		    size: 14910,
		    type: asset,
		  },
		  Object {
		    auxiliaryChunkIdHints: Array [],
		    auxiliaryChunkNames: Array [],
		    auxiliaryChunks: Array [],
		    cached: false,
		    chunkIdHints: Array [],
		    chunkNames: Array [
		      main,
		    ],
		    chunks: Array [
		      909,
		    ],
		    emitted: true,
		    filteredRelated: 0,
		    info: Object {
		      chunkhash: Array [],
		      contenthash: Array [],
		      fullhash: Array [],
		      isOverSizeLimit: false,
		      javascriptModule: false,
		      related: Object {
		        sourceMap: Array [
		          main.js.map,
		        ],
		      },
		    },
		    isOverSizeLimit: false,
		    name: main.js,
		    related: Array [],
		    size: 2663,
		    type: asset,
		  },
		]
	`);
	}
};
