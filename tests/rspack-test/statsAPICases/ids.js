/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have ids when ids is true",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a"
		};
	},
	async check(stats) {
		const options = {
			all: false,
			assets: true,
			modules: true,
			chunks: true,
			ids: true
		};
		expect(stats?.toJson(options)).toMatchInlineSnapshot(`
			Object {
			  assets: Array [
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
			        889,
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
			      name: main.js,
			      size: 204,
			      type: asset,
			    },
			  ],
			  assetsByChunkName: Object {
			    main: Array [
			      main.js,
			    ],
			  },
			  chunks: Array [
			    Object {
			      auxiliaryFiles: Array [],
			      childrenByOrder: Object {},
			      entry: true,
			      files: Array [
			        main.js,
			      ],
			      hash: 23469efbff8aa6fe,
			      id: 889,
			      idHints: Array [],
			      initial: true,
			      names: Array [
			        main,
			      ],
			      reason: undefined,
			      rendered: true,
			      runtime: Array [
			        main,
			      ],
			      size: 55,
			      sizes: Object {
			        javascript: 55,
			      },
			      type: chunk,
			    },
			  ],
			  filteredAssets: undefined,
			  filteredModules: undefined,
			  modules: Array [
			    Object {
			      buildTimeExecuted: false,
			      built: true,
			      cacheable: true,
			      cached: false,
			      chunks: Array [
			        889,
			      ],
			      codeGenerated: true,
			      dependent: undefined,
			      errors: 0,
			      failed: false,
			      id: 195,
			      identifier: <TEST_ROOT>/fixtures/a.js,
			      index: 0,
			      index2: 0,
			      issuer: undefined,
			      issuerId: undefined,
			      issuerName: undefined,
			      issuerPath: undefined,
			      layer: undefined,
			      moduleType: javascript/auto,
			      name: ./fixtures/a.js,
			      nameForCondition: <TEST_ROOT>/fixtures/a.js,
			      optional: false,
			      orphan: false,
			      postOrderIndex: 0,
			      preOrderIndex: 0,
			      size: 55,
			      sizes: Object {
			        javascript: 55,
			      },
			      type: module,
			      warnings: 0,
			    },
			  ],
			}
		`);
		expect(stats?.toString(options)).toMatchInlineSnapshot(`
		asset main.js 204 bytes {889} [emitted] (name: main)
		chunk {889} (runtime: main) main.js (main) 55 bytes [entry] [rendered]
		./fixtures/a.js [195] 55 bytes {889} [built] [code generated]
	`);
	}
};
