/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have stats",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/esm/abc",
			optimization: {
				concatenateModules: true
			}
		};
	},
	async check(stats) {
		const statsOptions = {
			modules: true,
			nestedModules: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson(statsOptions);
		const concatedModule = statsJson.modules.find(
			m => m.name === "./fixtures/esm/abc.js + 3 modules"
		);
		expect(concatedModule).toBeTruthy();
		expect(concatedModule.modules).toMatchInlineSnapshot(`
		Array [
		  Object {
		    "assets": Array [],
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": false,
		    "depth": 0,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		    "issuer": undefined,
		    "issuerId": undefined,
		    "issuerName": undefined,
		    "issuerPath": Array [],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/esm/abc.js",
		    "nameForCondition": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		    "optimizationBailout": Array [
		      "ModuleConcatenation bailout: Module is an entry point",
		    ],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": 3,
		    "preOrderIndex": 0,
		    "providedExports": Array [],
		    "reasons": Array [],
		    "size": 80,
		    "sizes": Object {
		      "javascript": 80,
		    },
		    "type": "module",
		    "usedExports": Array [],
		    "warnings": 0,
		  },
		  Object {
		    "assets": Array [],
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": false,
		    "depth": 1,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/a.js",
		    "issuer": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		    "issuerId": undefined,
		    "issuerName": "./fixtures/esm/abc.js",
		    "issuerPath": Array [
		      Object {
		        "id": undefined,
		        "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "name": "./fixtures/esm/abc.js",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/esm/a.js",
		    "nameForCondition": "<PROJECT_ROOT>/tests/fixtures/esm/a.js",
		    "optimizationBailout": Array [],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": 0,
		    "preOrderIndex": 1,
		    "providedExports": Array [
		      "a",
		      "default",
		    ],
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import",
		        "userRequest": "./a",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import specifier",
		        "userRequest": "./a",
		      },
		    ],
		    "size": 37,
		    "sizes": Object {
		      "javascript": 37,
		    },
		    "type": "module",
		    "usedExports": Array [
		      "a",
		    ],
		    "warnings": 0,
		  },
		  Object {
		    "assets": Array [],
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": false,
		    "depth": 1,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/b.js",
		    "issuer": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		    "issuerId": undefined,
		    "issuerName": "./fixtures/esm/abc.js",
		    "issuerPath": Array [
		      Object {
		        "id": undefined,
		        "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "name": "./fixtures/esm/abc.js",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/esm/b.js",
		    "nameForCondition": "<PROJECT_ROOT>/tests/fixtures/esm/b.js",
		    "optimizationBailout": Array [],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": 1,
		    "preOrderIndex": 2,
		    "providedExports": Array [
		      "b",
		      "default",
		    ],
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import",
		        "userRequest": "./b",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import specifier",
		        "userRequest": "./b",
		      },
		    ],
		    "size": 38,
		    "sizes": Object {
		      "javascript": 38,
		    },
		    "type": "module",
		    "usedExports": Array [
		      "default",
		    ],
		    "warnings": 0,
		  },
		  Object {
		    "assets": Array [],
		    "built": true,
		    "cacheable": true,
		    "cached": false,
		    "chunks": Array [],
		    "codeGenerated": false,
		    "depth": 1,
		    "errors": 0,
		    "failed": false,
		    "id": undefined,
		    "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/c.js",
		    "issuer": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		    "issuerId": undefined,
		    "issuerName": "./fixtures/esm/abc.js",
		    "issuerPath": Array [
		      Object {
		        "id": undefined,
		        "identifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "name": "./fixtures/esm/abc.js",
		      },
		    ],
		    "moduleType": "javascript/auto",
		    "name": "./fixtures/esm/c.js",
		    "nameForCondition": "<PROJECT_ROOT>/tests/fixtures/esm/c.js",
		    "optimizationBailout": Array [],
		    "optional": false,
		    "orphan": true,
		    "postOrderIndex": 2,
		    "preOrderIndex": 3,
		    "providedExports": Array [
		      "c",
		      "default",
		    ],
		    "reasons": Array [
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import",
		        "userRequest": "./c",
		      },
		      Object {
		        "moduleId": undefined,
		        "moduleIdentifier": "<PROJECT_ROOT>/tests/fixtures/esm/abc.js",
		        "moduleName": "./fixtures/esm/abc.js",
		        "type": "esm import specifier",
		        "userRequest": "./c",
		      },
		    ],
		    "size": 37,
		    "sizes": Object {
		      "javascript": 37,
		    },
		    "type": "module",
		    "usedExports": true,
		    "warnings": 0,
		  },
		]
	`);
		expect(stats?.toString(statsOptions).replace(/\d+ ms/g, "X ms"))
			.toMatchInlineSnapshot(`
		"asset main.js 475 bytes [emitted] (name: main)
		Entrypoint main 475 bytes = main.js
		orphan modules 192 bytes [orphan] 4 modules
		runtime modules 677 bytes 3 modules
		./fixtures/esm/abc.js + 3 modules 192 bytes [code generated]
		  | orphan modules 192 bytes [orphan] 4 modules
		Rspack compiled successfully"
	`);
	}
};
