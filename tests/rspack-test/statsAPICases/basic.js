/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have stats",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/a"
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
		expect(typeof stats?.hash).toBe("string");
		expect(stats?.toJson(statsOptions)).toMatchInlineSnapshot(`
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
			      filteredRelated: 0,
			      info: Object {
			        chunkhash: Array [],
			        contenthash: Array [],
			        fullhash: Array [],
			        isOverSizeLimit: false,
			        javascriptModule: false,
			        minimized: true,
			        related: Object {},
			      },
			      isOverSizeLimit: false,
			      name: main.js,
			      related: Array [],
			      size: 204,
			      type: asset,
			    },
			  ],
			  assetsByChunkName: Object {
			    main: Array [
			      main.js,
			    ],
			  },
			  children: Array [],
			  chunks: Array [
			    Object {
			      auxiliaryFiles: Array [],
			      children: Array [],
			      childrenByOrder: Object {},
			      entry: true,
			      files: Array [
			        main.js,
			      ],
			      filteredModules: undefined,
			      hash: af49f1dcb0a9d8d0,
			      id: 889,
			      idHints: Array [],
			      initial: true,
			      modules: Array [
			        Object {
			          assets: Array [],
			          buildTimeExecuted: false,
			          built: true,
			          cacheable: true,
			          cached: false,
			          chunks: Array [
			            889,
			          ],
			          codeGenerated: true,
			          dependent: false,
			          depth: 0,
			          errors: 0,
			          failed: false,
			          filteredReasons: undefined,
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
			          optimizationBailout: Array [
			            Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>,
			            ModuleConcatenation bailout: Module is not an ECMAScript module,
			          ],
			          optional: false,
			          orphan: false,
			          postOrderIndex: 0,
			          preOrderIndex: 0,
			          providedExports: null,
			          reasons: Array [
			            Object {
			              active: true,
			              explanation: undefined,
			              loc: undefined,
			              moduleId: null,
			              resolvedModuleId: null,
			              type: entry,
			              userRequest: ./fixtures/a,
			            },
			            Object {
			              active: true,
			              explanation: undefined,
			              loc: undefined,
			              moduleId: 195,
			              moduleIdentifier: <TEST_ROOT>/fixtures/a.js,
			              moduleName: ./fixtures/a.js,
			              resolvedModule: ./fixtures/a.js,
			              resolvedModuleId: 195,
			              resolvedModuleIdentifier: <TEST_ROOT>/fixtures/a.js,
			              type: cjs self exports reference,
			              userRequest: self,
			            },
			          ],
			          size: 55,
			          sizes: Object {
			            javascript: 55,
			          },
			          source: module.exports = function a() {
				return "This is a";
			};,
			          type: module,
			          usedExports: null,
			          warnings: 0,
			        },
			      ],
			      names: Array [
			        main,
			      ],
			      origins: Array [
			        Object {
			          loc: main,
			          module: ,
			          moduleId: undefined,
			          moduleIdentifier: ,
			          moduleName: ,
			          request: ./fixtures/a,
			        },
			      ],
			      parents: Array [],
			      reason: undefined,
			      rendered: true,
			      runtime: Array [
			        main,
			      ],
			      siblings: Array [],
			      size: 55,
			      sizes: Object {
			        javascript: 55,
			      },
			      type: chunk,
			    },
			  ],
			  entrypoints: Object {
			    main: Object {
			      assets: Array [
			        Object {
			          name: main.js,
			          size: 204,
			        },
			      ],
			      assetsSize: 204,
			      auxiliaryAssets: Array [],
			      auxiliaryAssetsSize: 0,
			      childAssets: Object {},
			      children: Object {},
			      chunks: Array [
			        889,
			      ],
			      filteredAssets: 0,
			      isOverSizeLimit: false,
			      name: main,
			    },
			  },
			  env: undefined,
			  errors: Array [],
			  errorsCount: 0,
			  filteredAssets: undefined,
			  filteredModules: undefined,
			  hash: ed9c3bb682239d9a,
			  modules: Array [
			    Object {
			      assets: Array [],
			      buildTimeExecuted: false,
			      built: true,
			      cacheable: true,
			      cached: false,
			      chunks: Array [
			        889,
			      ],
			      codeGenerated: true,
			      dependent: undefined,
			      depth: 0,
			      errors: 0,
			      failed: false,
			      filteredReasons: undefined,
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
			      optimizationBailout: Array [
			        Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>,
			        ModuleConcatenation bailout: Module is not an ECMAScript module,
			      ],
			      optional: false,
			      orphan: false,
			      postOrderIndex: 0,
			      preOrderIndex: 0,
			      providedExports: null,
			      reasons: Array [
			        Object {
			          active: true,
			          explanation: undefined,
			          loc: undefined,
			          moduleId: null,
			          resolvedModuleId: null,
			          type: entry,
			          userRequest: ./fixtures/a,
			        },
			        Object {
			          active: true,
			          explanation: undefined,
			          loc: undefined,
			          moduleId: 195,
			          moduleIdentifier: <TEST_ROOT>/fixtures/a.js,
			          moduleName: ./fixtures/a.js,
			          resolvedModule: ./fixtures/a.js,
			          resolvedModuleId: 195,
			          resolvedModuleIdentifier: <TEST_ROOT>/fixtures/a.js,
			          type: cjs self exports reference,
			          userRequest: self,
			        },
			      ],
			      size: 55,
			      sizes: Object {
			        javascript: 55,
			      },
			      source: module.exports = function a() {
				return "This is a";
			};,
			      type: module,
			      usedExports: null,
			      warnings: 0,
			    },
			  ],
			  namedChunkGroups: Object {
			    main: Object {
			      assets: Array [
			        Object {
			          name: main.js,
			          size: 204,
			        },
			      ],
			      assetsSize: 204,
			      auxiliaryAssets: Array [],
			      auxiliaryAssetsSize: 0,
			      childAssets: Object {},
			      children: Object {},
			      chunks: Array [
			        889,
			      ],
			      filteredAssets: 0,
			      isOverSizeLimit: false,
			      name: main,
			    },
			  },
			  outputPath: <TEST_ROOT>/dist,
			  publicPath: auto,
			  warnings: Array [],
			  warningsCount: 0,
			}
		`);
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
			PublicPath: auto
			asset main.js 204 bytes {889} [emitted] (name: main)
			Entrypoint main 204 bytes = main.js
			chunk {889} (runtime: main) main.js (main) 55 bytes [entry] [rendered]
			  > ./fixtures/a main
			  ./fixtures/a.js [195] 55 bytes {889} [depth 0] [built] [code generated]
			    [used exports unknown]
			    Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>
			    ModuleConcatenation bailout: Module is not an ECMAScript module
			    entry ./fixtures/a
			    cjs self exports reference self [195] ./fixtures/a.js
			./fixtures/a.js [195] 55 bytes {889} [depth 0] [built] [code generated]
			  [used exports unknown]
			  Statement with side_effects in source code at ./fixtures/a.js<LINE_COL_RANGE>
			  ModuleConcatenation bailout: Module is not an ECMAScript module
			  entry ./fixtures/a
			  cjs self exports reference self [195] ./fixtures/a.js
			  
			Rspack compiled successfully (ed9c3bb682239d9a)
		`);
	}
};
