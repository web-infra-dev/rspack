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
			all: true
		};
		expect(typeof stats?.hash).toBe("string");
		const statsJson = stats?.toJson({
			modules: true
		});
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
		"PublicPath: auto
		asset main.js 475 bytes {909} [emitted] (name: main)
		Entrypoint main 475 bytes = main.js
		chunk {909} main.js (main) [entry]
		  ./fixtures/esm/abc.js + 3 modules [533] {909} [depth 0]
		    [no exports]
		    [no exports used]
		    entry ./fixtures/esm/abc
		    | ./fixtures/esm/a.js [depth 1] [orphan]
		    |   [exports: a, default]
		    |   [only some exports used: a]
		    |   esm import ./a
		    |   esm import specifier ./a
		    | ./fixtures/esm/b.js [depth 1] [orphan]
		    |   [exports: b, default]
		    |   [only some exports used: default]
		    |   esm import ./b
		    |   esm import specifier ./b
		    | ./fixtures/esm/c.js [depth 1] [orphan]
		    |   [exports: c, default]
		    |   esm import ./c
		    |   esm import specifier ./c
		    | ./fixtures/esm/abc.js [depth 0] [orphan]
		    |   [no exports]
		    |   [no exports used]
		    |   ModuleConcatenation bailout: Module is an entry point
		javascript modules
		  ./fixtures/esm/abc.js [depth 0] [orphan]
		    [no exports]
		    [no exports used]
		    ModuleConcatenation bailout: Module is an entry point
		  ./fixtures/esm/abc.js + 3 modules [533] {909} [depth 0]
		    [no exports]
		    [no exports used]
		    entry ./fixtures/esm/abc
		    | ./fixtures/esm/a.js [depth 1] [orphan]
		    |   [exports: a, default]
		    |   [only some exports used: a]
		    |   esm import ./a
		    |   esm import specifier ./a
		    | ./fixtures/esm/b.js [depth 1] [orphan]
		    |   [exports: b, default]
		    |   [only some exports used: default]
		    |   esm import ./b
		    |   esm import specifier ./b
		    | ./fixtures/esm/c.js [depth 1] [orphan]
		    |   [exports: c, default]
		    |   esm import ./c
		    |   esm import specifier ./c
		    | ./fixtures/esm/abc.js [depth 0] [orphan]
		    |   [no exports]
		    |   [no exports used]
		    |   ModuleConcatenation bailout: Module is an entry point
		  ./fixtures/esm/a.js [depth 1] [orphan]
		    [exports: a, default]
		    [only some exports used: a]
		    esm import ./a
		    esm import specifier ./a
		  ./fixtures/esm/b.js [depth 1] [orphan]
		    [exports: b, default]
		    [only some exports used: default]
		    esm import ./b
		    esm import specifier ./b
		  ./fixtures/esm/c.js [depth 1] [orphan]
		    [exports: c, default]
		    esm import ./c
		    esm import specifier ./c
		runtime modules
		  webpack/runtime/has_own_property {909}
		    [no exports]
		    [used exports unknown]
		  webpack/runtime/make_namespace_object {909}
		    [no exports]
		    [used exports unknown]
		  webpack/runtime/define_property_getters {909}
		    [no exports]
		    [used exports unknown]
		  
		2024-06-13 11:28:49: Rspack 0.7.2 compiled successfully in 70 ms (f6096f777ad767c7ff89)"
	`);
	}
};
