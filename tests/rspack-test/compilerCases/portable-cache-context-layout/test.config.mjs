import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

async function build({ context, entry, root, storage, cache = true }) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		entry,
		output: {
			path: path.join(context, cache ? "dist" : "dist-cold"),
			library: { type: "commonjs2" }
		},
		target: "node",
		node: { __dirname: true, __filename: true },
		cache: cache
			? {
					type: "persistent",
					portable: root ? { root } : true,
					storage: { type: "filesystem", directory: storage }
				}
			: false
	});
	try {
		const stats = await runCompiler(compiler);
		const json = stats.toJson({ all: false, errors: true, modules: true });
		expect(json.errors).toEqual([]);
		return {
			built: json.modules
				.filter(module => module.built)
				.map(module => path.basename(module.name))
				.sort(),
			output: fs.readFileSync(
				path.join(compiler.options.output.path, "main.js"),
				"utf-8"
			)
		};
	} finally {
		await closeCompiler(compiler);
	}
}

function tempDir() {
	return fs.realpathSync.native(
		fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-layout-"))
	);
}

function copySources(target) {
	fs.mkdirSync(target, { recursive: true });
	fs.cpSync(path.join(import.meta.dirname, "src"), path.join(target, "src"), {
		recursive: true
	});
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should not reuse context-relative values when context moves within the portable root",
	async run() {
		// A worktree-like move of the whole root keeps the layout, so every module is reused.
		{
			const base = tempDir();
			try {
				const writer = path.join(base, "a", "repo");
				copySources(writer);
				fs.mkdirSync(path.join(writer, ".git"));
				const storage = path.join(writer, ".cache");
				await build({ context: writer, entry: "./src/index.js", storage });
				const reader = path.join(base, "b", "repo");
				fs.cpSync(writer, reader, { recursive: true });
				const next = await build({
					context: reader,
					entry: "./src/index.js",
					storage: path.join(reader, ".cache")
				});
				const cold = await build({
					context: reader,
					entry: "./src/index.js",
					cache: false
				});
				expect(next.output).toBe(cold.output);
			} finally {
				fs.rmSync(base, { recursive: true, force: true });
			}
		}

		// The same root, with context reached through a symlink inside it, then directly.
		// Skipped on Windows, where directory symlinks need extra privileges.
		if (process.platform !== "win32") {
			const base = tempDir();
			try {
				const repo = path.join(base, "repo");
				fs.mkdirSync(path.join(repo, ".git"), { recursive: true });
				copySources(path.join(repo, "packages", "pkg"));
				fs.symlinkSync(
					path.join(repo, "packages"),
					path.join(repo, "link"),
					"dir"
				);
				const storage = path.join(repo, ".cache");
				await build({
					context: path.join(repo, "link", "pkg"),
					entry: "./src/index.js",
					storage
				});
				const reader = path.join(repo, "packages", "pkg");
				const next = await build({
					context: reader,
					entry: "./src/index.js",
					storage
				});
				const cold = await build({
					context: reader,
					entry: "./src/index.js",
					cache: false
				});
				expect(next.output).toBe(cold.output);
			} finally {
				fs.rmSync(base, { recursive: true, force: true });
			}
		}

		// An explicit root below context, renamed in the copy.
		{
			const base = tempDir();
			try {
				const writer = path.join(base, "a");
				copySources(path.join(writer, "pkg"));
				await build({
					context: writer,
					entry: "./pkg/src/index.js",
					root: "./pkg",
					storage: path.join(writer, ".cache")
				});
				const reader = path.join(base, "b");
				fs.cpSync(writer, reader, { recursive: true });
				fs.renameSync(path.join(reader, "pkg"), path.join(reader, "renamed"));
				const next = await build({
					context: reader,
					entry: "./renamed/src/index.js",
					root: "./renamed",
					storage: path.join(reader, ".cache")
				});
				const cold = await build({
					context: reader,
					entry: "./renamed/src/index.js",
					cache: false
				});
				expect(next.output).toBe(cold.output);
			} finally {
				fs.rmSync(base, { recursive: true, force: true });
			}
		}
	}
};
