/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should contain assets",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				entryA: "./fixtures/a",
				entryB: "./fixtures/chunk-b"
			}
		};
	},
	async check(stats) {
		expect(
			stats.toJson({
				all: false,
				errorsCount: true,
				assets: true
			})
		).toMatchInlineSnapshot(`
			Object {
			  assets: Array [
			    Object {
			      auxiliaryChunkIdHints: Array [],
			      auxiliaryChunkNames: Array [],
			      cached: false,
			      chunkIdHints: Array [],
			      chunkNames: Array [
			        entryB,
			      ],
			      emitted: true,
			      info: Object {
			        chunkhash: Array [],
			        contenthash: Array [],
			        fullhash: Array [],
			        isOverSizeLimit: false,
			        javascriptModule: false,
			        minimized: true,
			        related: Object {},
			      },
			      name: entryB.js,
			      size: 3190,
			      type: asset,
			    },
			    Object {
			      auxiliaryChunkIdHints: Array [],
			      auxiliaryChunkNames: Array [],
			      cached: false,
			      chunkIdHints: Array [],
			      chunkNames: Array [
			        entryA,
			      ],
			      emitted: true,
			      info: Object {
			        chunkhash: Array [],
			        contenthash: Array [],
			        fullhash: Array [],
			        isOverSizeLimit: false,
			        javascriptModule: false,
			        minimized: true,
			        related: Object {},
			      },
			      name: entryA.js,
			      size: 195,
			      type: asset,
			    },
			    Object {
			      auxiliaryChunkIdHints: Array [],
			      auxiliaryChunkNames: Array [],
			      cached: false,
			      chunkIdHints: Array [],
			      chunkNames: Array [
			        chunkB,
			      ],
			      emitted: true,
			      info: Object {
			        chunkhash: Array [],
			        contenthash: Array [],
			        fullhash: Array [],
			        isOverSizeLimit: false,
			        javascriptModule: false,
			        minimized: true,
			        related: Object {},
			      },
			      name: chunkB.js,
			      size: 132,
			      type: asset,
			    },
			  ],
			  assetsByChunkName: Object {
			    chunkB: Array [
			      chunkB.js,
			    ],
			    entryA: Array [
			      entryA.js,
			    ],
			    entryB: Array [
			      entryB.js,
			    ],
			  },
			  errorsCount: 0,
			  filteredAssets: undefined,
			}
		`);
	}
};
