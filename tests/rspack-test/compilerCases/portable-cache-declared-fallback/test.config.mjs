import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

// Each resource exercises one place where a certified literal cannot be relocated. Escaped
// spellings hold no root bytes, so only the parser's own decision can bind those modules.
const DECLARE_LOADER = String.raw`
const path = require("node:path");
const TICK = String.fromCharCode(96);
const escapeAll = (value) => {
	let escaped = "";
	for (let i = 0; i < value.length; i++) {
		escaped += "\\u" + value.charCodeAt(i).toString(16).padStart(4, "0");
	}
	return escaped;
};
module.exports = function () {
	const file = this.resourcePath;
	// Above the root, at the same place for every checkout in this layout.
	const outside = path.join(path.dirname(file), "..", "..", "..", "outside.js");
	const dep = path.join(path.dirname(file), "dep.js");
	switch (path.basename(file)) {
		case "template.js":
			this.experiments.portableOutput({ paths: [file] });
			return "export const value = " + TICK + escapeAll(file) + TICK + ";";
		case "evaluated.js":
			this.experiments.portableOutput({ paths: [file] });
			return (
				"export let value = 'none';\n" +
				"if (\"" + escapeAll(file) + "\" === \"" + escapeAll(file) + "\") { value = \"" + escapeAll(file) + "\"; }"
			);
		case "imported.js":
			this.experiments.portableOutput({ paths: [file, dep] });
			return "import dep from " + JSON.stringify(dep) + "; export const value = dep;";
		case "uncertified.js":
			this.experiments.portableOutput({ paths: [file] });
			return "export const value = " + JSON.stringify(file) + ";";
		case "noparse.js":
			this.experiments.portableOutput({ paths: [file] });
			return "module.exports = \"" + escapeAll(file) + "\";";
		case "create.js":
			this.experiments.portableOutput({ paths: [file] });
			return 'import { createRequire } from "node:module"; export let value; function record(p) { value = p; } export const cache = createRequire(import.meta.url, record(' + JSON.stringify(file) + ')).cache;';
		case "outside.js":
			this.experiments.portableOutput({ paths: [outside] });
			return "export const value = " + JSON.stringify(outside) + ";";
		default:
			throw new Error("unexpected resource " + file);
	}
};
`;

// Does not certify, so the chain it ends is not portable.
const PLAIN_LOADER = "module.exports = function (source) { return source; };";

const CASES = [
	"template",
	"evaluated",
	"imported",
	"uncertified",
	"noparse",
	"create",
	"outside"
];

async function build(context, cache = { type: "persistent", portable: true }) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		entry: "./src/index.js",
		externals: { "node:module": "commonjs node:module" },
		output: { path: path.join(context, "dist") },
		module: {
			noParse: /noparse\.js$/,
			rules: [
				{ test: /create\.js$/, parser: { createRequire: true } },
				{
					test: /(template|evaluated|imported|noparse|outside|create)\.js$/,
					loader: "./loaders/declare.js"
				},
				{
					test: /uncertified\.js$/,
					use: [
						"./loaders/plain.js",
						"./loaders/declare.js"
					]
				}
			]
		},
		cache,
		infrastructureLogging: { level: "error" }
	});
	try {
		const stats = await runCompiler(compiler);
		if (stats.hasErrors()) throw new Error(stats.toString({ all: false, errors: true, errorDetails: true }));
		const output = fs.readFileSync(
			path.join(context, "dist", "main.js"),
			"utf-8"
		);
		const values = [];
		new Function("console", "require", output)({ log: (value) => values.push(value) }, createRequire(import.meta.url));
		const built = new Set(
			[...stats.compilation.builtModules]
				.map((module) => module.identifier())
				.flatMap((identifier) =>
					CASES.filter((name) =>
						identifier.endsWith(path.join("src", `${name}.js`))
					)
				)
		);
		return { output, values: JSON.parse(values[0]), built };
	} finally {
		await closeCompiler(compiler);
	}
}

function writeProject(root) {
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.mkdirSync(path.join(root, "src"));
	fs.mkdirSync(path.join(root, "loaders"));
	fs.writeFileSync(path.join(root, "package.json"), '{ "name": "fixture" }');
	fs.writeFileSync(path.join(root, "loaders", "declare.js"), DECLARE_LOADER);
	fs.writeFileSync(path.join(root, "loaders", "plain.js"), PLAIN_LOADER);
	for (const name of CASES) {
		fs.writeFileSync(path.join(root, "src", `${name}.js`), "");
	}
	fs.writeFileSync(path.join(root, "src", "dep.js"), "export default 7;");
	fs.writeFileSync(
		path.join(root, "src", "index.js"),
		[
			...CASES.map((name) => `import * as ${name} from "./${name}.js";`),
			"console.log(JSON.stringify({",
			...CASES.map((name) =>
				name === "noparse"
					? "noparse: noparse.default,"
					: `${name}: ${name}.value,`
			),
			"}));"
		].join("\n")
	);
}

function expectValues(result, root, outside) {
	const file = (name) => path.join(root, "src", `${name}.js`);
	expect(result.values).toEqual({
		template: file("template"),
		evaluated: file("evaluated"),
		imported: 7,
		uncertified: file("uncertified"),
		noparse: file("noparse"),
		create: file("create"),
		outside
	});
	expect(result.output).not.toContain("[portable-root]");
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should keep real text and bind modules whose certified literals cannot be relocated",
	async run() {
		const temp = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-declared-fallback-"))
		);
		try {
			// A declared path above the root is not relocated; it stays fixed.
			const outside = path.join(temp, "outside.js");
			const writer = path.join(temp, "a", "proj");
			writeProject(writer);
			const first = await build(writer);
			expectValues(first, writer, outside);
			const writerUncached = await build(writer, false);
			expect(writerUncached.output).toBe(first.output);

			const reader = path.join(temp, "reader-with-longer-root", "proj");
			fs.mkdirSync(path.dirname(reader), { recursive: true });
			fs.cpSync(writer, reader, { recursive: true });
			const moved = await build(reader);
			expectValues(moved, reader, outside);
			// Bound modules rebuild; only the fixed outside literal is reused.
			expect([...moved.built].sort()).toEqual(
				CASES.filter((name) => name !== "outside").sort()
			);
			removeTree(path.join(reader, "node_modules"));
			const cold = await build(reader);
			expect(moved.output).toBe(cold.output);
			const uncached = await build(reader, false);
			expect(uncached.output).toBe(cold.output);
		} finally {
			removeTree(temp);
		}
	}
};
