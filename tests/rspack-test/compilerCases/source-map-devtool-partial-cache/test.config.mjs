import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

async function build(
	root,
	cacheName,
	changed,
	newCache,
	identityA = "worker.js",
	omitA = false
) {
	const codeA = identityA === "worker.js" ? "a();\n" : "a(2);\n";
	fs.writeFileSync(path.join(root, "a.js"), codeA);
	const originals = {};
	const mapped = [];
	fs.writeFileSync(path.join(root, "b.js"), changed ? "b(2);\n" : "b(1);\n");
	const compiler = rspack({
		context: root,
		mode: "development",
		entry: { a: "./a.js", b: "./b.js" },
		// Configure the map plugin explicitly so its append callback observes actual misses.
		devtool: false,
		incremental: false,
		experiments: { newCache },
		cache: {
			type: "persistent",
			storage: { type: "filesystem", directory: path.join(root, cacheName) }
		},
		output: { path: path.join(root, "dist"), clean: true },
		plugins: [
			new rspack.SourceMapDevToolPlugin({
				filename: "[file].map",
				append(data) {
					mapped.push(data.filename);
					return "\n//# sourceMappingURL=[url]";
				}
			}),
			{
				apply(compiler) {
					compiler.hooks.thisCompilation.tap(
						"CollisionAssets",
						(compilation) => {
							compilation.hooks.processAssets.tap(
								{
									name: "CollisionAssets",
									stage:
										compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
								},
								() => {
									for (const [file, identity, code] of [
										["a.js", identityA, codeA],
										[
											"b.js",
											"loader!worker.js",
											changed ? "b(2);\n" : "b(1);\n"
										]
									]) {
										if (omitA && file === "a.js") {
											compilation.deleteAsset(file);
											continue;
										}
										const map = {
											version: 3,
											sources: [identity],
											sourcesContent: [code],
											names: [],
											mappings: "AAAA"
										};
										compilation.updateAsset(
											file,
											new compiler.webpack.sources.SourceMapSource(
												code,
												identity,
												map
											)
										);
									}
								}
							);
							compilation.hooks.processAssets.tap(
								{
									name: "CaptureInputMaps",
									stage:
										compiler.webpack.Compilation
											.PROCESS_ASSETS_STAGE_DEV_TOOLING - 1
								},
								() => {
									for (const { name, source } of compilation.getAssets()) {
										originals[name] = JSON.stringify(source.map());
									}
								}
							);
						}
					);
				}
			}
		]
	});
	try {
		const stats = await runCompiler(compiler);
		const json = stats.toJson({
			all: false,
			errors: true,
			warnings: true,
			logging: "verbose",
			cacheInfo: true
		});
		expect(json.errors).toEqual([]);
		expect(json.warnings).toEqual([]);
		const assets = Object.fromEntries(
			fs
				.readdirSync(path.join(root, "dist"))
				.sort()
				.map((file) => [file, fs.readFileSync(path.join(root, "dist", file))])
		);
		return {
			hash: stats.hash,
			originals,
			assets,
			mapped: mapped.sort(),
			logging: json.logging,
			cacheInfo: json.cacheInfo
		};
	} finally {
		await closeCompiler(compiler);
	}
}

export default [false, true].map((newCache) => ({
	name: newCache ? "new-cache" : "legacy-cache",
	description:
		"keeps cached conflicting sources in devtool filename collision resolution",
	async run() {
		const root = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-map-collision-"))
		);
		try {
			fs.mkdirSync(path.join(root, ".git"));
			fs.writeFileSync(
				path.join(root, "package.json"),
				'{"name":"map-collision"}'
			);
			fs.writeFileSync(path.join(root, "a.js"), "a();\n");
			const first = await build(root, "warm-cache", false, newCache);
			const warm = await build(root, "warm-cache", true, newCache);
			const cold = await build(root, "cold-cache", true, newCache);
			const expectCache = (result, hits, misses) => {
				if (newCache) {
					expect(
						Object.entries(result.cacheInfo).find(([key]) =>
							key.endsWith("|rspack.SourceMapDevToolPlugin")
						)?.[1]
					).toEqual(expect.objectContaining({ hits, misses, reused: hits }));
				} else {
					const entry = result.logging[
						"rspack.SourceMapDevToolPlugin"
					].entries.find((entry) => entry.type === "cache");
					expect(entry.message).toContain(`(${hits}/${hits + misses})`);
				}
			};
			console.log(
				"collision cache evidence",
				JSON.stringify({
					newCache,
					mapped: [first.mapped, warm.mapped, cold.mapped],
					cache: [first.cacheInfo, warm.cacheInfo, cold.cacheInfo],
					legacy: [
						first.logging["rspack.SourceMapDevToolPlugin"],
						warm.logging["rspack.SourceMapDevToolPlugin"],
						cold.logging["rspack.SourceMapDevToolPlugin"]
					]
				})
			);
			expectCache(first, 0, 2);
			expectCache(warm, 1, 1);
			expectCache(cold, 0, 2);
			expect(first.mapped).toEqual(["a.js", "b.js"]);
			expect(warm.mapped).toEqual(["b.js"]);
			expect(cold.mapped).toEqual(["a.js", "b.js"]);
			expect(warm.originals).toEqual(cold.originals);
			expect(JSON.parse(warm.originals["a.js"]).sources).toEqual(["worker.js"]);
			expect(JSON.parse(warm.originals["b.js"]).sources).toEqual([
				"loader!worker.js"
			]);
			const coldA = JSON.parse(cold.assets["a.js.map"]);
			const coldB = JSON.parse(cold.assets["b.js.map"]);
			expect(coldA.sources).toEqual(["webpack:///worker.js"]);
			expect(coldB.sources).toEqual(["webpack:///worker.js?1b50"]);
			console.log(
				"cold raw maps",
				cold.assets["a.js.map"].toString(),
				cold.assets["b.js.map"].toString()
			);
			console.log(
				"warm raw maps",
				warm.assets["a.js.map"].toString(),
				warm.assets["b.js.map"].toString()
			);
			expect(warm.assets).toEqual(cold.assets);
			const repeat = await build(root, "warm-cache", true, newCache);
			expectCache(repeat, 2, 0);
			expect(repeat.mapped).toEqual([]);
			expect(repeat.assets).toEqual(cold.assets);

			// Changing A's content and source identity removes the collision. B keeps its version,
			// but its previously resolved name must still be invalidated.
			const changedContext = await build(
				root,
				"warm-cache",
				true,
				newCache,
				"other.js"
			);
			const coldContext = await build(
				root,
				"other-cold-cache",
				true,
				newCache,
				"other.js"
			);
			expectCache(changedContext, 0, 2);
			expect(changedContext.mapped).toEqual(["a.js", "b.js"]);
			expect(changedContext.assets).toEqual(coldContext.assets);
			expect(JSON.parse(changedContext.assets["b.js.map"]).sources).toEqual([
				"webpack:///worker.js"
			]);

			// Removing an asset after hashing must invalidate its former collision partner.
			const beforeRemove = await build(root, "remove-cache", true, newCache);
			const removed = await build(root, "remove-cache", true, newCache, "worker.js", true);
			const coldRemoved = await build(root, "remove-cold", true, newCache, "worker.js", true);
			expect(removed.hash).toBe(beforeRemove.hash);
			expectCache(removed, 0, 1);
			expect(removed.mapped).toEqual(["b.js"]);
			expect(removed.assets).toEqual(coldRemoved.assets);
			expect(JSON.parse(removed.assets["b.js.map"]).sources).toEqual([
				"webpack:///worker.js"
			]);
			const repeatRemoved = await build(root, "remove-cache", true, newCache, "worker.js", true);
			expectCache(repeatRemoved, 1, 0);
			expect(repeatRemoved.mapped).toEqual([]);
			expect(repeatRemoved.assets).toEqual(coldRemoved.assets);
		} finally {
			removeTree(root);
		}
	}
}));
