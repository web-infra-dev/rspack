import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

const LOADER = String.raw`
const fs = require("node:fs");
const path = require("node:path");
module.exports = function (source) {
	fs.appendFileSync(path.join(this.rootContext, "loader-calls.txt"), path.basename(this.resourcePath) + "\n");
	this.experiments.portableOutput({ paths: [] });
	if (!source.includes("quiet")) {
		for (const message of ["second warning", "first warning", "second warning"]) {
			const warning = new Error(message + (source.includes("root-sensitive") ? " " + this.resourcePath : ""));
			warning.stack = "";
			this.emitWarning(warning);
		}
		if (source.includes("emit-error")) {
			const error = new Error("loader error");
			error.stack = "";
			this.emitError(error);
		}
	}
	return source;
};
`;

const buildsByCompiler = new WeakMap();

function writeProject(root, error) {
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.writeFileSync(path.join(root, "package.json"), '{"name":"loader-diagnostics"}');
	fs.writeFileSync(path.join(root, "loader.cjs"), LOADER);
	fs.writeFileSync(path.join(root, "index.js"), 'import "./reusable.js"; import "./bound.js"; import "./quiet.js";');
	fs.writeFileSync(path.join(root, "reusable.js"), `module.exports = 1; // ${error ? "emit-error" : "warning"}`);
	fs.writeFileSync(path.join(root, "bound.js"), 'module.exports = 2; // root-sensitive');
	fs.writeFileSync(path.join(root, "quiet.js"), 'module.exports = 3; // quiet');
}

function createCompiler(context, { cache = { type: "persistent" }, seedFrom, incremental = false } = {}) {
	const builds = [];
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		incremental,
		entry: "./index.js",
		output: { path: path.join(context, "dist") },
		cache,
		experiments: { portableCache: seedFrom ? { seedFrom } : true },
		module: { rules: [{ test: /[\\/](reusable|bound|quiet)\.js$/, loader: "./loader.cjs" }] },
		plugins: [(compiler) => {
			compiler.hooks.thisCompilation.tap("diagnostic-hook", (compilation) => {
				compilation.hooks.buildModule.tap("diagnostic-hook", (module) => {
					builds.push(path.basename(module.resource || ""));
				});
				const warning = new Error("fresh hook warning");
				warning.stack = "";
				compilation.warnings.push(warning);
			});
		}],
		infrastructureLogging: { level: "error" }
	});
	buildsByCompiler.set(compiler, builds);
	return compiler;
}

async function run(compiler, changes) {
	const context = compiler.context;
	const builds = buildsByCompiler.get(compiler);
	builds.length = 0;
	fs.writeFileSync(path.join(context, "loader-calls.txt"), "");
	const stats = changes ? await new Promise((resolve, reject) => {
		compiler.run((error, stats) => error ? reject(error) : resolve(stats), changes);
	}) : await runCompiler(compiler);
	return {
		...stats.toJson({ all: false, errors: true, warnings: true }),
		builds: [...builds],
		calls: fs.readFileSync(path.join(context, "loader-calls.txt"), "utf8").trim().split("\n").filter(Boolean)
	};
}

async function build(root, options) {
	const compiler = createCompiler(root, options);
	try {
		return await run(compiler);
	} finally {
		await closeCompiler(compiler);
	}
}

function expectDiagnostics(actual, expected) {
	expect(actual.errors).toEqual(expected.errors);
	expect(actual.warnings).toEqual(expected.warnings);
}

const CHAIN_LOADER = String.raw`
const path = require("node:path");
module.exports = function (source) {
	const warning = new Error("warning " + path.basename(this.resourcePath));
	warning.stack = "";
	this.emitWarning(warning);
	return source;
};
`;

// Each module of the chain is found only after its importer is built, so a cold build
// publishes the loader warnings in chain order. A plugin reports them in that order.
async function buildChain(root) {
	const compiler = rspack({
		context: root,
		mode: "development",
		devtool: false,
		incremental: false,
		entry: "./p0.js",
		output: { path: path.join(root, "dist") },
		cache: { type: "persistent" },
		experiments: { portableCache: true },
		module: { rules: [{ test: /[\\/]p\d\.js$/, loader: "./loader.cjs" }] },
		plugins: [(compiler) => {
			compiler.hooks.thisCompilation.tap("diagnostic-report", (compilation) => {
				compilation.hooks.processAssets.tap({
					name: "diagnostic-report",
					stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT
				}, () => {
					compilation.emitAsset("diagnostics.json", new rspack.sources.RawSource(
						JSON.stringify(compilation.warnings.map((warning) => warning.message.match(/warning p\d\.js/)?.[0]))
					));
				});
			});
		}],
		infrastructureLogging: { level: "error" }
	});
	try {
		await runCompiler(compiler);
		return fs.readFileSync(path.join(root, "dist", "diagnostics.json"), "utf8");
	} finally {
		await closeCompiler(compiler);
	}
}

const chainOrder = {
	name: "retained-warning-order",
	description: "publishes retained loader warnings in the order a cold build emits them",
	async run() {
		const root = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-loader-diagnostics-order-")));
		try {
			fs.mkdirSync(path.join(root, ".git"));
			fs.writeFileSync(path.join(root, "package.json"), '{"name":"loader-diagnostics-order"}');
			fs.writeFileSync(path.join(root, "loader.cjs"), CHAIN_LOADER);
			const length = 8;
			for (let index = 0; index < length; index++) {
				fs.writeFileSync(path.join(root, `p${index}.js`), `${index + 1 < length ? `import "./p${index + 1}.js";\n` : ""}console.log(${index});`);
			}
			const cold = await buildChain(root);
			expect(JSON.parse(cold)).toEqual(Array.from({ length }, (_, index) => `warning p${index}.js`));
			expect(await buildChain(root)).toBe(cold);
		} finally {
			fs.rmSync(root, { recursive: true, force: true });
		}
	}
};

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [...[false, true].map((error) => ({
	name: error ? "emitted-error" : "emitted-warnings",
	description: "retains loader diagnostics when persistent graph reuse skips the loader",
	async run() {
		const temp = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-loader-diagnostics-")));
		try {
			const donor = path.join(temp, "donor");
			const reader = path.join(temp, "longer-reader-根-😀");
			writeProject(donor, error);
			writeProject(reader, error);
			const cold = await build(donor);
			expect(cold.calls.sort()).toEqual(["bound.js", "quiet.js", "reusable.js"]);
			expect(cold.warnings).toHaveLength(7);
			expect(cold.errors).toHaveLength(error ? 1 : 0);
			// Keep emission order and intentional duplicate warnings within each module.
			expect(cold.warnings[0].message).toContain("fresh hook warning");
			for (let index = 0; index < 6; index++) {
				expect(cold.warnings[index + 1].message).toContain(index % 3 === 1 ? "first warning" : "second warning");
			}
			for (let iteration = 0; iteration < 2; iteration++) {
				const warm = await build(donor);
				expect(warm.calls).toEqual([]);
				expect(warm.builds).toEqual([]);
				expectDiagnostics(warm, cold);
			}
			const moved = await build(reader, {
				seedFrom: path.join(donor, "node_modules/.cache/rspack/development")
			});
			// emitWarning's formatted loader path also contains the writer's root. Rebuild its
			// owners on relocation, while the diagnostic-free module keeps its cached build.
			expect(moved.calls.sort()).toEqual(["bound.js", "reusable.js"]);
			const readerCold = await build(reader, { cache: false });
			expectDiagnostics(moved, readerCold);
			const readerWarm = await build(reader);
			expect(readerWarm.calls).toEqual([]);
			expect(readerWarm.builds).toEqual([]);
			expectDiagnostics(readerWarm, readerCold);

			// Incremental rebuilds reuse the same diagnostic owner without accumulating copies.
			const compiler = createCompiler(reader, { incremental: true });
			try {
				for (let iteration = 0; iteration < 3; iteration++) {
					const result = await run(compiler);
					expect(result.calls).toEqual([]);
					expect(result.builds).toEqual([]);
					expectDiagnostics(result, readerCold);
				}
				const reusable = path.join(reader, "reusable.js");
				fs.writeFileSync(reusable, 'module.exports = 1; // quiet');
				const quiet = await run(compiler, { modifiedFiles: new Set([reusable]) });
				expect(quiet.calls).toEqual(["reusable.js"]);
				expect(quiet.errors).toEqual([]);
				expect(quiet.warnings).toEqual(readerCold.warnings.filter((warning) =>
				!warning.moduleName?.includes("reusable.js")));
				const quietAgain = await run(compiler);
				expect(quietAgain.calls).toEqual([]);
				expectDiagnostics(quietAgain, quiet);
				// Removing an owner also removes its cached diagnostics.
				const entry = path.join(reader, "index.js");
				fs.writeFileSync(entry, 'import "./reusable.js";');
				const removed = await run(compiler, { modifiedFiles: new Set([entry]) });
				expect(removed.calls).toEqual([]);
				expect(removed.errors).toEqual([]);
				expect(removed.warnings).toEqual([readerCold.warnings[0]]);
			} finally {
				await closeCompiler(compiler);
			}
		} finally {
			fs.rmSync(temp, { recursive: true, force: true });
		}
	}
})), chainOrder];
