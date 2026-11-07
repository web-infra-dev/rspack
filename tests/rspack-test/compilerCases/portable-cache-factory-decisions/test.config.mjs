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

async function expectWarmEquality(reader, configuration, moved) {
	const warm = await build(reader, configuration);
	expect(warm.built).toEqual([]);
	expect(warm.assets).toEqual(moved.assets);
}

function execute({ assets }) {
	const module = { exports: {} };
	new Function("module", "exports", assets["main.js"])(module, module.exports);
	return module.exports;
}

const sharedImporters = {
	"src/index.js": 'module.exports = [require("./a.js"), require("./c1.js")];',
	"src/a.js": 'module.exports = require("./dep.js");',
	"src/c1.js": 'module.exports = require("./b.js");',
	"src/b.js": 'module.exports = require("./dep.js");',
	"src/dep.js": "module.exports = (function () { return this === undefined; })();"
};
const DEP_OF_A = "a.js > ./dep.js";
const DEP_OF_B = "b.js > ./dep.js";
// Holds b.js back, so the group of a.js produces dep.js in a cold build.
const A_PRODUCES = { "index.js > ./c1.js": 200 };
// With two groups in a.js, holds `import("./dep")` back, so `require("./dep.js")` produces dep.js.
const REQUIRE_OF_A_PRODUCES = { ...A_PRODUCES, "a.js > ./dep": 200 };

// Counts the factory calls of each dependency group, and holds chosen groups back.
function factoryGroups(delays) {
	const groups = {
		delays,
		calls: {},
		apply(compiler) {
			compiler.hooks.normalModuleFactory.tap("FactoryGroups", (factory) => {
				factory.hooks.beforeResolve.tapPromise("FactoryGroups", async ({ request, contextInfo }) => {
					const group = `${path.basename(contextInfo.issuer)} > ${request}`;
					groups.calls[group] = (groups.calls[group] ?? 0) + 1;
					if (groups.delays[group]) {
						await new Promise((resolve) => setTimeout(resolve, groups.delays[group]));
					}
				});
			});
		}
	};
	return groups;
}

function strictWhenImportedBy(issuer, plugins = []) {
	return {
		plugins,
		module: { rules: [{ test: /dep\.js$/, issuer, parser: { overrideStrict: "strict" } }] }
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
	...[
		{
			name: "changed-decision-of-producing-importer",
			description: "rebuilds a shared module with the changed decision of the importer that produced it",
			issuer: /reader.*a\.js$/,
			// The group of b.js reaches `AddTask` first among the groups the rebuild refactorizes.
			repairDelays: { [DEP_OF_A]: 50 },
			rebuilt: true,
			exports: [true, true]
		},
		{
			name: "changed-decision-of-other-importer",
			description: "keeps a shared module when only an importer that did not produce it changes its decision",
			issuer: /reader.*b\.js$/,
			rebuilt: false,
			exports: [false, false]
		},
		{
			name: "same-difference-with-producing-importer",
			description: "keeps a shared module whose importers differ at both roots, when the producer's rule matches",
			issuer: /a\.js$/,
			rebuilt: false,
			exports: [true, true]
		},
		{
			name: "same-difference-with-other-importer",
			description: "keeps a shared module whose importers differ at both roots, when the other importer's rule matches",
			issuer: /b\.js$/,
			rebuilt: false,
			exports: [false, false]
		},
		{
			name: "safe-shared-module",
			description: "keeps a shared module whose importers all agree",
			issuer: /never/,
			rebuilt: false,
			exports: [false, false]
		}
	].map(({ name, description, issuer, repairDelays = {}, rebuilt, exports }) => ({
		name,
		description,
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const groups = factoryGroups(A_PRODUCES);
				const configuration = strictWhenImportedBy(issuer, [groups]);
				await build(writer, configuration);
				copyTree(writer, reader);
				groups.calls = {};
				groups.delays = { ...A_PRODUCES, ...repairDelays };
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual(rebuilt ? [path.join(reader, "src", "dep.js")] : []);
				expect(execute(moved)).toEqual(exports);
				// Verification calls every factory once, and only a rebuild refactorizes the groups of dep.js.
				// The entry dependency is new to the recovered graph, so make factorizes it before verification.
				expect(groups.calls).toEqual({
					" > ./src/index.js": 2,
					"index.js > ./a.js": 1,
					"index.js > ./c1.js": 1,
					"c1.js > ./b.js": 1,
					[DEP_OF_A]: rebuilt ? 2 : 1,
					[DEP_OF_B]: rebuilt ? 2 : 1
				});
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	})),
	{
		name: "producing-importer-removed",
		description: "rebuilds a shared module for the remaining importer when the importer that produced it is gone",
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/, [factoryGroups(A_PRODUCES)]);
				await build(writer, configuration);
				copyTree(writer, reader);
				fs.writeFileSync(path.join(reader, "src/index.js"), 'module.exports = [require("./c1.js")];');
				const moved = await build(reader, configuration);
				expect(moved.built).toContain(path.join(reader, "src", "dep.js"));
				expect(execute(moved)).toEqual([true]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-importer-re-resolved",
		description: "rebuilds a shared module for the remaining importer when the producer's group resolves elsewhere",
		async run() {
			await withProjects({ ...sharedImporters, ...replacements }, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/, [
					factoryGroups(A_PRODUCES),
					beforeResolve((data) => {
						if (data.request === "./dep.js" && /reader.*a\.js$/.test(data.contextInfo.issuer)) data.request = "./dep2.js";
					})
				]);
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toContain(path.join(reader, "src", "dep.js"));
				expect(execute(moved)).toEqual(["replacement", true]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-importer-rebuilt-without-import",
		description: "keeps the other importer's disagreement until the rebuild of the producer leaves the module to it",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/a.js": 'module.exports = "a"; globalThis.never && require("./dep.js");'
			}, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/, [factoryGroups(A_PRODUCES)]);
				configuration.module.noParse = /reader.*a\.js$/;
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built.sort()).toEqual([path.join(reader, "src", "a.js"), path.join(reader, "src", "dep.js")]);
				expect(execute(moved)).toEqual(["a", true]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-importer-removed-by-no-parse",
		description: "keeps a shared module when the rebuild of its producer removes the import whose decision changed",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/a.js": 'module.exports = "a"; globalThis.never && require("./dep.js");'
			}, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*a\.js$/, [factoryGroups(A_PRODUCES)]);
				configuration.module.noParse = /reader.*a\.js$/;
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "a.js")]);
				expect(execute(moved)).toEqual(["a", false]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-importer-rebuilt-keeps-import",
		description: "keeps a shared module when its producer is rebuilt at the reader and only the other importer changes its decision",
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/, [factoryGroups(A_PRODUCES)]);
				await build(writer, configuration);
				copyTree(writer, reader);
				fs.appendFileSync(path.join(reader, "src/a.js"), "\n// edited");
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "a.js")]);
				expect(execute(moved)).toEqual([false, false]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-importer-edited-at-writer",
		description: "keeps a shared module when its producer was edited at the writer and only the other importer changes its decision",
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/, [factoryGroups(A_PRODUCES)]);
				await build(writer, configuration);
				fs.appendFileSync(path.join(writer, "src/a.js"), "\n// edited");
				expect((await build(writer, configuration)).built).toEqual([path.join(writer, "src", "a.js")]);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([]);
				expect(execute(moved)).toEqual([false, false]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "disagreeing-importers-in-a-cycle",
		description: "still rebuilds the modules of an import cycle whose deciding groups come from each other",
		async run() {
			await withProjects({
				"src/index.js": 'module.exports = [require("./t1.js"), require("./x.js")];',
				"src/x.js": 'module.exports = require("./t1.js");',
				"src/t1.js": 'exports.strict = () => require("./t2.js").strict;',
				"src/t2.js": 'exports.strict = (function () { return this === undefined; })(); exports.t1 = require("./t1.js");'
			}, async (writer, reader) => {
				const configuration = {
					plugins: [factoryGroups({ "index.js > ./x.js": 200 })],
					module: {
						rules: [
							{ test: /t1\.js$/, issuer: /reader.*t2\.js$/, parser: { overrideStrict: "strict" } },
							{ test: /t2\.js$/, issuer: /reader.*t1\.js$/, parser: { overrideStrict: "strict" } }
						]
					}
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				// The producer of t1.js is gone, so t2.js decides t1.js while t1.js decides t2.js.
				fs.writeFileSync(path.join(reader, "src/index.js"), 'module.exports = [require("./x.js")];');
				const moved = await build(reader, configuration);
				expect(execute(moved)[0].strict()).toBe(true);
				await expectWarmEquality(reader, configuration, moved);
				// A cold build takes t1.js from x.js. The fallback takes it from t2.js, a documented limit
				// that this module's behavior does not show, so the assets can differ.
				fs.rmSync(path.join(reader, "node_modules", ".cache"), { recursive: true, force: true });
				expect(execute(await build(reader, configuration))[0].strict()).toBe(true);
			});
		}
	},
	{
		name: "producing-entry-removed",
		description: "rebuilds a shared module for the remaining importer when it was produced as an entry that is gone",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/index.js": 'module.exports = [require("./c1.js")];'
			}, async (writer, reader) => {
				const configuration = strictWhenImportedBy(/reader.*b\.js$/);
				await build(writer, { ...configuration, entry: ["./src/dep.js", "./src/index.js"] });
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "dep.js")]);
				expect(execute(moved)).toEqual([true]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "producing-entry-kept",
		description: "keeps a shared module produced as an entry when only an importer changes its decision",
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const configuration = {
					...strictWhenImportedBy(/reader.*b\.js$/),
					entry: ["./src/dep.js", "./src/index.js"]
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([]);
				expect(execute(moved)).toEqual([false, false]);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	{
		name: "two-groups-of-producing-importer",
		description: "keeps a shared module when only the other of the producer's two groups changes its decision",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/a.js": 'module.exports = require("./dep.js"); import("./dep");'
			}, async (writer, reader) => {
				const configuration = {
					plugins: [factoryGroups(REQUIRE_OF_A_PRODUCES)],
					module: { rules: [{ test: /dep\.js$/, dependency: "esm", issuer: /reader/, parser: { overrideStrict: "strict" } }] }
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([]);
			});
		}
	},
	{
		name: "two-groups-of-producing-importer-changed",
		description: "rebuilds a shared module when the producing group of its importer changes and the sibling group agrees",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/a.js": 'module.exports = { sync: require("./dep.js"), later: () => import("./dep") };'
			}, async (writer, reader) => {
				const configuration = {
					plugins: [factoryGroups(REQUIRE_OF_A_PRODUCES)],
					module: { rules: [{ test: /dep\.js$/, dependency: "commonjs", issuer: /reader.*a\.js$/, parser: { overrideStrict: "strict" } }] }
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([path.join(reader, "src", "dep.js")]);
				const [{ sync, later }] = execute(moved);
				expect([sync, (await later()).default]).toEqual([true, true]);
				await expectWarmEquality(reader, configuration, moved);
				await expectColdEquality(reader, configuration, moved);
			});
		}
	},
	...[
		{
			name: "writer-rebuild-keeps-for-issuer",
			description: "keeps a shared module when only its issuer disagrees, after a writer rebuild another importer produced",
			issuer: /reader.*a\.js$/,
			rebuilt: false,
			exports: [false, false]
		},
		{
			name: "writer-rebuild-decides-by-producer",
			description: "rebuilds a shared module with the decision of the importer that produced the writer's rebuild",
			issuer: /reader.*b\.js$/,
			// The group of a.js reaches `AddTask` first among the groups the rebuild refactorizes.
			repairDelays: { [DEP_OF_B]: 50 },
			rebuilt: true,
			exports: [true, true]
		}
	].map(({ name, description, issuer, repairDelays = {}, rebuilt, exports }) => ({
		name,
		description,
		async run() {
			await withProjects(sharedImporters, async (writer, reader) => {
				const groups = factoryGroups(A_PRODUCES);
				let issuerOfDep;
				const configuration = strictWhenImportedBy(issuer, [groups, {
					apply(compiler) {
						compiler.hooks.done.tap("IssuerOfDep", ({ compilation }) => {
							const dep = [...compilation.modules].find((module) => module.identifier().endsWith("dep.js"));
							issuerOfDep = compilation.moduleGraph.getIssuer(dep).identifier();
						});
					}
				}]);
				await build(writer, configuration);
				expect(issuerOfDep).toBe(path.join(writer, "src", "a.js"));
				// The group of b.js produces this rebuild, and fixing issuers keeps a.js as the issuer.
				fs.appendFileSync(path.join(writer, "src/dep.js"), "\n// changed");
				groups.delays = { [DEP_OF_A]: 200 };
				expect((await build(writer, configuration)).built).toEqual([path.join(writer, "src", "dep.js")]);
				expect(issuerOfDep).toBe(path.join(writer, "src", "a.js"));
				copyTree(writer, reader);
				groups.delays = repairDelays;
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual(rebuilt ? [path.join(reader, "src", "dep.js")] : []);
				expect(execute(moved)).toEqual(exports);
				await expectWarmEquality(reader, configuration, moved);
				// A cold build in which the group of b.js produces dep.js, as at the writer.
				groups.delays = { "index.js > ./a.js": 200 };
				await expectColdEquality(reader, configuration, moved);
			});
		}
	})),
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
				// Module identifiers carry the loader request.
				expect([...moved.built].sort()).toEqual([
					`${path.join(reader, "digest.cjs")}!${path.join(reader, "src", "digest.js")}`,
					`${path.join(reader, "map.cjs")}!${path.join(reader, "src", "dep.js")}`
				].sort());
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
