import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";

const scenarios = [
	"persistent",
	"explicit-location",
	"memory",
	"disabled",
	"run-error",
	"stats-error",
	"close-error"
];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default scenarios.map(scenario => ({
	name: scenario,
	description: `should isolate and close independent shared compilers (${scenario})`,
	async run() {
		const directory = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-mf-shared-cache-"));
		const failure = new Error(`independent ${scenario}`);
		const persistent = !["memory", "disabled"].includes(scenario);
		let previousLocations;
		try {
			for (let round = 0; round < 2; round++) {
				const compilers = [];
				const closed = new Set();
				const locations = new Set();
				const cache = persistent ? {
					type: "persistent",
					name: "custom-name",
					storage: {
						type: "filesystem",
						directory: path.join(directory, "cache"),
						...(scenario === "explicit-location" ? {
							location: path.join(directory, "custom-location")
						} : {})
					}
				} : scenario === "memory" ? { type: "memory" } : false;
				const compiler = rspack({
					context: import.meta.dirname,
					mode: "production",
					target: "async-node",
					entry: "./index.js",
					output: { path: path.join(directory, "dist") },
					optimization: { minimize: false },
					cache,
					experiments: { newCache: true },
					resolve: {
						alias: {
							"shared-a": path.join(import.meta.dirname, "shared-a.js"),
							"shared-b": path.join(import.meta.dirname, "shared-b.js")
						}
					},
					plugins: [
						new rspack.container.ModuleFederationPlugin({
							name: "shared_cache_test",
							shared: Object.fromEntries(["shared-a", "shared-b"].map(name => [name, {
								import: `./${name}.js`,
								version: "1.0.0",
								treeShaking: { mode: "runtime-infer" }
							}]))
						}),
						{
							apply(current) {
								compilers.push(current);
								if (compilers.length === 1) return;
								const first = compilers.length === 2;
								const close = current.close.bind(current);
								current.close = callback => close(error => {
									setImmediate(() => {
										closed.add(current);
										callback(error || (first && scenario === "close-error" ? failure : undefined));
									});
								});
								current.hooks.beforeRun.tapAsync("SharedCacheTest", (_compiler, callback) => {
									// Fail before opening a second database if cache isolation regresses.
									try {
										if (persistent) {
											const location = current.options.cache.storage.location;
											expect(location).not.toBe(compiler.options.cache.storage.location);
											expect(locations.has(location)).toBe(false);
											locations.add(location);
										} else {
											expect(current.options.cache).toEqual(compiler.options.cache);
										}
									} catch (error) {
										callback(error);
										return;
									}
									setImmediate(() => callback(first && scenario === "run-error" ? failure : undefined));
								});
								if (first && scenario === "stats-error") {
									current.hooks.thisCompilation.tap("SharedCacheTest", compilation => {
										compilation.errors.push(failure);
									});
								}
							}
						}
					]
				});
				try {
					if (scenario.endsWith("-error")) {
						await expect(runCompiler(compiler)).rejects.toThrow(
							scenario === "stats-error" ? "Compile failed" : failure.message
						);
					} else {
						const stats = await runCompiler(compiler);
						expect(stats.hasErrors()).toBe(false);
					}
					expect(compilers).toHaveLength(3);
					expect(closed).toEqual(new Set(compilers.slice(1)));
					if (previousLocations) expect(locations).toEqual(previousLocations);
					previousLocations = locations;
				} finally {
					await Promise.all(compilers.filter(c => !closed.has(c)).map(closeCompiler));
				}
			}
		} finally {
			fs.rmSync(directory, { recursive: true, force: true });
		}
	}
}));
