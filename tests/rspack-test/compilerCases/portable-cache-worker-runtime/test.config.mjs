import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

async function build(root) {
	const compiler = rspack({
		context: root,
		mode: "development",
		devtool: false,
		target: "web",
		entry: "./src/index.js",
		output: { path: path.join(root, "dist"), clean: true },
		cache: {
			type: "persistent",
			portable: true,
			storage: { type: "filesystem", location: path.join(root, ".cache") }
		},
		experiments: { portableCache: { seed: false } },
		// HMR prints the full hash, which every worker runtime name feeds.
		plugins: [new rspack.HotModuleReplacementPlugin()]
	});
	try {
		const stats = await runCompiler(compiler);
		expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
		return {
			built: [...stats.compilation.builtModules].length,
			hash: stats.compilation.hash,
			assets: fs
				.readdirSync(path.join(root, "dist"))
				.sort()
				.map(name => [name, fs.readFileSync(path.join(root, "dist", name), "utf-8")])
		};
	} finally {
		await closeCompiler(compiler);
	}
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "should name worker runtimes the same way in every checkout",
	async run() {
		const temp = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-worker-runtime-"))
		);
		try {
			const writer = path.join(temp, "writer", "proj");
			const reader = path.join(temp, "reader-with-longer-root", "proj");
			fs.mkdirSync(path.join(writer, ".git"), { recursive: true });
			fs.mkdirSync(path.join(writer, "src"));
			fs.writeFileSync(
				path.join(writer, "src/index.js"),
				'new Worker(new URL("./worker.js", import.meta.url));'
			);
			fs.writeFileSync(path.join(writer, "src/worker.js"), "self.postMessage(1);");
			await build(writer);
			fs.mkdirSync(path.dirname(reader), { recursive: true });
			fs.cpSync(writer, reader, { recursive: true });
			const moved = await build(reader);
			fs.rmSync(path.join(reader, ".cache"), { recursive: true, force: true });
			const cold = await build(reader);
			expect(moved.built).toBeLessThan(cold.built);
			expect(moved.hash).toBe(cold.hash);
			expect(moved.assets).toEqual(cold.assets);
		} finally {
			fs.rmSync(temp, { recursive: true, force: true });
		}
	}
};
