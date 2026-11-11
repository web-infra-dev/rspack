import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

const ENTRIES = 6;

// Size and modification time of every file the source map cache keeps.
function sourceMapCacheFiles(directory) {
	const files = new Map();
	const walk = (current) => {
		for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
			const file = path.join(current, entry.name);
			if (entry.isDirectory()) {
				walk(file);
			} else if (file.includes("source_map")) {
				const stat = fs.statSync(file);
				files.set(file, `${stat.size}:${stat.mtimeMs}`);
			}
		}
	};
	walk(directory);
	return files;
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "writes only the changed source map cache entry after an edit",
	async run() {
		const root = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-map-edit-"))
		);
		const cache = path.join(root, "cache");
		// Each map is large enough that the cache stores it in a pack file of its own.
		const write = (index, revision) =>
			fs.writeFileSync(
				path.join(root, `e${index}.js`),
				`export default ${JSON.stringify(revision.padEnd(700_000, String(index)))};\n`
			);
		async function build() {
			const compiler = rspack({
				context: root,
				mode: "development",
				devtool: "source-map",
				entry: Object.fromEntries(
					Array.from({ length: ENTRIES }, (_, index) => [`e${index}`, `./e${index}.js`])
				),
				output: { path: path.join(root, "dist") },
				cache: { type: "persistent", storage: { type: "filesystem", directory: cache } }
			});
			try {
				const stats = await runCompiler(compiler);
				expect(stats.hasErrors()).toBe(false);
				return stats.compilation.builtModules.size;
			} finally {
				await closeCompiler(compiler);
			}
		}
		try {
			for (let index = 0; index < ENTRIES; index++) write(index, "a");
			await build();
			expect(await build()).toBe(0);
			const before = sourceMapCacheFiles(cache);
			expect(before.size).toBeGreaterThanOrEqual(ENTRIES);

			write(0, "b");
			expect(await build()).toBe(1);
			const after = sourceMapCacheFiles(cache);
			const kept = [...before].filter(([file, stamp]) => after.get(file) === stamp);
			// The maps of the five untouched entries stay as they are on disk. A new compilation
			// hash used to rewrite every entry.
			expect(kept.length).toBeGreaterThanOrEqual(ENTRIES - 2);
		} finally {
			removeTree(root);
		}
	}
};
