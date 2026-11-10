import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

export default [false, true].flatMap((newCache) =>
	[false, true].map((portable) => ({
		name: `${newCache ? "new" : "legacy"}-${portable ? "portable" : "local"}`,
		description: "reuses unchanged maps without resolving their source names again",
		async run() {
			const root = fs.realpathSync.native(
				fs.mkdtempSync(path.join(os.tmpdir(), "rspack-warm-map-"))
			);
			async function build(cacheName, concatSource = false) {
				const named = [];
				const compiler = rspack({
					context: root,
					mode: "development",
					entry: "./index.js",
					devtool: false,
					incremental: false,
					experiments: { newCache },
					cache: {
						type: "persistent",
						portable,
						storage: {
							type: "filesystem",
							directory: path.join(root, cacheName)
						}
					},
					output: { path: path.join(root, "dist") },
					plugins: [
						{
							apply(compiler) {
								compiler.hooks.thisCompilation.tap(
									"EquivalentSource",
									(compilation) => {
										compilation.hooks.processAssets.tap(
											{
												name: "EquivalentSource",
												stage:
													compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
											},
											() => {
												if (concatSource) {
													// Equivalent graphs retain the asset version and emitted map.
													for (const { name, source } of compilation.getAssets()) {
														compilation.updateAsset(
															name,
															new compiler.webpack.sources.ConcatSource(source)
														);
													}
												}
												compilation.emitAsset(
													"mapless.txt",
													new compiler.webpack.sources.RawSource("unmapped\n")
												);
											}
										);
									}
								);
							}
						},
						new rspack.SourceMapDevToolPlugin({
							filename: "[file].map",
							moduleFilenameTemplate(info) {
								named.push(info.identifier);
								return `webpack:///${info.resourcePath}`;
							}
						})
					]
				});
				try {
					const stats = await runCompiler(compiler);
					expect(stats.hasErrors()).toBe(false);
					expect(stats.hasWarnings()).toBe(false);
					return {
						named,
						built: stats.compilation.builtModules.size,
						assets: Object.fromEntries(
							fs.readdirSync(path.join(root, "dist")).map((file) => [
								file,
								fs.readFileSync(path.join(root, "dist", file))
							])
						)
					};
				} finally {
					await closeCompiler(compiler);
				}
			}
			try {
				fs.mkdirSync(path.join(root, ".git"));
				fs.writeFileSync(path.join(root, "package.json"), '{"name":"warm-map"}');
				fs.writeFileSync(
					path.join(root, "index.js"),
					'export const load = () => import("./async.js");\n'
				);
				fs.writeFileSync(path.join(root, "async.js"), "export default 42;\n");
				const first = await build("warm-cache", true);
				const warm = await build("warm-cache");
				const cold = await build("cold-cache");
				expect(first.named.length).toBeGreaterThan(0);
				if (!newCache) expect(warm.built).toBe(0);
				expect(warm.named).toEqual([]);
				expect(warm.assets).toEqual(cold.assets);
				expect(warm.assets).toEqual(first.assets);
			} finally {
				fs.rmSync(root, { recursive: true, force: true });
			}
		}
	}))
);
