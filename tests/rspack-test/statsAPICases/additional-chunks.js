/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should contain additional chunks",
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
				chunkGroups: true
			})
		).toMatchInlineSnapshot(`
			Object {
			  errorsCount: 0,
			  namedChunkGroups: Object {
			    chunkB: Object {
			      assets: Array [
			        Object {
			          name: chunkB.js,
			          size: 132,
			        },
			      ],
			      assetsSize: 132,
			      auxiliaryAssets: undefined,
			      auxiliaryAssetsSize: undefined,
			      childAssets: undefined,
			      children: undefined,
			      chunks: Array [
			        513,
			      ],
			      filteredAssets: 0,
			      name: chunkB,
			    },
			    entryA: Object {
			      assets: Array [
			        Object {
			          name: entryA.js,
			          size: 195,
			        },
			      ],
			      assetsSize: 195,
			      auxiliaryAssets: undefined,
			      auxiliaryAssetsSize: undefined,
			      childAssets: undefined,
			      children: undefined,
			      chunks: Array [
			        759,
			      ],
			      filteredAssets: 0,
			      name: entryA,
			    },
			    entryB: Object {
			      assets: Array [
			        Object {
			          name: entryB.js,
			          size: 3171,
			        },
			      ],
			      assetsSize: 3171,
			      auxiliaryAssets: undefined,
			      auxiliaryAssetsSize: undefined,
			      childAssets: undefined,
			      children: undefined,
			      chunks: Array [
			        706,
			      ],
			      filteredAssets: 0,
			      name: entryB,
			    },
			  },
			}
		`);
	}
};
