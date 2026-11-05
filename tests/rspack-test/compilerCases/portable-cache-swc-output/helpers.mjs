import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export const swcOptions = {
	rspackExperiments: { portableOutput: true },
	inputSourceMap: false,
	detectSyntax: "auto",
	jsc: {
		target: "es2020",
		transform: { react: { runtime: "classic", development: true } }
	}
};

// A local classic JSX runtime exposes the dev filename as an observable value.
const RUNTIME = `
export default {
  createElement(type, props, ...children) {
    return { type, fileName: props.__source.fileName, line: props.__source.lineNumber,
      column: props.__source.columnNumber, title: props.title, children };
  }
};
`;

export function jsxSource(before = "", title = '"portable"', children = "hello") {
	return `import React from "./jsx-runtime.js";
${before}
export const value = <section title=${title}>${children}</section>;
export const after = "after the generated filename";
`;
}

export function writeProject(root, modules) {
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.mkdirSync(path.join(root, "src"));
	fs.writeFileSync(
		path.join(root, "package.json"),
		'{"name":"swc-portable-fixture"}'
	);
	fs.writeFileSync(path.join(root, "src", "jsx-runtime.js"), RUNTIME);
	fs.writeFileSync(path.join(root, "src", "automatic-runtime.js"), `
export const Fragment = "fragment";
export function jsxDEV(type, props, key, isStatic, source) {
  return { type, title: props.title, fileName: source?.fileName,
    line: source?.lineNumber, column: source?.columnNumber,
    children: props.children == null ? [] : [].concat(props.children) };
}
`);
	for (const [name, source] of Object.entries(modules)) {
		fs.writeFileSync(path.join(root, "src", name), source);
	}
	fs.writeFileSync(
		path.join(root, "src", "index.js"),
		[
			...Object.keys(modules).map(
				(name, index) => `import * as m${index} from "./${name}";`
			),
			`console.log(JSON.stringify([${Object.keys(modules)
				.map((_, index) => `m${index}`)
				.join(",")} ]));`
		].join("\n")
	);
}

export function swcRule(test, options = swcOptions, refresh = false) {
	return {
		test,
		use: [
			...(refresh ? ["builtin:react-refresh-loader"] : []),
			{ loader: "builtin:swc-loader", options }
		]
	};
}

export async function build(
	context,
	rules,
	cache = { type: "persistent", portable: true },
	{ configureCompiler, ...configuration } = {}
) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: "source-map",
		incremental: false,
		entry: "./src/index.js",
		output: {
			path: path.join(context, "dist"),
			filename: "[name].[contenthash:8].js"
		},
		module: { rules },
		resolve: {
			alias: { "react/jsx-dev-runtime": path.join(context, "src", "automatic-runtime.js") }
		},
		optimization: { realContentHash: false },
		cache,
		infrastructureLogging: { level: "error" },
		...configuration
	});
	try {
		configureCompiler?.(compiler);
		const stats = await runCompiler(compiler);
		const diagnostics = stats.toJson({ all: false, errors: true, warnings: true });
		expect(diagnostics.errors).toEqual([]);
		const file = stats.compilation.getAssets()
			.map((asset) => asset.name)
			.find((name) => name.endsWith(".js"));
		const output = fs.readFileSync(path.join(context, "dist", file), "utf-8");
		const map = fs.readFileSync(
			path.join(context, "dist", `${file}.map`),
			"utf-8"
		);
		const logs = [];
		let refreshCalls = 0;
		// The refresh plugin normally supplies this runtime. Provide it at execution time:
		// SWC would remove an unused import before the refresh loader appends its code.
		new Function("console", "$ReactRefreshRuntime$", output)(
			{ log: (value) => logs.push(value) },
			{
				createSignatureFunctionForTransform() { return (type) => type; },
				register() {},
				refresh() { refreshCalls++; }
			}
		);
		await Promise.resolve();
		if (rules.some((rule) => rule.use.includes("builtin:react-refresh-loader"))) {
			expect(refreshCalls).toBeGreaterThan(0);
		}
		expect(logs).toHaveLength(1);
		const modules = [...stats.compilation.modules];
		const sources = Object.fromEntries(
			modules
				.filter((module) => /\.(jsx|tsx)$/.test(module.identifier()))
				.map((module) => [
					path.basename(module.identifier()),
					module.originalSource().source().toString()
				])
		);
		const built = [...stats.compilation.builtModules]
			.filter((module) => /\.(jsx|tsx)$/.test(module.identifier()))
			.map((module) => path.basename(module.identifier()))
			.sort();
		return {
			file, output, map, values: JSON.parse(logs[0]), sources, built, diagnostics,
			fileDependencies: [...stats.compilation.fileDependencies],
			missingDependencies: [...stats.compilation.missingDependencies]
		};
	} finally {
		await closeCompiler(compiler);
	}
}

export async function withProjects(run) {
	const temp = fs.realpathSync.native(
		fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-swc-"))
	);
	try {
		await run(
			path.join(temp, "a", "proj"),
			path.join(temp, "much-lönger-读者-😀", "proj")
		);
	} finally {
		fs.rmSync(temp, { recursive: true, force: true });
	}
}

// fs.cpSync loses non-ASCII destinations on Windows in Node 22.
function copyTree(from, to) {
	fs.mkdirSync(to, { recursive: true });
	for (const entry of fs.readdirSync(from, { withFileTypes: true })) {
		const source = path.join(from, entry.name);
		const target = path.join(to, entry.name);
		if (entry.isDirectory()) copyTree(source, target);
		else if (entry.isSymbolicLink()) fs.symlinkSync(fs.readlinkSync(source), target);
		else fs.copyFileSync(source, target);
	}
}

export function copyProject(writer, reader) {
	copyTree(writer, reader);
}

export function expectEqualOutput(actual, expected) {
	for (const key of ["file", "output", "map", "sources", "values", "diagnostics"]) {
		expect(actual[key]).toEqual(expected[key]);
	}
}

export async function expectColdEquality(moved, reader, rules, configuration) {
	fs.rmSync(path.join(reader, "node_modules"), { recursive: true, force: true });
	const cold = await build(reader, rules, undefined, configuration);
	const uncached = await build(reader, rules, false, configuration);
	for (const result of [cold, uncached]) {
		expect(result.built).toEqual(Object.keys(moved.sources).sort());
		expectEqualOutput(result, moved);
	}
}
