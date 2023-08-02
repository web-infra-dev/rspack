"use strict";

// require("./helpers/warmup-webpack");

const path = require("path");
const fs = require("graceful-fs");
const webpack = require("@rspack/core").rspack;
const prettyFormat = require("pretty-format").default;

const CWD_PATTERN = new RegExp(process.cwd().replace(/\\/g, "/"), "gm");
const ERROR_STACK_PATTERN = /(?:\n\s+at\s.*)+/gm;

function cleanError(err) {
	const result: any = {};
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
	print(received: any) {
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
		mkdir(dir, callback) {
			callback();
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
	const stats: any = await new Promise((resolve, reject) => {
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
		      "formatted": "error[internal]: Failed to resolve ./answer in javascript/esm|<cwd>/tests/fixtures/errors/resolve-fail-esm/index.js\\n = Did you mean 'answer.js'?\\n   BREAKING CHANGE: The request './answer' failed to resolve only because it was resolved as fully specified\\n   (probably because the origin is strict EcmaScript Module, e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\\"type\\": \\"module\\"').\\n   The extension in the request is mandatory for it to be fully specified.\\n   Add the extension to the request.\\n\\n",
		      "message": "Failed to resolve ./answer in javascript/esm|<cwd>/tests/fixtures/errors/resolve-fail-esm/index.js",
		      "title": "",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
});
