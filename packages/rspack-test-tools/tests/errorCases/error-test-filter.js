let errors = [];

/** @type {import('../..').TErrorCaseConfig} */
module.exports = {
    description:
        "Testing map function on errors and warnings: test map of errors",
    options() {
        return {
            entry: "./resolve-fail-esm",
            plugins: [
                compiler => {
                    compiler.hooks.afterCompile.tap("test errors map", compilation => {
                        compilation.errors.push(new Error(""));
                        compilation.errors = compilation.errors.filter(item => item.message);

                        errors = compilation.errors.map((item, index) => {
                            item.index = index;
                            return item;
                        });
                    });
                }
            ]
        };
    },
    async check() {
        expect(errors).toMatchInlineSnapshot(`
		Array [
		  Object {
		  "index": 0,
		  "loc": Object {
		    "end": Object {
		      "column": 33,
		      "line": 1,
		    },
		    "start": Object {
		      "column": 0,
		      "line": 1,
		    },
		  },
		  "message": "Module not found: Can't resolve './answer' in '<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm'",
		  "module": NormalModule {
		    "buildInfo": KnownBuildInfo {},
		    "buildMeta": Object {},
		    "context": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm",
		    "factoryMeta": Object {},
		    "layer": undefined,
		    "loaders": Array [],
		    "matchResource": undefined,
		    "rawRequest": "./resolve-fail-esm",
		    "request": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		    "resource": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		    "resourceResolveData": ReadonlyResourceData {
		      "fragment": "",
		      "path": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "query": "",
		      "resource": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		    },
		    "type": "javascript/esm",
		    "useSimpleSourceMap": false,
		    "useSourceMap": false,
		    "userRequest": "<TEST_TOOLS_ROOT>/tests/fixtures/errors/resolve-fail-esm/index.js",
		  },
		  "name": "Error",
		  "stack": undefined,
		},
		]
	`);
    }
};
