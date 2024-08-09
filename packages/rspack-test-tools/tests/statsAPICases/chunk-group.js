/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should generate chunk group asset",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/order/index"
			},
			optimization: {
				minimize: false
			},
			devtool: "source-map"
		};
	},
	async check(stats) {
		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(stats?.toJson(statsOptions).entrypoints).toMatchInlineSnapshot(`
		Object {
		  "main": Object {
		    "assets": Array [
		      Object {
		        "name": "main.js",
		        "size": 14167,
		      },
		    ],
		    "assetsSize": 14167,
		    "auxiliaryAssets": Array [
		      Object {
		        "name": "main.js.map",
		        "size": 681,
		      },
		    ],
		    "auxiliaryAssetsSize": 681,
		    "childAssets": Object {},
		    "children": Object {
		      "prefetch": Array [
		        Object {
		          "assets": Array [
		            Object {
		              "name": "chunk.js",
		              "size": 862,
		            },
		          ],
		          "assetsSize": 862,
		          "auxiliaryAssets": Array [
		            Object {
		              "name": "chunk.js.map",
		              "size": 514,
		            },
		          ],
		          "auxiliaryAssetsSize": 514,
		          "chunks": Array [
		            "919",
		          ],
		          "name": "chunk",
		        },
		      ],
		    },
		    "chunks": Array [
		      "909",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": false,
		    "name": "main",
		  },
		}
	`);
		expect(stats?.toJson(statsOptions).namedChunkGroups).toMatchInlineSnapshot(`
		Object {
		  "chunk": Object {
		    "assets": Array [
		      Object {
		        "name": "chunk.js",
		        "size": 862,
		      },
		    ],
		    "assetsSize": 862,
		    "auxiliaryAssets": Array [
		      Object {
		        "name": "chunk.js.map",
		        "size": 514,
		      },
		    ],
		    "auxiliaryAssetsSize": 514,
		    "childAssets": Object {
		      "prefetch": Array [
		        "chunk-b.js",
		      ],
		      "preload": Array [
		        "chunk-b.js",
		      ],
		    },
		    "children": Object {
		      "prefetch": Array [
		        Object {
		          "assets": Array [
		            Object {
		              "name": "chunk-c.js",
		              "size": 136,
		            },
		          ],
		          "assetsSize": 136,
		          "auxiliaryAssets": Array [],
		          "auxiliaryAssetsSize": 0,
		          "chunks": Array [
		            "212",
		          ],
		          "name": "chunk-c",
		        },
		        Object {
		          "assets": Array [
		            Object {
		              "name": "chunk-a.js",
		              "size": 136,
		            },
		          ],
		          "assetsSize": 136,
		          "auxiliaryAssets": Array [],
		          "auxiliaryAssetsSize": 0,
		          "chunks": Array [
		            "807",
		          ],
		          "name": "chunk-a",
		        },
		      ],
		      "preload": Array [
		        Object {
		          "assets": Array [
		            Object {
		              "name": "chunk-b.js",
		              "size": 136,
		            },
		          ],
		          "assetsSize": 136,
		          "auxiliaryAssets": Array [],
		          "auxiliaryAssetsSize": 0,
		          "chunks": Array [
		            "805",
		          ],
		          "name": "chunk-b",
		        },
		      ],
		    },
		    "chunks": Array [
		      "919",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": undefined,
		    "name": "chunk",
		  },
		  "chunk-a": Object {
		    "assets": Array [
		      Object {
		        "name": "chunk-a.js",
		        "size": 136,
		      },
		    ],
		    "assetsSize": 136,
		    "auxiliaryAssets": Array [],
		    "auxiliaryAssetsSize": 0,
		    "childAssets": Object {},
		    "children": Object {},
		    "chunks": Array [
		      "807",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": undefined,
		    "name": "chunk-a",
		  },
		  "chunk-b": Object {
		    "assets": Array [
		      Object {
		        "name": "chunk-b.js",
		        "size": 136,
		      },
		    ],
		    "assetsSize": 136,
		    "auxiliaryAssets": Array [],
		    "auxiliaryAssetsSize": 0,
		    "childAssets": Object {},
		    "children": Object {},
		    "chunks": Array [
		      "805",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": undefined,
		    "name": "chunk-b",
		  },
		  "chunk-c": Object {
		    "assets": Array [
		      Object {
		        "name": "chunk-c.js",
		        "size": 136,
		      },
		    ],
		    "assetsSize": 136,
		    "auxiliaryAssets": Array [],
		    "auxiliaryAssetsSize": 0,
		    "childAssets": Object {},
		    "children": Object {},
		    "chunks": Array [
		      "212",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": undefined,
		    "name": "chunk-c",
		  },
		  "main": Object {
		    "assets": Array [
		      Object {
		        "name": "main.js",
		        "size": 14167,
		      },
		    ],
		    "assetsSize": 14167,
		    "auxiliaryAssets": Array [
		      Object {
		        "name": "main.js.map",
		        "size": 681,
		      },
		    ],
		    "auxiliaryAssetsSize": 681,
		    "childAssets": Object {},
		    "children": Object {
		      "prefetch": Array [
		        Object {
		          "assets": Array [
		            Object {
		              "name": "chunk.js",
		              "size": 862,
		            },
		          ],
		          "assetsSize": 862,
		          "auxiliaryAssets": Array [
		            Object {
		              "name": "chunk.js.map",
		              "size": 514,
		            },
		          ],
		          "auxiliaryAssetsSize": 514,
		          "chunks": Array [
		            "919",
		          ],
		          "name": "chunk",
		        },
		      ],
		    },
		    "chunks": Array [
		      "909",
		    ],
		    "filteredAssets": 0,
		    "isOverSizeLimit": false,
		    "name": "main",
		  },
		}
	`);
	}
};
