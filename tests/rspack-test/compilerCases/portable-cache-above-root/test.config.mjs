import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

async function build(context, cache = { type: "persistent", portable: true }) {
	const logs = [];
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		entry: "./src/index.js",
		output: { path: path.join(context, "dist") },
		cache,
		infrastructureLogging: {
			level: "info",
			console: { ...console, info: (...args) => logs.push(args.join(" ")), log: () => {} }
		}
	});
	try {
		const stats = await runCompiler(compiler);
		expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
		return {
			built: [...stats.compilation.builtModules].map(module => path.relative(context, module.identifier())),
			dropped: logs.filter(line => /dropped \d+ entries bound to another root/.test(line)),
			output: fs.readFileSync(path.join(context, "dist", "main.js"), "utf-8")
		};
	} finally {
		await closeCompiler(compiler);
	}
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should reuse modules whose resolver lookups missed above the portable root after a move",
	async run() {
		// The temp dir must not sit behind a symlink: a root whose realpath differs has a layout of
		// its own, and this case is about the layout above the root, not the root's spelling.
		const temp = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-above-root-"))
		);
		try {
			const writer = path.join(temp, "a", "proj");
			fs.mkdirSync(path.join(writer, ".git"), { recursive: true });
			fs.mkdirSync(path.join(writer, "src"));
			fs.mkdirSync(path.join(writer, "node_modules", "pkg"), { recursive: true });
			fs.writeFileSync(path.join(writer, "node_modules", "pkg", "package.json"), '{ "name": "pkg", "main": "index.js" }');
			fs.writeFileSync(path.join(writer, "node_modules", "pkg", "index.js"), "export default 7;");
			// A bare request walks the resolver up past the root and records the ancestors' missing
			// `node_modules`. Without a root package.json the description-file walk leaves the root
			// too, so `../package.json` is recorded as missing as well.
			fs.writeFileSync(path.join(writer, "src", "dep.js"), 'import v from "pkg";\nexport default v;');
			fs.writeFileSync(path.join(writer, "src", "index.js"), 'import v from "./dep.js";\nconsole.log(v);');
			await build(writer);
			const warm = await build(writer);
			expect(warm.built).toEqual([]);

			// A sibling directory keeps the layout above the root, like another git worktree.
			const reader = path.join(temp, "b", "proj");
			fs.mkdirSync(path.dirname(reader), { recursive: true });
			fs.cpSync(writer, reader, { recursive: true });
			const moved = await build(reader);
			expect(moved.dropped).toEqual([]);
			expect(moved.built).toEqual([]);
			const cold = await build(reader, false);
			expect(moved.output).toBe(cold.output);

			// Ancestor lookups must notice a newly available package after relocation.
			const changed = path.join(temp, 'changed', 'proj');
			fs.mkdirSync(path.dirname(changed), { recursive: true });
			fs.cpSync(writer, changed, { recursive: true });
			fs.writeFileSync(
				path.join(temp, 'changed', 'package.json'),
				'{ "name": "parent" }',
			);
			const changedBuild = await build(changed);
			const changedCold = await build(changed, false);
			expect(changedBuild.output).toBe(changedCold.output);

			// A deeper checkout has additional ancestors the writer never searched.
			fs.writeFileSync(
				path.join(writer, 'src', 'index.js'),
				'try { console.log(require("optional")); } catch { console.log("missing"); }',
			);
			await build(writer);
			const deepReader = path.join(
				temp,
				...Array.from({ length: 20 }, (_, i) => `level-${i}`),
				'proj',
			);
			fs.mkdirSync(path.dirname(deepReader), { recursive: true });
			fs.cpSync(writer, deepReader, { recursive: true });
			const optional = path.join(temp, 'node_modules', 'optional');
			fs.mkdirSync(optional, { recursive: true });
			fs.writeFileSync(
				path.join(optional, 'package.json'),
				'{ "name": "optional", "main": "index.js" }',
			);
			fs.writeFileSync(path.join(optional, 'index.js'), 'module.exports = 99;');
			const deepMoved = await build(deepReader);
			const deepCold = await build(deepReader, false);
			expect(deepMoved.output).toBe(deepCold.output);
		} finally {
			removeTree(temp);
		}
	}
};
