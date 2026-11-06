import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

const RUNS = "__portableSnapshotOwnersLoaderRuns";

async function build(context, resolve, cache = { type: "persistent" }) {
	globalThis[RUNS] = [];
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		incremental: false,
		devtool: "source-map",
		entry: "./src/index.js",
		cache,
		experiments: { portableCache: { seed: false } },
		resolve,
		module: { rules: [{ test: /\.js$/, loader: "./loader.cjs" }] },
		output: {
			path: path.join(context, "dist"),
			library: { type: "commonjs2" },
			clean: true
		}
	});
	try {
		const stats = await runCompiler(compiler);
		const diagnostics = stats.toJson({ all: false, errors: true, warnings: true });
		expect(diagnostics.errors).toEqual([]);
		const assets = fs.readdirSync(path.join(context, "dist")).sort().map(name => [
			name, fs.readFileSync(path.join(context, "dist", name))
		]);
		const runtime = { exports: {} };
		new Function("module", "exports", assets.find(([name]) => name === "main.js")[1].toString())(
			runtime, runtime.exports);
		return {
			assets,
			diagnostics,
			values: await runtime.exports.default,
			runs: globalThis[RUNS].map(file => path.relative(context, file)).sort()
		};
	} finally {
		await closeCompiler(compiler);
		delete globalThis[RUNS];
	}
}

function project(root, source) {
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.mkdirSync(path.join(root, "src"));
	fs.writeFileSync(path.join(root, "package.json"), '{"name":"snapshot-owners"}');
	fs.writeFileSync(path.join(root, "loader.cjs"), `module.exports = function(source, map) {
globalThis[${JSON.stringify(RUNS)}].push(this.resourcePath);
this.callback(null, source, map);
};`);
	fs.writeFileSync(path.join(root, "src/shared.js"), "export default 7;");
	fs.writeFileSync(path.join(root, "src/index.js"), source);
}

function same(result, cold) {
	expect(result.values).toEqual(cold.values);
	expect(result.assets).toEqual(cold.assets);
	expect(result.diagnostics).toEqual(cold.diagnostics);
}

async function warm(root, resolve, cold) {
	const result = await build(root, resolve);
	expect(result.runs).toEqual([]);
	same(result, cold);
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "isolates fixed factory expiry from independently validated layout build inputs",
	async run() {
		const temp = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-snapshot-owners-")));
		try {
			for (const fixedFirst of [true, false]) {
				for (const changed of [false, true]) {
					const base = path.join(temp, `${fixedFirst}-${changed}`);
					const writer = path.join(base, "a/project");
					const reader = path.join(base, "b/project");
					const imports = ["import fixed from 'choice';", "import relative from './shared.js';"];
					if (!fixedFirst) imports.reverse();
					project(writer, `${imports.join("\n")}\nexport default [fixed, relative];`);
					const resolve = { alias: { choice: path.join(writer, "src/shared.js") } };
					const initial = await build(writer, resolve);
					expect(initial.values).toEqual([7, 7]);
					await warm(writer, resolve, initial);
					fs.mkdirSync(path.dirname(reader), { recursive: true });
					fs.cpSync(writer, reader, { recursive: true });
					if (changed) fs.writeFileSync(path.join(reader, "src/shared.js"), "export default 99;");
					const moved = await build(reader, resolve);
					expect(moved.values).toEqual([7, changed ? 99 : 7]);
					expect(moved.runs).not.toContain(path.join("src", "index.js"));
					if (changed) expect(moved.runs).toContain(path.join("src", "shared.js"));
					else expect(moved.runs).not.toContain(path.join("src", "shared.js"));
					const cold = await build(reader, resolve, false);
					same(moved, cold);
					await warm(reader, resolve, cold);
					// A partial save must keep the independently validated shared-file snapshot.
					fs.appendFileSync(path.join(reader, "src/index.js"), "\nexport const revision = 1;");
					const partial = await build(reader, resolve);
					expect(partial.runs).toEqual([path.join("src", "index.js")]);
					await warm(reader, resolve, await build(reader, resolve, false));
					fs.writeFileSync(path.join(reader, "src/shared.js"), "export default 101;");
					const updated = await build(reader, resolve);
					expect(updated.values).toEqual([7, 101]);
					expect(updated.runs).toEqual([path.join("src", "shared.js")]);
					same(updated, await build(reader, resolve, false));
				}
			}
			// A fixed fallback and an issuer-derived missing lookup share paths. Creating the
			// reader's ordinary package must still refactorize, keeping the importer build.
			const writer = path.join(temp, "missing/a/project");
			const reader = path.join(temp, "missing/b/project");
			project(writer, "import value from 'optional'; export default value;");
			const resolve = { fallback: { optional: path.join(writer, "src/shared.js") } };
			await warm(writer, resolve, await build(writer, resolve));
			fs.mkdirSync(path.dirname(reader), { recursive: true });
			fs.cpSync(writer, reader, { recursive: true });
			const packageDir = path.join(reader, "node_modules/optional");
			fs.mkdirSync(packageDir, { recursive: true });
			fs.writeFileSync(path.join(packageDir, "package.json"), '{"main":"index.js"}');
			fs.writeFileSync(path.join(packageDir, "index.js"), "module.exports = 99;");
			const created = await build(reader, resolve);
			expect(created.values).toBe(99);
			expect(created.runs).not.toContain(path.join("src", "index.js"));
			const cold = await build(reader, resolve, false);
			same(created, cold);
			await warm(reader, resolve, cold);
		} finally {
			fs.rmSync(temp, { recursive: true, force: true });
		}
	}
};
