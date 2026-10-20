import { rspack } from "@rspack/core";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

// Logs `<root name>/src/<file>` so the check does not depend on how the temp
// directory resolves, such as /var and /private/var on macOS.
const COUNT_LOADER = `
const fs = require("node:fs");
const path = require("node:path");
module.exports = function (source) {
	const projectRoot = path.dirname(path.dirname(path.dirname(this.resourcePath)));
	fs.appendFileSync(
		this.getOptions().log,
		path.relative(projectRoot, this.resourcePath) + "\\n"
	);
	return source;
};
`;

function writeFixture(projectRoot) {
	fs.mkdirSync(path.join(projectRoot, "src"), { recursive: true });
	fs.writeFileSync(
		path.join(projectRoot, "src", "index.js"),
		'import value from "./value.js";\nconsole.log(value);\n'
	);
	fs.writeFileSync(
		path.join(projectRoot, "src", "value.js"),
		'export default "value";\n'
	);
}

function cacheDirectory(projectRoot) {
	return path.join(projectRoot, "node_modules", ".cache", "rspack");
}

async function build(projectRoot, { newCache, loader, log }) {
	fs.writeFileSync(log, "");
	const infrastructureLogs = [];
	const compiler = rspack({
		context: projectRoot,
		mode: "development",
		devtool: false,
		target: "node",
		entry: "./src/index.js",
		experiments: { newCache },
		cache: { type: "persistent" },
		infrastructureLogging: { level: "none" },
		module: {
			rules: [{ test: /\.js$/, use: [{ loader, options: { log } }] }]
		},
		output: { path: path.join(projectRoot, "dist"), filename: "main.js" }
	});
	compiler.hooks.infrastructureLog.tap("PersistentCacheOtherRoot", (name, type, args) => {
		infrastructureLogs.push(`${name} ${type} ${args.join(" ")}`);
	});

	let persistentCacheLog;
	try {
		await new Promise((resolve, reject) => {
			compiler.run((error, stats) => {
				if (error) return reject(error);
				if (stats.hasErrors()) {
					return reject(
						new Error(stats.toString({ all: false, errors: true, errorDetails: true }))
					);
				}
				persistentCacheLog = stats.toString({
					all: false,
					colors: false,
					logging: false,
					loggingDebug: /^rspack\.persistentCache$/
				});
				resolve();
			});
		});
	} finally {
		// Closing flushes the cache to disk and the pending native infrastructure logs.
		await new Promise((resolve, reject) =>
			compiler.close(error => (error ? reject(error) : resolve()))
		);
	}

	const builtModules = fs
		.readFileSync(log, "utf8")
		.split("\n")
		.filter(Boolean)
		.sort();
	return {
		builtModules,
		persistentCacheLog,
		infrastructureLog: infrastructureLogs.join("\n")
	};
}

function createCase({ backend, newCache, assertInvalidated }) {
	return {
		description: `should not restore a ${backend} persistent cache written under another root`,
		async run() {
			const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-cache-other-root-"));
			try {
				const loader = path.join(tempRoot, "count-loader.cjs");
				fs.writeFileSync(loader, COUNT_LOADER);
				const options = { newCache, loader, log: path.join(tempRoot, "built.log") };
				const rootA = path.join(tempRoot, "a");
				const rootB = path.join(tempRoot, "b");
				writeFixture(rootA);
				writeFixture(rootB);
				const modulesIn = rootName =>
					[
						path.join(rootName, "src", "index.js"),
						path.join(rootName, "src", "value.js")
					].sort();

				expect((await build(rootA, options)).builtModules).toEqual(modulesIn("a"));

				fs.cpSync(cacheDirectory(rootA), cacheDirectory(rootB), { recursive: true });
				const moved = await build(rootB, options);
				expect(moved.builtModules).toEqual(modulesIn("b"));
				assertInvalidated(moved);

				expect((await build(rootB, options)).builtModules).toEqual([]);
			} finally {
				fs.rmSync(tempRoot, { recursive: true, force: true });
			}
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
	createCase({
		backend: "legacy",
		newCache: false,
		assertInvalidated({ persistentCacheLog }) {
			expect(persistentCacheLog).toContain(
				"meta persistent cache recovery failed: persistent cache version does not match"
			);
			expect(persistentCacheLog).not.toContain("make persistent cache recovery succeeded");
		}
	}),
	createCase({
		backend: "new",
		newCache: true,
		assertInvalidated({ infrastructureLog }) {
			expect(infrastructureLog).toContain(
				"rspack.cache.IdleFileCache log Resetting cache, the cache version doesn't match"
			);
		}
	})
];
