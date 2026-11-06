import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
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

// A root short enough that its base64 spelling is shorter than 12 characters at some alignment.
// Only Linux has a writable short real path: macOS links /tmp to /private/tmp, and Windows has none.
async function withShortRoots(files, run) {
	const make = () => {
		for (;;) {
			const root = path.join(path.sep, "tmp", crypto.randomBytes(2).toString("hex").slice(0, 3));
			try {
				fs.mkdirSync(root);
				return root;
			} catch (error) {
				if (error.code !== "EEXIST") throw error;
			}
		}
	};
	const writer = make();
	const reader = make();
	try {
		fs.mkdirSync(path.join(writer, ".git"));
		for (const [name, source] of Object.entries({
			"package.json": '{"name":"factory-decisions"}',
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
		for (const root of [writer, reader]) fs.rmSync(root, { recursive: true, force: true });
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

const parsed = {
	files: {
		"src/dep.js": 'module.exports = typeof require("./leaf.js");',
		"src/leaf.js": 'module.exports = "leaf";'
	}
};
const emitOption = {
	"options.cjs": "module.exports = function () { const value = this.getOptions().value; return 'module.exports = ' + JSON.stringify(String(value instanceof Set ? [...value] : value)); };"
};
scenarios.push(
	{
		name: "no-parse-regex",
		...parsed,
		noParse: () => /writer-only.*dep\.js$/
	},
	{
		name: "no-parse-function",
		...parsed,
		noParse: (writer) => (resource) => resource.startsWith(writer) && /dep\.js$/.test(resource)
	},
	{
		name: "function-use-regexp-option",
		files: emitOption,
		rules: (writer) => [{ test: /dep\.js$/, use: (info) => [{
			loader: "./options.cjs",
			options: { value: new RegExp(info.resource.startsWith(writer + path.sep) ? "writer" : "reader") }
		}] }]
	},
	{
		name: "function-use-set-option",
		files: emitOption,
		rules: (writer) => [{ test: /dep\.js$/, use: (info) => [{
			loader: "./options.cjs",
			options: { value: new Set([info.resource.startsWith(writer + path.sep) ? "writer" : "reader"]) }
		}] }]
	},
	{
		name: "import-module-rule",
		files: {
			"import.cjs": "module.exports = async function () { const value = await this.importModule('./leaf.js'); return 'module.exports = ' + JSON.stringify(value); };",
			"src/leaf.js": 'module.exports = "original";'
		},
		rules: (writer) => [
			{ test: /dep\.js$/, use: ["./import.cjs"] },
			{ test: /leaf\.js$/, include: writer, use: [transform] }
		]
	}
);

// Reports what the loader saw through `this.getOptions()`: the option's type and its value.
const observeLoader = {
	"observe.cjs": `module.exports = function () {
		const { value, log } = this.getOptions();
		if (log) require("node:fs").appendFileSync(log, "run\\n");
		const type = Object.prototype.toString.call(value);
		const shown = type === "[object Set]" ? [...value].join()
			: type === "[object Function]" ? value()
			: type === "[object Date]" ? value.toISOString()
			: JSON.stringify(value) ?? "undefined";
		return "module.exports = " + JSON.stringify(type + " " + shown);
	};`
};

function execute({ assets }) {
	const module = { exports: {} };
	new Function("module", "exports", assets["main.js"])(module, module.exports);
	return module.exports;
}

function useObserved(writer, writerValue, readerValue, cache) {
	return {
		module: {
			rules: [{
				test: /dep\.js$/,
				use: (info) => [{
					loader: "./observe.cjs",
					cache,
					options: { value: info.resource.startsWith(writer + path.sep) ? writerValue : readerValue }
				}]
			}]
		}
	};
}

function runChild(options) {
	const result = spawnSync(
		process.execPath,
		[fileURLToPath(new URL("./child.mjs", import.meta.url)), JSON.stringify(options)],
		{ encoding: "utf8", timeout: 60_000 }
	);
	expect(result.error).toBeUndefined();
	expect({ status: result.status, stderr: result.stderr }).toEqual({ status: 0, stderr: "" });
	return JSON.parse(result.stdout);
}

async function buildAcrossProcesses(kind, loaderCache, check) {
	await withProjects(observeLoader, async (writer, reader) => {
		const options = { writer, kind, loaderCache };
		const written = runChild({ ...options, context: writer, cached: true });
		copyTree(writer, reader);
		const moved = runChild({ ...options, context: reader, cached: true });
		fs.rmSync(path.join(reader, "node_modules", ".cache"), { recursive: true, force: true });
		const cold = runChild({ ...options, context: reader, cached: true });
		const uncached = runChild({ ...options, context: reader, cached: false });
		expect(moved.executed).toEqual(cold.executed);
		expect(moved.executed).toEqual(uncached.executed);
		expect(moved.assets).toEqual(cold.assets);
		expect(moved.assets).toEqual(uncached.assets);
		check({ written, moved });
	});
}

const collisions = [
	{ name: "undefined", value: undefined, text: "[undefined]" },
	{ name: "nan", value: NaN, text: "NaN" },
	{ name: "date", value: new Date("2020-01-01T00:00:00.000Z"), text: "2020-01-01T00:00:00.000Z" }
];
const loaderCaches = [{ suffix: "uncached-loader", cache: false }, { suffix: "cached-loader", cache: true }];

// The loader result cache is bound to the absolute context, so two builds share it only at one root.
function useLoaderResultCache(kind, options) {
	const item = { loader: "./observe.cjs", cache: true, options };
	return {
		experiments: { newCache: { module: false, codeGeneration: false, devtool: false, loader: true, minimize: false } },
		module: { rules: [{ test: /dep\.js$/, use: kind === "function" ? () => [item] : [item] }] }
	};
}

export default [
	...[
		{ kind: "set", type: "[object Set]" },
		{ kind: "function", type: "[object Function]" }
	].flatMap(({ kind, type }) => loaderCaches.map(({ suffix, cache }) => ({
		name: `process-${kind}-option-${suffix}`,
		description: `rebuilds a function use returning a ${kind} option when writer and reader are separate processes`,
		async run() {
			await buildAcrossProcesses(kind, cache, ({ written, moved }) => {
				expect(written.executed).toBe(`${type} writer`);
				expect(moved.executed).toBe(`${type} reader`);
			});
		}
	}))),
	...loaderCaches.map(({ suffix, cache }) => ({
		name: `process-plain-option-${suffix}`,
		description: "reuses a function use returning identical plain data when writer and reader are separate processes",
		async run() {
			await buildAcrossProcesses("plain", cache, ({ moved }) => {
				expect(moved.built).toEqual([]);
			});
		}
	})),
	...collisions.flatMap(({ name, value, text }) => [
		{ direction: "value-to-text", writerValue: value, readerValue: text },
		{ direction: "text-to-value", writerValue: text, readerValue: value }
	].flatMap(({ direction, writerValue, readerValue }) => loaderCaches.map(({ suffix, cache }) => ({
		name: `collision-${name}-${direction}-${suffix}`,
		description: `tells a ${name} option from its text across a move, ${direction}`,
		async run() {
			await withProjects(observeLoader, async (writer, reader) => {
				const configuration = useObserved(writer, writerValue, readerValue, cache);
				const written = await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(execute(moved)).not.toEqual(execute(written));
				fs.rmSync(path.join(reader, "node_modules", ".cache"), { recursive: true, force: true });
				const cold = await build(reader, configuration);
				const uncached = await build(reader, configuration, false);
				expect(execute(moved)).toEqual(execute(cold));
				expect(execute(moved)).toEqual(execute(uncached));
				expect(moved.assets).toEqual(cold.assets);
				expect(moved.assets).toEqual(uncached.assets);
			});
		}
	})))),
	...loaderCaches.flatMap(({ suffix, cache }) => [
		{
			name: `plain-options-unchanged-${suffix}`,
			description: "reuses modules whose function use returns identical plain data",
			async run() {
				await withProjects(observeLoader, async (writer, reader) => {
					const options = () => ({ list: [1, "x", null, undefined, true], nested: { ratio: 1.5 } });
					const configuration = useObserved(writer, options(), options(), cache);
					await build(writer, configuration);
					copyTree(writer, reader);
					const moved = await build(reader, configuration);
					expect(moved.built).toEqual([]);
					await expectColdEquality(reader, configuration, moved);
				});
			}
		},
		{
			name: `plain-options-changed-${suffix}`,
			description: "rebuilds modules whose function use returns changed plain data",
			async run() {
				await withProjects(observeLoader, async (writer, reader) => {
					const configuration = useObserved(writer, { size: 1 }, { size: 2 }, cache);
					const written = await build(writer, configuration);
					copyTree(writer, reader);
					const moved = await build(reader, configuration);
					expect(moved.built).toHaveLength(1);
					expect(execute(moved)).not.toEqual(execute(written));
					await expectColdEquality(reader, configuration, moved);
				});
			}
		}
	]),
	...["function", "array"].flatMap((kind) => [
		{
			name: `loader-result-cache-skips-set-option-${kind}-use`,
			description: `does not serve a loader result by a set option from a ${kind} use on a portable build`,
			async run() {
				await withProjects(observeLoader, async (writer) => {
					const first = await build(writer, useLoaderResultCache(kind, { value: new Set(["first"]) }));
					expect(execute(first)).toBe("[object Set] first");
					const second = await build(writer, useLoaderResultCache(kind, { value: new Set(["second"]) }));
					expect(execute(second)).toBe("[object Set] second");
				});
			}
		},
		{
			name: `loader-result-cache-keeps-plain-option-${kind}-use`,
			description: `serves a loader result by plain options from a ${kind} use on a portable build`,
			async run() {
				await withProjects(observeLoader, async (writer) => {
					const log = path.join(writer, "runs.log");
					const configuration = useLoaderResultCache(kind, { value: "plain", log });
					await build(writer, configuration);
					const second = await build(writer, configuration);
					expect(execute(second)).toBe('[object String] "plain"');
					expect(fs.readFileSync(log, "utf8")).toBe("run\n");
				});
			}
		}
	]),
	...scenarios.map(({ name, files = {}, rules, plugins, noParse }) => ({
		name,
		description: `decides ${name} at the reader of a relocated cache`,
		async run() {
			await withProjects(files, async (writer, reader) => {
				const configuration = {
					module: { rules: rules?.(writer) ?? [], ...(noParse ? { noParse: noParse(writer) } : {}) },
					plugins: plugins?.(writer) ?? []
				};
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
		name: "function-use-absolute-loader",
		description: "keeps builds when a function use returns an absolute loader path that moves with the checkout",
		async run() {
			await withProjects({
				"src/index.js": 'module.exports = require("./dep.js") + require("./other.js");',
				"src/other.js": 'module.exports = "other";'
			}, async (writer, reader) => {
				const configuration = (root) => ({
					module: { rules: [{ test: /(dep|other)\.js$/, use: () => [path.join(root, "transform.cjs")] }] }
				});
				await build(writer, configuration(writer));
				copyTree(writer, reader);
				const moved = await build(reader, configuration(reader));
				expect(moved.built).toEqual([]);
				await expectColdEquality(reader, configuration(reader), moved);
			});
		}
	},
	{
		name: "changed-decision-of-one-importer",
		description: "rebuilds a shared module when one importer's rule decision changes, even if another importer's does not",
		async run() {
			await withProjects({
				"src/index.js": 'module.exports = [require("./a.js"), require("./c1.js")];',
				"src/a.js": 'module.exports = require("./dep.js");',
				"src/c1.js": 'module.exports = require("./b.js");',
				"src/b.js": 'module.exports = require("./dep.js");',
				"src/dep.js": "module.exports = (function () { return this === undefined; })();"
			}, async (writer, reader) => {
				const configuration = {
					module: { rules: [{ test: /dep\.js$/, issuer: /reader.*a\.js$/, parser: { overrideStrict: "strict" } }] }
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "dep.js")]);
			});
		}
	},
	{
		name: "short-root-base64",
		description: "rebuilds loader output that holds a short root in base64, as an inline source map or a bare digest",
		async run() {
			if (process.platform !== "linux") return;
			await withShortRoots({
				"src/index.js": 'module.exports = [require("./dep.js"), require("./digest.js")];',
				"src/digest.js": 'module.exports = "original";',
				"map.cjs": 'module.exports = function () { const map = JSON.stringify({ version: 3, sources: [this.resourcePath], names: [], mappings: "AAAA" }); return "module.exports = " + JSON.stringify("data:application/json;base64," + Buffer.from(map).toString("base64")); };',
				"digest.cjs": 'module.exports = function () { return "module.exports = " + JSON.stringify(Buffer.from(this.resourcePath).toString("base64")); };'
			}, async (writer, reader) => {
				expect(Buffer.byteLength(writer)).toBeLessThan(9);
				const configuration = {
					module: { rules: [{ test: /dep\.js$/, use: ["./map.cjs"] }, { test: /digest\.js$/, use: ["./digest.cjs"] }] }
				};
				const written = await build(writer, configuration);
				expect(JSON.stringify(written.assets)).toContain(Buffer.from(path.join(writer, "src", "digest.js")).toString("base64"));
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "dep.js"), path.join(reader, "src", "digest.js")]);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "inline-source-map",
		description: "rebuilds loader output that holds the root in a base64 inline source map",
		async run() {
			await withProjects({
				"map.cjs": 'module.exports = function () { const map = JSON.stringify({ version: 3, file: this.resourcePath }); return "module.exports = " + JSON.stringify("data:application/json;base64," + Buffer.from(map).toString("base64")); };'
			}, async (writer, reader) => {
				const configuration = { module: { rules: [{ test: /dep\.js$/, use: ["./map.cjs"] }] } };
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
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
