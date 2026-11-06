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

export default [
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
		name: "two-groups-of-producing-importer",
		description: "keeps a shared module when one of the producer's two groups still agrees with its build",
		async run() {
			await withProjects({
				...sharedImporters,
				"src/a.js": 'module.exports = require("./dep.js"); import("./dep.js");'
			}, async (writer, reader) => {
				const configuration = {
					plugins: [factoryGroups(A_PRODUCES)],
					module: { rules: [{ test: /dep\.js$/, dependency: "esm", issuer: /reader/, parser: { overrideStrict: "strict" } }] }
				};
				await build(writer, configuration);
				copyTree(writer, reader);
				const moved = await build(reader, configuration);
				expect(moved.built).toEqual([]);
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
