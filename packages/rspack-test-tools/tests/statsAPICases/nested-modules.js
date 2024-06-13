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
			nestedModules: true
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
		    "chunks": Array [],
		    "depth": 1,
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
		    "orphan": true,
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
		    "type": "module",
		    "usedExports": Array [
		      "a",
		    ],
		  },
		  Object {
		    "assets": Array [],
		    "chunks": Array [],
		    "depth": 1,
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
		    "orphan": true,
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
		    "type": "module",
		    "usedExports": Array [
		      "default",
		    ],
		  },
		  Object {
		    "assets": Array [],
		    "chunks": Array [],
		    "depth": 1,
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
		    "orphan": true,
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
		    "type": "module",
		    "usedExports": true,
		  },
		  Object {
		    "assets": Array [],
		    "chunks": Array [],
		    "depth": 0,
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
		    "orphan": true,
		    "providedExports": Array [],
		    "reasons": Array [],
		    "size": 80,
		    "type": "module",
		    "usedExports": Array [],
		  },
		]
	`);
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"asset main.js 475 bytes [emitted] (name: main)
		Entrypoint main 475 bytes = main.js
		orphan modules [orphan] 4 modules
		runtime modules 3 modules
		./fixtures/esm/abc.js + 3 modules
		  | orphan modules [orphan] 4 modules
		Rspack 0.7.2 compiled successfully in 97 ms"
	`);
	}
};
