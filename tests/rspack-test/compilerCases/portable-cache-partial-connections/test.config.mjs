import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

const LOADER_RUNS = "__portablePartialConnectionsLoaderRuns";
const COUNT = 32;
const importer = (i, revision = 0) => {
	const request = i % 2 === 0 ? `fixed-${i}` : `./value-${i}.js`;
	return `import value from ${JSON.stringify(request)};
import * as grouped from ${JSON.stringify(request)};
export default [value, grouped.default, ${revision}];
${i === 0 ? `import { absent } from ${JSON.stringify(request)}; export const warning = absent;` : ""}`;
};

async function build(context, aliases, cache = { type: "persistent" }) {
	globalThis[LOADER_RUNS] = [];
	const compiler = rspack({
		context,
		mode: "development",
		target: "node",
		devtool: "source-map",
		incremental: false,
		entry: "./src/index.js",
		output: {
			path: path.join(context, "dist"),
			library: { type: "commonjs2" },
			clean: true
		},
		cache,
		experiments: { portableCache: { seed: false } },
		resolve: { alias: aliases },
		module: {
			parser: { javascript: { exportsPresence: "warn" } },
			rules: [{ test: /\.js$/, loader: "./loader.cjs" }]
		}
	});
	try {
		const stats = await runCompiler(compiler);
		const diagnostics = stats.toJson({ all: false, errors: true, warnings: true });
		expect(diagnostics.errors).toEqual([]);
		expect(diagnostics.warnings).toEqual([expect.objectContaining({
		message: expect.stringContaining("'absent'")
	})]);
		const logging = stats.toJson({
			all: false,
			logging: "verbose",
			loggingDebug: /^rspack\.persistentCache$/
		}).logging;
		const logs = Object.values(logging ?? {}).flatMap(entry =>
			(entry.entries ?? []).map(line => line.message));
		const invalid = logs.filter(line => line.includes("inconsistent make cache"));
		if (invalid.length) throw new Error(invalid.join("\n"));
		const assets = fs.readdirSync(path.join(context, "dist")).sort().map(name => [
			name, fs.readFileSync(path.join(context, "dist", name))
		]);
		const runtime = { exports: {} };
		new Function("module", "exports", assets.find(([name]) => name === "main.js")[1].toString())(
			runtime, runtime.exports);
		return {
			assets,
			diagnostics,
			values: runtime.exports.default,
			built: [...stats.compilation.builtModules]
				.map(module => path.relative(context, module.resource)).sort(),
			loaderRuns: globalThis[LOADER_RUNS]
				.map(resource => path.relative(context, resource)).sort()
		};
	} finally {
		await closeCompiler(compiler);
		delete globalThis[LOADER_RUNS];
	}
}

function expectSame(result, cold) {
	expect(result.assets).toEqual(cold.assets);
	expect(result.diagnostics).toEqual(cold.diagnostics);
	expect(result.values).toEqual(cold.values);
}

function expectWarm(result, cold) {
	expect(result.built).toEqual([]);
	expect(result.loaderRuns).toEqual([]);
	expectSame(result, cold);
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "recovers connections from partial graph saves across generations and relocation",
	async run() {
		const temp = fs.realpathSync.native(fs.mkdtempSync(
			path.join(os.tmpdir(), "rspack-partial-connections-")));
		try {
			const writer = path.join(temp, "a", "project");
			const reader = path.join(temp, "b", "project");
			fs.mkdirSync(path.join(writer, ".git"), { recursive: true });
			fs.mkdirSync(path.join(writer, "src"));
			fs.writeFileSync(path.join(writer, "package.json"), '{"name":"partial-connections"}');
			fs.writeFileSync(path.join(writer, "loader.cjs"), `module.exports = function(source) {
globalThis[${JSON.stringify(LOADER_RUNS)}].push(this.resourcePath);
return source;
};`);
			const aliases = {};
			for (let i = 0; i < COUNT; i++) {
				fs.writeFileSync(path.join(writer, "src", `importer-${i}.js`), importer(i));
				fs.writeFileSync(path.join(writer, "src", `value-${i}.js`), `export default ${i};`);
				if (i % 2 === 0) aliases[`fixed-${i}`] = path.join(writer, "src", `value-${i}.js`);
			}
			fs.writeFileSync(path.join(writer, "src", "index.js"), [
				...Array.from({ length: COUNT }, (_, i) => `import v${i} from './importer-${i}.js';`),
				`export default [${Array.from({ length: COUNT }, (_, i) => `v${i}`).join(",")}];`
			].join("\n"));
			const compile = (root, cache) => build(root, aliases, cache);
			const initial = await compile(writer);
			expect(initial.built).toHaveLength(1 + COUNT * 2);
			expect(initial.loaderRuns).toEqual(initial.built);
			expect(initial.values).toEqual(Array.from({ length: COUNT }, (_, i) => [i, i, 0]));
			expectWarm(await compile(writer), initial);

			const revisions = Array(COUNT).fill(0);
			async function change(root, indices) {
				for (const i of indices) {
					revisions[i]++;
					fs.writeFileSync(path.join(root, "src", `importer-${i}.js`), importer(i, revisions[i]));
				}
				const partial = await compile(root);
				const changed = indices.map(i => path.join("src", `importer-${i}.js`)).sort();
				expect(partial.built).toEqual(changed);
				expect(partial.loaderRuns).toEqual(changed);
				expect(partial.values).toEqual(revisions.map((revision, i) => [i, i, revision]));
				const cold = await compile(root, false);
				expect(cold.built).toHaveLength(1 + COUNT * 2);
				expectSame(partial, cold);
				expectWarm(await compile(root), cold);
			}
			// Each partial save keeps most nodes from older compilations. Their serialized
			// connection indices need not be unique across these generations.
			await change(writer, [1]);
			await change(writer, [3, 5]);
			await change(writer, [7, 9, 11]);
			fs.mkdirSync(path.dirname(reader), { recursive: true });
			fs.cpSync(writer, reader, { recursive: true });
			// The same absolute aliases still resolve in the writer, while relative
			// requests move to the reader. Saving that repair adds another generation.
			const moved = await compile(reader);
			const fixedTargets = Array.from({ length: COUNT / 2 }, (_, i) =>
				path.relative(reader, path.join(writer, "src", `value-${i * 2}.js`))).sort();
			expect(moved.built).toEqual(fixedTargets);
			expect(moved.loaderRuns).toEqual(fixedTargets);
			const cold = await compile(reader, false);
			expect(cold.built).toHaveLength(1 + COUNT * 2);
			expectSame(moved, cold);
			expectWarm(await compile(reader), cold);
			await change(reader, [13, 15]);
			await change(reader, [17, 19, 21]);
		} finally {
			removeTree(temp);
		}
	}
};
