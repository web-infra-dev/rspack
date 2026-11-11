import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

// The loader certifies the resource path and emits it in several spellings, with a source map
// whose generated columns follow the real text.
const LOADER = String.raw`
const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const vlq = (value) => {
	let rest = value < 0 ? (-value << 1) | 1 : value << 1;
	let encoded = "";
	do {
		let digit = rest & 31;
		rest >>>= 5;
		if (rest) digit |= 32;
		encoded += BASE64[digit];
	} while (rest);
	return encoded;
};
const escapeAll = (value) => {
	let escaped = "";
	for (let i = 0; i < value.length; i++) {
		escaped += "\\u" + value.charCodeAt(i).toString(16).padStart(4, "0");
	}
	return escaped;
};
module.exports = function (source) {
	this.experiments.portableOutput({ paths: [this.resourcePath] });
	const file = this.resourcePath;
	const lines = [
		"export const file = " + JSON.stringify(file) + ";",
		"export const single = '" + file.replace(/\\/g, "\\\\").replace(/'/g, "\\'") + "';",
		"export const escaped = \"" + escapeAll(file) + "\";",
		"export const info = jsx(\"div\", { fileName: " + JSON.stringify(file) + ", lineNumber: 1 });",
		"export const keyed = { " + JSON.stringify(file) + ": 1 };",
		"function jsx(type, props) { return [type, props.fileName, props.lineNumber]; }",
		source
	];
	// Every segment maps to the start of the resource; columns are UTF-16 offsets in the output.
	const mappings = lines
		.map((line) => {
			const segments = [];
			let column = 0;
			const add = (generated) => {
				segments.push(vlq(generated - column) + "AAA");
				column = generated;
			};
			add(0);
			const semicolon = line.lastIndexOf(";");
			if (semicolon > 0) add(semicolon);
			return segments.join(",");
		})
		.join(";");
	this.callback(null, lines.join("\n"), {
		version: 3,
		sources: [this.resourcePath],
		sourcesContent: [source],
		names: [],
		mappings
	});
};
`;

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

async function build(context, cache = { type: "persistent", portable: true }) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: "source-map",
		entry: "./src/index.js",
		output: {
			path: path.join(context, "dist"),
			filename: "[name].[contenthash:8].js"
		},
		module: {
			rules: [{ test: /a\.js$/, loader: "./loaders/declare.js" }]
		},
		optimization: { realContentHash: false },
		cache,
		infrastructureLogging: { level: "error" }
	});
	try {
		const stats = await runCompiler(compiler);
		const json = stats.toJson({ all: false, errors: true, assets: true });
		expect(json.errors).toEqual([]);
		const file = stats.compilation.getAssets()
			.map((asset) => asset.name)
			.find((name) => name.endsWith(".js"));
		const output = fs.readFileSync(path.join(context, "dist", file), "utf-8");
		const map = fs.readFileSync(
			path.join(context, "dist", `${file}.map`),
			"utf-8"
		);
		const values = [];
		new Function("console", output)({ log: (value) => values.push(value) });
		const module = [...stats.compilation.modules].find((module) =>
			module.identifier().endsWith(path.join("src", "a.js"))
		);
		return {
			file,
			map,
			output,
			values: JSON.parse(values[0]),
			source: module.originalSource().source().toString(),
			built: [...stats.compilation.builtModules].some((module) =>
				module.identifier().endsWith(path.join("src", "a.js"))
			)
		};
	} finally {
		await closeCompiler(compiler);
	}
}

function writeProject(root) {
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.mkdirSync(path.join(root, "src"));
	fs.mkdirSync(path.join(root, "loaders"));
	fs.writeFileSync(path.join(root, "package.json"), '{ "name": "fixture" }');
	fs.writeFileSync(path.join(root, "loaders", "declare.js"), LOADER);
	fs.writeFileSync(path.join(root, "src", "a.js"), "export const own = 1;\n");
	fs.writeFileSync(
		path.join(root, "src", "index.js"),
		'import * as a from "./a.js"; console.log(JSON.stringify(a));'
	);
}

function expectPaths(result, root) {
	const file = path.join(root, "src", "a.js");
	expect(result.values.file).toBe(file);
	expect(result.values.single).toBe(file);
	expect(result.values.escaped).toBe(file);
	expect(result.values.info).toEqual(["div", file, 1]);
	expect(result.values.keyed).toEqual({ [file]: 1 });
	// Public source readers see real literals, never the canonical root.
	expect(result.source).toContain(JSON.stringify(file).replace(/[^\x00-\x7f]/g, (char) =>
		`\\u${char.charCodeAt(0).toString(16).padStart(4, "0")}`));
	expect(result.source).not.toContain("[portable-root]");
	expect(result.output).not.toContain("[portable-root]");
	expect(result.map).not.toContain("[portable-root]");
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should relocate certified loader path literals, with maps and hashes equal to a cold build",
	async run() {
		const temp = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-declared-"))
		);
		try {
			const writer = path.join(temp, "a", "proj");
			writeProject(writer);
			const first = await build(writer);
			expectPaths(first, writer);
			expect(first.built).toBe(true);
			// A certified build normalizes literals with or without a persistent cache.
			const writerUncached = await build(writer, false);
			expect(writerUncached.output).toBe(first.output);
			expect(writerUncached.map).toBe(first.map);
			expect(writerUncached.file).toBe(first.file);

			// A longer, non-ASCII root shifts every column after the literal.
			const reader = path.join(temp, "lönger-reader-root", "proj");
			copyTree(writer, reader);
			const moved = await build(reader);
			expect(moved.built).toBe(false);
			expectPaths(moved, reader);
			removeTree(path.join(reader, "node_modules"));
			const cold = await build(reader);
			expect(cold.built).toBe(true);
			expect(moved.output).toBe(cold.output);
			expect(moved.map).toBe(cold.map);
			expect(moved.file).toBe(cold.file);
			const uncached = await build(reader, false);
			expect(uncached.output).toBe(cold.output);
			expect(uncached.map).toBe(cold.map);
			expect(uncached.file).toBe(cold.file);
			// The output follows the root it was built in.
			expect(moved.file).not.toBe(first.file);
		} finally {
			removeTree(temp);
		}
	}
};
