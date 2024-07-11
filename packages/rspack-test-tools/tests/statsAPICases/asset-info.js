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
		    "chunkNames": Array [
		      "main",
		    ],
		    "chunks": Array [
		      "909",
		    ],
		    "emitted": true,
		    "info": Object {
		      "chunkHash": Array [],
		      "contentHash": Array [
		        "89a353e9c515885abd8e",
		      ],
		      "development": false,
		      "hotModuleReplacement": false,
		      "immutable": true,
		      "minimized": false,
		      "related": Object {},
		      "sourceFilename": "fixtures/asset/image.png",
		    },
		    "name": "89a353e9c515885abd8e.png",
		    "size": 14910,
		    "type": "asset",
		  },
		  Object {
		    "chunkNames": Array [
		      "main",
		    ],
		    "chunks": Array [
		      "909",
		    ],
		    "emitted": true,
		    "info": Object {
		      "chunkHash": Array [],
		      "contentHash": Array [],
		      "development": false,
		      "hotModuleReplacement": false,
		      "immutable": false,
		      "javascriptModule": false,
		      "minimized": false,
		      "related": Object {
		        "sourceMap": Array [
		          "main.js.map",
		        ],
		      },
		    },
		    "name": "main.js",
		    "size": 2425,
		    "type": "asset",
		  },
		]
	`);
	}
};
