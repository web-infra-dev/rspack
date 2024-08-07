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
		    "auxiliaryChunkIdHints": Array [],
		    "auxiliaryChunkNames": Array [
		      "main",
		    ],
		    "auxiliaryChunks": Array [
		      "909",
		    ],
		    "cached": false,
		    "chunkIdHints": Array [],
		    "chunkNames": Array [],
		    "chunks": Array [],
		    "emitted": true,
		    "filteredRelated": 0,
		    "info": Object {
		      "chunkhash": Array [],
		      "contenthash": Array [],
		      "development": false,
		      "fullhash": Array [
		        "89a353e9c515885abd8e",
		      ],
		      "hotModuleReplacement": false,
		      "immutable": true,
		      "isOverSizeLimit": false,
		      "minimized": false,
		      "related": Object {},
		      "sourceFilename": "fixtures/asset/image.png",
		    },
		    "isOverSizeLimit": false,
		    "name": "89a353e9c515885abd8e.png",
		    "related": Array [],
		    "size": 14910,
		    "type": "asset",
		  },
		  Object {
		    "auxiliaryChunkIdHints": Array [],
		    "auxiliaryChunkNames": Array [],
		    "auxiliaryChunks": Array [],
		    "cached": false,
		    "chunkIdHints": Array [],
		    "chunkNames": Array [
		      "main",
		    ],
		    "chunks": Array [
		      "909",
		    ],
		    "emitted": true,
		    "filteredRelated": 0,
		    "info": Object {
		      "chunkhash": Array [],
		      "contenthash": Array [],
		      "development": false,
		      "fullhash": Array [],
		      "hotModuleReplacement": false,
		      "immutable": false,
		      "isOverSizeLimit": false,
		      "javascriptModule": false,
		      "minimized": false,
		      "related": Object {
		        "sourceMap": Array [
		          "main.js.map",
		        ],
		      },
		    },
		    "isOverSizeLimit": false,
		    "name": "main.js",
		    "related": Array [],
		    "size": 2425,
		    "type": "asset",
		  },
		]
	`);
	}
};
