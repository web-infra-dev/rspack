/** @type {import('../../dist').TStatsAPICaseConfig} */
module.exports = {
  description: "should inject bundler info runtime modules",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./fixtures/index",
      experiments: {
        rspackFuture: {
          bundlerInfo: {
            force: true
          }
        }
      }
    };
  },
  async check(stats) {
    const statsOptions = {
      runtimeModules: true
    };
    expect(typeof stats?.hash).toBe("string");
    const statsJson = stats?.toJson(statsOptions);
    expect(
      statsJson.modules.filter(m => m.identifier.startsWith("webpack/runtime/"))
    ).toMatchInlineSnapshot(`
		Array [
		  Object {
		    "assets": Array [],
		    "buildTimeExecuted": false,
		    "built": false,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [
		      "909",
		    ],
		    "codeGenerated": true,
		    "depth": undefined,
		    "errors": 0,
		    "failed": false,
		    "id": "",
		    "identifier": "webpack/runtime/rspack_version",
		    "issuer": undefined,
		    "issuerId": undefined,
		    "issuerName": undefined,
		    "issuerPath": Array [],
		    "moduleType": "runtime",
		    "name": "webpack/runtime/rspack_version",
		    "nameForCondition": undefined,
		    "optimizationBailout": Array [],
		    "optional": false,
		    "orphan": false,
		    "postOrderIndex": undefined,
		    "preOrderIndex": undefined,
		    "providedExports": Array [],
		    "reasons": Array [],
		    "size": 59,
		    "sizes": Object {
		      "runtime": 59,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		  Object {
		    "assets": Array [],
		    "buildTimeExecuted": false,
		    "built": false,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [
		      "909",
		    ],
		    "codeGenerated": true,
		    "depth": undefined,
		    "errors": 0,
		    "failed": false,
		    "id": "",
		    "identifier": "webpack/runtime/rspack_unique_id",
		    "issuer": undefined,
		    "issuerId": undefined,
		    "issuerName": undefined,
		    "issuerPath": Array [],
		    "moduleType": "runtime",
		    "name": "webpack/runtime/rspack_unique_id",
		    "nameForCondition": undefined,
		    "optimizationBailout": Array [],
		    "optional": false,
		    "orphan": false,
		    "postOrderIndex": undefined,
		    "preOrderIndex": undefined,
		    "providedExports": Array [],
		    "reasons": Array [],
		    "size": 206,
		    "sizes": Object {
		      "runtime": 206,
		    },
		    "type": "module",
		    "usedExports": null,
		    "warnings": 0,
		  },
		]
	`);
  }
};
