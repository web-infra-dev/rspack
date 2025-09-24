defineStatsAPICase(Utils.basename(__filename), {
	description: "should have stats",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./a"
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
		      hash: 4ee9e6e51ec11d11,
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
		          id: 670,
		          identifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
		          index: 0,
		          index2: 0,
		          issuer: undefined,
		          issuerId: undefined,
		          issuerName: undefined,
		          issuerPath: undefined,
		          layer: undefined,
		          moduleType: javascript/auto,
		          name: ./a.js,
		          nameForCondition: <TEST_TOOLS_ROOT>/fixtures/a.js,
		          optimizationBailout: Array [
		            Statement with side_effects in source code at ./a.js<LINE_COL_RANGE>,
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
		              userRequest: ./a,
		            },
		            Object {
		              active: true,
		              explanation: undefined,
		              loc: undefined,
		              moduleId: 670,
		              moduleIdentifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
		              moduleName: ./a.js,
		              resolvedModule: ./a.js,
		              resolvedModuleId: 670,
		              resolvedModuleIdentifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
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
		          request: ./a,
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
		  hash: 01bf851dd9d2b320,
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
		      id: 670,
		      identifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
		      index: 0,
		      index2: 0,
		      issuer: undefined,
		      issuerId: undefined,
		      issuerName: undefined,
		      issuerPath: undefined,
		      layer: undefined,
		      moduleType: javascript/auto,
		      name: ./a.js,
		      nameForCondition: <TEST_TOOLS_ROOT>/fixtures/a.js,
		      optimizationBailout: Array [
		        Statement with side_effects in source code at ./a.js<LINE_COL_RANGE>,
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
		          userRequest: ./a,
		        },
		        Object {
		          active: true,
		          explanation: undefined,
		          loc: undefined,
		          moduleId: 670,
		          moduleIdentifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
		          moduleName: ./a.js,
		          resolvedModule: ./a.js,
		          resolvedModuleId: 670,
		          resolvedModuleIdentifier: <TEST_TOOLS_ROOT>/fixtures/a.js,
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
		  outputPath: <TEST_TOOLS_ROOT>/dist,
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
		  > ./a main
		  ./a.js [670] 55 bytes {889} [depth 0] [built] [code generated]
		    [used exports unknown]
		    Statement with side_effects in source code at ./a.js<LINE_COL_RANGE>
		    ModuleConcatenation bailout: Module is not an ECMAScript module
		    entry ./a
		    cjs self exports reference self [670] ./a.js
		./a.js [670] 55 bytes {889} [depth 0] [built] [code generated]
		  [used exports unknown]
		  Statement with side_effects in source code at ./a.js<LINE_COL_RANGE>
		  ModuleConcatenation bailout: Module is not an ECMAScript module
		  entry ./a
		  cjs self exports reference self [670] ./a.js
		  
		Rspack compiled successfully (01bf851dd9d2b320)
	`);
	}
});
