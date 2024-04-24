"use strict";

// require("./helpers/warmup-webpack");

const path = require("path");
const fs = require("graceful-fs");
const webpack = require("..");
const prettyFormat = require("pretty-format").default;

const CWD_PATTERN = new RegExp(
	path.join(process.cwd(), "../../").replace(/\\/g, "/"),
	"gm"
);
const ERROR_STACK_PATTERN = /(?:\n\s+at\s.*)+/gm;

function cleanError(err) {
	const result = {};
	for (const key of Object.getOwnPropertyNames(err)) {
		result[key] = err[key];
	}

	if (result.message) {
		result.message = err.message.replace(ERROR_STACK_PATTERN, "");
	}

	if (result.stack) {
		result.stack = result.stack.replace(ERROR_STACK_PATTERN, "");
	}

	return result;
}

function serialize(received) {
	return prettyFormat(received, prettyFormatOptions)
		.replace(CWD_PATTERN, "<cwd>")
		.trim();
}

const prettyFormatOptions = {
	escapeRegex: false,
	printFunctionName: false,
	plugins: [
		{
			test(val) {
				return typeof val === "string";
			},
			print(val) {
				return `"${val
					.replace(/\\/gm, "/")
					.replace(/"/gm, '\\"')
					.replace(/\r?\n/gm, "\\n")}"`;
			}
		}
	]
};

expect.addSnapshotSerializer({
	test(received) {
		return received.errors || received.warnings;
	},
	print(received) {
		return serialize({
			errors: received.errors.map(cleanError),
			warnings: received.warnings.map(cleanError)
		});
	}
});

expect.addSnapshotSerializer({
	test(received) {
		return received.message;
	},
	print(received) {
		return serialize(cleanError(received));
	}
});

const defaults = {
	options: {
		context: path.resolve(__dirname, "fixtures", "errors"),
		mode: "none",
		devtool: false,
		optimization: {
			minimize: false
		}
	},
	outputFileSystem: {
		// CHANGE: rspack outputFileSystem `mkdirp` uses option `{ recursive: true }`, webpack's second parameter is alway a callback
		mkdir(dir, maybeOptionOrCallback, maybeCallback) {
			if (typeof maybeOptionOrCallback === "function") {
				maybeOptionOrCallback();
			} else if (typeof maybeCallback === "function") {
				maybeCallback();
			}
		},
		writeFile(file, content, callback) {
			callback();
		},
		stat(file, callback) {
			callback(new Error("ENOENT"));
		},
		mkdirSync() {},
		writeFileSync() {}
	}
};

async function compile(options) {
	const stats = await new Promise((resolve, reject) => {
		const compiler = webpack({ ...defaults.options, ...options });
		if (options.mode === "production") {
			if (options.optimization) options.optimization.minimize = true;
			else options.optimization = { minimize: true };
		}
		compiler.outputFileSystem = defaults.outputFileSystem;

		try {
			compiler.run((bailedError, stats) => {
				if (bailedError) {
					return reject(bailedError);
				}
				compiler.close(closeError => {
					if (closeError) {
						return reject(closeError);
					}
					resolve(stats);
				});
			});
		} catch (err) {
			// capture sync thrown errors
			reject(err);
		}
	});

	expect(typeof stats).toEqual("object");
	const statsResult = stats.toJson({ errorDetails: false });
	expect(typeof statsResult).toBe("object");
	const { errors, warnings } = statsResult;
	expect(Array.isArray(errors)).toBe(true);
	expect(Array.isArray(warnings)).toBe(true);

	return { errors, warnings };
}

it("should emit warnings for resolve failure in esm", async () => {
	await expect(
		compile({
			entry: "./resolve-fail-esm"
		})
	).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "formatted": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "message": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "moduleId": "./resolve-fail-esm/index.js",
		      "moduleIdentifier": "javascript/esm|<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "moduleName": "./resolve-fail-esm/index.js",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
});

describe("Testing proxy methods on errors", () => {
	it("test push", async () => {
		await expect(
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test push", compilation => {
							compilation.errors.push("test push");
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "formatted": "  × test push\\n",
		      "message": "  × test push\\n",
		    },
		    Object {
		      "formatted": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "message": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "moduleId": "./resolve-fail-esm/index.js",
		      "moduleIdentifier": "javascript/esm|<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "moduleName": "./resolve-fail-esm/index.js",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	});

	it("test pop", async () => {
		await expect(
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test pop", compilation => {
							compilation.errors.pop();
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [],
		}
	`);
	});

	it("test shift&unshift", async () => {
		await expect(
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap(
							"test shift and unshift",
							compilation => {
								compilation.errors.shift();
								compilation.errors.unshift("test unshift");
							}
						);
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "formatted": "  × test unshift\\n",
		      "message": "  × test unshift\\n",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	});

	it("test splice 1", async () => {
		await expect(
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.errors.splice(0, 1, "test splice");
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "formatted": "  × test splice\\n",
		      "message": "  × test splice\\n",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	});

	it("test splice 2", async () => {
		await expect(
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.errors.splice(0, 0, "test splice");
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "formatted": "  × test splice\\n",
		      "message": "  × test splice\\n",
		    },
		    Object {
		      "formatted": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "message": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		      "moduleId": "./resolve-fail-esm/index.js",
		      "moduleIdentifier": "javascript/esm|<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "moduleName": "./resolve-fail-esm/index.js",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	});
});

describe("Testing proxy methods on warnings", () => {
	it("test push", async () => {
		await expect(
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test push", compilation => {
							compilation.warnings.push(new Error("test push"));
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "formatted": "  ⚠ Error: test push\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:313:34\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		      "message": "  ⚠ Error: test push\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:313:34\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		    },
		    Object {
		      "formatted": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<cwd>packages/rspack/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		    },
		  ],
		}
	`);
	});

	it("test pop", async () => {
		await expect(
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test pop", compilation => {
							compilation.warnings.pop();
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [],
		}
	`);
	});

	it("test shift&unshift", async () => {
		await expect(
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap(
							"test shift and unshift",
							compilation => {
								compilation.warnings.shift();
								compilation.warnings.unshift(new Error("test unshift"));
							}
						);
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "formatted": "  ⚠ Error: test unshift\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:368:38\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		      "message": "  ⚠ Error: test unshift\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:368:38\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		    },
		  ],
		}
	`);
	});

	it("test splice 1", async () => {
		await expect(
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.warnings.splice(0, 1, new Error("test splice"));
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "formatted": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:394:42\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		      "message": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:394:42\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		    },
		  ],
		}
	`);
	});

	it("test splice 2", async () => {
		await expect(
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test splice", compilation => {
							compilation.warnings.splice(0, 0, new Error("test splice"));
						});
					}
				]
			})
		).resolves.toMatchInlineSnapshot(`
		Object {
		  "errors": Array [],
		  "warnings": Array [
		    Object {
		      "formatted": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:419:42\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		      "message": "  ⚠ Error: test splice\\n  │     at <cwd>packages/rspack/tests/Errors.test.js:419:42\\n  │     at Hook.eval [as callAsync] (eval at create (<cwd>node_modules/tapable/lib/HookCodeFactory.js:33:10), <anonymous>:9:1)\\n  │     at Hook.CALL_ASYNC_DELEGATE [as _callAsync] (<cwd>node_modules/tapable/lib/Hook.js:18:14)\\n  │     at <cwd>packages/rspack/dist/Compiler.js:419:41\\n  │     at <cwd>packages/rspack/dist/Compiler.js:745:65\\n",
		    },
		    Object {
		      "formatted": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		      "moduleId": "./require.main.require.js",
		      "moduleIdentifier": "<cwd>packages/rspack/tests/fixtures/errors/require.main.require.js",
		      "moduleName": "./require.main.require.js",
		    },
		  ],
		}
	`);
	});
});

describe("Testing map function on errors and warnings", () => {
	it("test map of warnings", async () => {
		const warnPromise = new Promise(resolve => {
			compile({
				entry: "./require.main.require",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap(
							"test warnings map",
							compilation => {
								resolve(
									compilation.warnings.map((item, index) => ({
										index,
										...item
									}))
								);
							}
						);
					}
				]
			});
		});
		await expect(warnPromise).resolves.toMatchInlineSnapshot(`
		Array [
		  Object {
		  "formatted": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		  "index": 0,
		  "message": "  ⚠ Module parse warning:\\n  ╰─▶   ⚠ Module parse failed: require.main.require() is not supported by Rspack.\\n         ╭────\\n       1 │ require.main.require('./file');\\n         · ──────────────────────────────\\n         ╰────\\n      \\n",
		  "moduleId": "./require.main.require.js",
		  "moduleIdentifier": "<cwd>packages/rspack/tests/fixtures/errors/require.main.require.js",
		  "moduleName": "./require.main.require.js",
		},
		]
	`);
	});

	it("test map of errors", async () => {
		const errorPromise = new Promise(resolve => {
			compile({
				entry: "./resolve-fail-esm",
				plugins: [
					compiler => {
						compiler.hooks.afterCompile.tap("test errors map", compilation => {
							resolve(
								compilation.errors.map((item, index) => ({ index, ...item }))
							);
						});
					}
				]
			});
		});
		await expect(errorPromise).resolves.toMatchInlineSnapshot(`
		Array [
		  Object {
		  "formatted": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		  "index": 0,
		  "message": "  × Resolve error: Can't resolve './answer' in '<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm'\\n   ╭────\\n 1 │ import { answer } from './answer';\\n   ·                        ──────────\\n   ╰────\\n  help: Did you mean './answer.js'?\\n        \\n        The request './answer' failed to resolve only because it was resolved as fully specified,\\n        probably because the origin is strict EcmaScript Module,\\n        e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"'.\\n        \\n        The extension in the request is mandatory for it to be fully specified.\\n        Add the extension to the request.\\n",
		  "moduleId": "./resolve-fail-esm/index.js",
		  "moduleIdentifier": "javascript/esm|<cwd>packages/rspack/tests/fixtures/errors/resolve-fail-esm/index.js",
		  "moduleName": "./resolve-fail-esm/index.js",
		},
		]
	`);
	});
});
