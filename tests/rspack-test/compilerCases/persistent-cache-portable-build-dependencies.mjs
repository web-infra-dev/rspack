import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CASE_NAME = "persistent-cache-portable-build-dependencies";
const COMPILE_SCRIPT = String.raw`
const fs = require("node:fs");
const rspack = require(process.argv[1]);
const options = JSON.parse(process.argv[2]);
const compiler = rspack(options);
compiler.run((error, stats) => {
	const buildError = error || (stats?.hasErrors()
		? new Error(stats.toString({ all: false, errors: true })) : null);
	const log = stats?.toString({
		all: false, colors: false, logging: false,
		loggingDebug: /^rspack\.persistentCache$/
	});
	const dependencies = stats ? [...stats.compilation.fileDependencies] : [];
	compiler.close(closeError => {
		if (buildError || closeError) {
			console.error(buildError || closeError);
			process.exitCode = 1;
		} else {
			fs.writeFileSync(process.argv[3], JSON.stringify({
				log, dependencies
			}));
		}
	});
});
`;

function compile(options, resultFile) {
	execFileSync(
		process.execPath,
		[
			"-e",
			COMPILE_SCRIPT,
			fileURLToPath(import.meta.resolve("@rspack/core")),
			JSON.stringify(options),
			resultFile
		],
		{ cwd: options.context, stdio: "pipe", windowsHide: true, timeout: 60_000 }
	);
	return JSON.parse(fs.readFileSync(resultFile, "utf8"));
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [false, true].map(readonly => ({
	name: readonly ? "readonly" : "writable",
	description: `should relocate portable build dependencies with readonly=${readonly}`,
	run() {
		// Match the resolver's real paths, including macOS /var -> /private/var.
		const root = fs.realpathSync(
			fs.mkdtempSync(path.join(os.tmpdir(), `${CASE_NAME}-`))
		);
		try {
			const first = path.join(root, "first");
			const second = path.join(root, "second");
			const seedCache = path.join(root, "seed-cache");
			const events = path.join(root, "loader-events");
			const resultFile = path.join(root, "result.json");
			fs.mkdirSync(first, { recursive: true });
			fs.writeFileSync(
				path.join(first, "index.js"),
				'console.log("portable");\n'
			);
			fs.writeFileSync(
				path.join(first, "package.json"),
				'{"name":"portable-fixture"}\n'
			);
			fs.writeFileSync(
				path.join(first, "build.config.json"),
				'{"value":"initial"}\n'
			);
			fs.writeFileSync(
				path.join(first, "probe-loader.cjs"),
				`const fs = require("node:fs");
const path = require("node:path");
module.exports = function(source) {
	fs.appendFileSync(${JSON.stringify(events)}, "called\\n");
	// Read only as a build dependency, not as a normal module file dependency.
	const config = JSON.parse(fs.readFileSync(path.join(this.rootContext, "build.config.json"), "utf8"));
	return source + "\\nconsole.log(" + JSON.stringify(config.value) + ");\\n";
};\n`
			);
			fs.cpSync(first, second, { recursive: true, preserveTimestamps: true });

			const options = (app, cache, readOnly) => ({
				context: app,
				mode: "development",
				devtool: false,
				entry: "./index.js",
				incremental: false,
				experiments: { newCache: false },
				output: { path: path.join(app, "dist"), filename: "main.js" },
				// Use include instead of RegExp so options survive JSON serialization.
				module: {
					rules: [
						{
							include: path.join(app, "index.js"),
							use: [path.join(app, "probe-loader.cjs")]
						}
					]
				},
				cache: {
					type: "persistent",
					portable: true,
					readonly: readOnly,
					buildDependencies: ["package.json", "build.config.json"].map(file =>
						path.join(app, file)
					),
					storage: { type: "filesystem", directory: cache }
				}
			});
			const run = (app, cache, readOnly) =>
				compile(options(app, cache, readOnly), resultFile);
			const calls = () =>
				fs.readFileSync(events, "utf8").trim().split("\n").length;
			const assertHit = result => {
				expect(result.log).toContain("build dependencies are valid");
				expect(result.log).toContain(
					"snapshot restored with no changed dependencies"
				);
				expect(result.log).toContain(
					"make persistent cache recovery succeeded"
				);
				expect(result.log).not.toContain("cache invalidated");
			};

			run(first, seedCache, false);
			expect(calls()).toBe(1);
			const sameCache = path.join(root, "same-cache");
			fs.cpSync(seedCache, sameCache, { recursive: true });
			assertHit(run(first, sameCache, true));
			expect(calls()).toBe(1);

			const movedCache = path.join(root, "moved-cache");
			fs.cpSync(seedCache, movedCache, { recursive: true });
			const relocated = run(second, movedCache, readonly);
			assertHit(relocated);
			expect(calls()).toBe(1);
			expect(relocated.dependencies).toContain(path.join(second, "index.js"));
			expect(
				relocated.dependencies.some(file => file.startsWith(first + path.sep))
			).toBe(false);
			expect(
				fs.readFileSync(path.join(second, "dist/main.js"), "utf8")
			).toContain('"initial"');

			// The old checkout must not be needed after relocation either.
			fs.rmSync(first, { recursive: true, force: true });
			assertHit(run(second, movedCache, readonly));
			expect(calls()).toBe(1);

			fs.writeFileSync(
				path.join(second, "build.config.json"),
				'{"value":"updated"}\n'
			);
			const changed = run(second, movedCache, readonly);
			expect(changed.log).toContain(
				"persistent cache invalidated because build dependencies changed"
			);
			expect(calls()).toBe(2);
			expect(
				fs.readFileSync(path.join(second, "dist/main.js"), "utf8")
			).toContain('"updated"');
			if (readonly) {
				expect(changed.log).toContain(
					"persistent cache is readonly, stale entries will not be rewritten"
				);
				expect(run(second, movedCache, true).log).toContain(
					"persistent cache invalidated because build dependencies changed"
				);
				expect(calls()).toBe(3);
			} else {
				assertHit(run(second, movedCache, false));
				expect(calls()).toBe(2);
			}
		} finally {
			fs.rmSync(root, { recursive: true, force: true });
		}
	}
}));
