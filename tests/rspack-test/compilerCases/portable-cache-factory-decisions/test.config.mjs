import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

const transform = "./transform.cjs";

async function build(context, configuration, cache = { type: "persistent", portable: true }) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		incremental: false,
		target: "node",
		entry: "./src/index.js",
		output: {
			path: path.join(context, "dist"),
			filename: "main.js",
			library: { type: "commonjs2" }
		},
		optimization: { minimize: false },
		infrastructureLogging: { level: "error" },
		...configuration,
		cache
	});
	try {
		const stats = await runCompiler(compiler);
		expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
		return {
			assets: Object.fromEntries(stats.compilation.getAssets().map(({ name, source }) => [
				name, source.source().toString()
			])),
			built: [...stats.compilation.builtModules].map((module) => module.identifier())
		};
	} finally {
		await closeCompiler(compiler);
	}
}

function copyTree(from, to) {
	fs.mkdirSync(to, { recursive: true });
	for (const entry of fs.readdirSync(from, { withFileTypes: true })) {
		const source = path.join(from, entry.name);
		const target = path.join(to, entry.name);
		if (entry.isDirectory()) copyTree(source, target);
		else fs.copyFileSync(source, target);
	}
}

async function withProjects(files, run) {
	const temp = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-factory-")));
	const writer = path.join(temp, "writer-only", "project");
	const reader = path.join(temp, "reader", "project");
	try {
		fs.mkdirSync(path.join(writer, ".git"), { recursive: true });
		for (const [name, source] of Object.entries({
			"package.json": '{"name":"factory-decisions"}',
			"transform.cjs": 'module.exports = source => source.replaceAll("original", "transformed");',
			"src/index.js": 'module.exports = require("./dep.js");',
			"src/dep.js": 'module.exports = "original";',
			...files
		})) {
			const file = path.join(writer, name);
			fs.mkdirSync(path.dirname(file), { recursive: true });
			fs.writeFileSync(file, source);
		}
		await run(writer, reader);
	} finally {
		fs.rmSync(temp, { recursive: true, force: true });
	}
}

async function expectColdEquality(reader, configuration, moved) {
	fs.rmSync(path.join(reader, "node_modules", ".cache"), { recursive: true, force: true });
	const cold = await build(reader, configuration);
	const uncached = await build(reader, configuration, false);
	expect(moved.assets).toEqual(cold.assets);
	expect(moved.assets).toEqual(uncached.assets);
}

function beforeResolve(callback) {
	return {
		apply(compiler) {
			compiler.hooks.normalModuleFactory.tap("FactoryDecisions", (factory) => {
				factory.hooks.beforeResolve.tap("FactoryDecisions", callback);
			});
		}
	};
}

const replacements = { "src/dep2.js": 'module.exports = "replacement";' };
const scenarios = [
	{
		name: "ancestor-include",
		rules: (writer) => [{ test: /dep\.js$/, include: path.dirname(writer), use: [transform] }]
	},
	{
		name: "regex-include",
		rules: () => [{ test: /dep\.js$/, include: /writer-only/, use: [transform] }]
	},
	{
		name: "regex-issuer",
		rules: () => [{ test: /dep\.js$/, issuer: /writer-only/, use: [transform] }]
	},
	{
		name: "nested-or",
		rules: () => [{ test: /dep\.js$/, include: { or: [/never/, { or: [/writer-only/] }] }, use: [transform] }]
	},
	{
		name: "function-use",
		rules: (writer) => [{ test: /dep\.js$/, use: (info) => info.resource.startsWith(writer + path.sep) ? [transform] : [] }]
	},
	{
		name: "function-use-options",
		files: {
			"options.cjs": "module.exports = function () { return 'module.exports = ' + JSON.stringify(this.getOptions().value); };"
		},
		rules: (writer) => [{ test: /dep\.js$/, use: (info) => [{
			loader: "./options.cjs",
			options: { value: info.resource.startsWith(writer + path.sep) ? "writer" : "reader" }
		}] }]
	},
	{
		name: "function-use-loader-filter",
		rules: (writer) => [{ test: /dep\.js$/, use: (info) => [{
			loader: transform,
			codeFilter: info.resource.startsWith(writer + path.sep) ? "original" : "never"
		}] }]
	},
	{
		name: "parser-override-strict",
		files: { "src/dep.js": "module.exports = (function () { return this === undefined; })();" },
		rules: (writer) => [{ test: /dep\.js$/, include: writer, parser: { overrideStrict: "strict" } }]
	},
	{
		name: "parser-dynamic-import-mode",
		files: {
			"src/dep.js": 'module.exports = import("./lazy.js");',
			"src/lazy.js": 'export default "lazy";'
		},
		rules: (writer) => [{ test: /dep\.js$/, include: writer, parser: { dynamicImportMode: "eager" } }]
	},
	{
		name: "generator-one-of",
		files: { "src/dep.js": 'module.exports = require("./value.txt");', "src/value.txt": "original" },
		rules: (writer) => [{ oneOf: [
			{ test: /\.txt$/, include: writer, type: "asset/inline", generator: { dataUrl: { mimetype: "text/x-writer" } } },
			{ test: /\.txt$/, type: "asset/inline", generator: { dataUrl: { mimetype: "text/x-reader" } } }
		] }]
	},
	{
		name: "replacement-regex",
		files: replacements,
		plugins: () => [new rspack.NormalModuleReplacementPlugin(/writer-only.*dep\.js$/, "./dep2.js")]
	},
	{
		name: "replacement-function",
		files: replacements,
		plugins: (writer) => [new rspack.NormalModuleReplacementPlugin(/dep\.js$/, (data) => {
			if (data.context.startsWith(writer + path.sep)) data.request = "./dep2.js";
		})]
	},
	{
		name: "ignore-check-resource",
		files: {
			"src/dep.js": 'try { module.exports = require("./optional.js"); } catch { module.exports = "ignored"; }',
			"src/optional.js": 'module.exports = "optional";'
		},
		plugins: (writer) => [new rspack.IgnorePlugin({ checkResource: (request, context) => context.startsWith(writer + path.sep) && request === "./optional.js" })]
	},
	{
		name: "shared-factory-hook",
		files: {
			"src/dep.js": 'module.exports = require("shared-target");',
			"node_modules/shared-target/package.json": '{"name":"shared-target","version":"1.0.0","main":"index.js"}',
			"node_modules/shared-target/index.js": 'module.exports = "shared";'
		},
		plugins: () => [new rspack.sharing.ProvideSharedPlugin({ provides: ["shared-target"] })]
	},
	{
		name: "before-resolve",
		files: replacements,
		plugins: (writer) => [beforeResolve((data) => {
			if (data.request === "./dep.js" && data.context.startsWith(writer + path.sep)) data.request = "./dep2.js";
		})]
	}
];

export default [
	...scenarios.map(({ name, files = {}, rules, plugins }) => ({
		name,
		description: `decides ${name} at the reader of a relocated cache`,
		async run() {
			await withProjects(files, async (writer, reader) => {
				const configuration = { module: { rules: rules?.(writer) ?? [] }, plugins: plugins?.(writer) ?? [] };
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	})),
	{
		name: "reuse",
		description: "keeps unchanged function decisions, mjs, type-module packages and side-effect-only ESM imports without builds",
		async run() {
			await withProjects({
				"src/index.js": 'import { value } from "./dep.js"; import { m } from "./b.mjs"; import { c } from "./pkg/c.js"; import "./pkg/unused.js"; console.log(value, m, c);',
				"src/dep.js": 'export const value = "original";',
				"src/b.mjs": 'export const m = "m";',
				"src/pkg/package.json": '{"type":"module","sideEffects":false}',
				"src/pkg/c.js": 'import { d } from "./d.js"; export const c = d;',
				"src/pkg/d.js": 'export const d = "d";',
				"src/pkg/unused.js": 'throw new Error("side-effect-free");'
			}, async (writer, reader) => {
				const configuration = {
					module: { rules: [{ test: /dep\.js$/, exclude: (resource) => /never/.test(resource) }] },
					optimization: { minimize: false, sideEffects: true, providedExports: true, usedExports: true }
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([]);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "unsettled-function-use",
		description: "finishes when a function use returns different options on every call, across an import cycle",
		async run() {
			await withProjects({
				"src/index.js": 'module.exports = require("./m.js");',
				"src/m.js": 'exports.value = "m"; require("./n.js");',
				"src/n.js": 'exports.value = "n"; require("./m.js");',
				"loader.cjs": "module.exports = (source) => source;"
			}, async (writer, reader) => {
				let calls = 0;
				const configuration = {
					module: {
						rules: [{ test: /[mn]\.js$/, use: () => [{ loader: "./loader.cjs", options: { call: ++calls } }] }]
					}
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toHaveLength(2);
				expect(calls).toBeLessThan(20);
			});
		}
	},
	{
		name: "factory-hook-lifecycle",
		description: "calls dependency factory hooks on relocation and skips them on a same-root warm build",
		async run() {
			await withProjects({}, async (writer, reader) => {
				const requests = [];
				const configuration = { plugins: [beforeResolve((data) => {
					if (data.request === "./dep.js") requests.push(data.request);
				})] };
				await build(writer, configuration);
				requests.length = 0;
				expect((await build(writer, configuration)).built).toEqual([]);
				expect(requests).toEqual([]);
				copyTree(writer, reader);
				requests.length = 0;
				const moved = await build(reader, configuration);
				expect(requests.length).toBeGreaterThan(0);
				expect(moved.built).toEqual([]);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "unreachable",
		description: "does not pass modules unreachable at the reader through factory hooks",
		async run() {
			await withProjects({
				"src/index.js": 'require("./orphan.js"); module.exports = require("./dep.js");',
				"src/orphan.js": 'module.exports = require("./orphan-leaf.js");',
				"src/orphan-leaf.js": 'module.exports = "orphan";'
			}, async (writer, reader) => {
				const requests = [];
				const configuration = { plugins: [beforeResolve((data) => { requests.push(data.request); })] };
				await build(writer, configuration);
				copyTree(writer, reader);
				fs.writeFileSync(path.join(reader, "src/index.js"), 'module.exports = require("./dep.js");');
				requests.length = 0;
				const moved = await build(reader, configuration);
				expect(requests).not.toContain("./orphan-leaf.js");
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "unreachable-shared",
		description: "does not provide shared modules kept only by recovery anchors",
		async run() {
			await withProjects({
				"src/index.js": 'module.exports = require("shared-target");',
				"node_modules/shared-target/package.json": '{"name":"shared-target","version":"1.0.0","main":"index.js"}',
				"node_modules/shared-target/index.js": 'module.exports = require("./leaf.js");',
				"node_modules/shared-target/leaf.js": 'module.exports = "shared";'
			}, async (writer, reader) => {
				const configuration = { plugins: [new rspack.sharing.ProvideSharedPlugin({ provides: ["shared-target"] })] };
				await build(writer, configuration);
				copyTree(writer, reader);
				fs.writeFileSync(path.join(reader, "src/index.js"), 'module.exports = "reader";');
				const moved = await build(reader, configuration);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	}
];
