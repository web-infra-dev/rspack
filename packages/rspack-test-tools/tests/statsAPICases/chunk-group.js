function deepReplaceNumbers(obj) {
	if (typeof obj === "object" && obj !== null) {
		for (const key in obj) {
			if (Object.prototype.hasOwnProperty.call(obj, key)) {
				if (
					typeof obj[key] === "number" &&
					(key.includes("size") || key.includes("Size"))
				) {
					obj[key] = "xxx";
				} else if (typeof obj[key] === "object") {
					deepReplaceNumbers(obj[key]);
				}
			}
		}
	}
}

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

		const entrypoints = stats?.toJson(statsOptions).entrypoints;

		deepReplaceNumbers(entrypoints);

		expect(entrypoints).toMatchInlineSnapshot(`
		Object {
		  main: Object {
		    assets: Array [
		      Object {
		        name: main.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [
		      Object {
		        name: main.js.map,
		        size: xxx,
		      },
		    ],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {},
		    children: Object {
		      prefetch: Array [
		        Object {
		          assets: Array [
		            Object {
		              name: chunk.js,
		              size: xxx,
		            },
		          ],
		          assetsSize: xxx,
		          auxiliaryAssets: Array [
		            Object {
		              name: chunk.js.map,
		              size: xxx,
		            },
		          ],
		          auxiliaryAssetsSize: xxx,
		          chunks: Array [
		            919,
		          ],
		          name: chunk,
		        },
		      ],
		    },
		    chunks: Array [
		      909,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: false,
		    name: main,
		  },
		}
	`);

		const namedChunkGroups = stats?.toJson(statsOptions).namedChunkGroups;

		deepReplaceNumbers(namedChunkGroups);

		expect(namedChunkGroups).toMatchInlineSnapshot(`
		Object {
		  chunk: Object {
		    assets: Array [
		      Object {
		        name: chunk.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [
		      Object {
		        name: chunk.js.map,
		        size: xxx,
		      },
		    ],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {
		      prefetch: Array [
		        chunk-b.js,
		      ],
		      preload: Array [
		        chunk-b.js,
		      ],
		    },
		    children: Object {
		      prefetch: Array [
		        Object {
		          assets: Array [
		            Object {
		              name: chunk-c.js,
		              size: xxx,
		            },
		          ],
		          assetsSize: xxx,
		          auxiliaryAssets: Array [],
		          auxiliaryAssetsSize: xxx,
		          chunks: Array [
		            212,
		          ],
		          name: chunk-c,
		        },
		        Object {
		          assets: Array [
		            Object {
		              name: chunk-a.js,
		              size: xxx,
		            },
		          ],
		          assetsSize: xxx,
		          auxiliaryAssets: Array [],
		          auxiliaryAssetsSize: xxx,
		          chunks: Array [
		            807,
		          ],
		          name: chunk-a,
		        },
		      ],
		      preload: Array [
		        Object {
		          assets: Array [
		            Object {
		              name: chunk-b.js,
		              size: xxx,
		            },
		          ],
		          assetsSize: xxx,
		          auxiliaryAssets: Array [],
		          auxiliaryAssetsSize: xxx,
		          chunks: Array [
		            805,
		          ],
		          name: chunk-b,
		        },
		      ],
		    },
		    chunks: Array [
		      919,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: undefined,
		    name: chunk,
		  },
		  chunk-a: Object {
		    assets: Array [
		      Object {
		        name: chunk-a.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {},
		    children: Object {},
		    chunks: Array [
		      807,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: undefined,
		    name: chunk-a,
		  },
		  chunk-b: Object {
		    assets: Array [
		      Object {
		        name: chunk-b.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {},
		    children: Object {},
		    chunks: Array [
		      805,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: undefined,
		    name: chunk-b,
		  },
		  chunk-c: Object {
		    assets: Array [
		      Object {
		        name: chunk-c.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {},
		    children: Object {},
		    chunks: Array [
		      212,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: undefined,
		    name: chunk-c,
		  },
		  main: Object {
		    assets: Array [
		      Object {
		        name: main.js,
		        size: xxx,
		      },
		    ],
		    assetsSize: xxx,
		    auxiliaryAssets: Array [
		      Object {
		        name: main.js.map,
		        size: xxx,
		      },
		    ],
		    auxiliaryAssetsSize: xxx,
		    childAssets: Object {},
		    children: Object {
		      prefetch: Array [
		        Object {
		          assets: Array [
		            Object {
		              name: chunk.js,
		              size: xxx,
		            },
		          ],
		          assetsSize: xxx,
		          auxiliaryAssets: Array [
		            Object {
		              name: chunk.js.map,
		              size: xxx,
		            },
		          ],
		          auxiliaryAssetsSize: xxx,
		          chunks: Array [
		            919,
		          ],
		          name: chunk,
		        },
		      ],
		    },
		    chunks: Array [
		      909,
		    ],
		    filteredAssets: 0,
		    isOverSizeLimit: false,
		    name: main,
		  },
		}
	`);
	}
};
