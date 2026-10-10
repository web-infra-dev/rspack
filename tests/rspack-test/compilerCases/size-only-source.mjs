import { createFsFromVolume, Volume } from "memfs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [{
	description: "should retain only the on-disk size when emission is skipped",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			cache: true,
			incremental: false,
			output: { path: "/out", clean: false, compareBeforeEmit: true },
			plugins: [{
				apply(compiler) {
					const emitted = new Set();
					compiler.hooks.compilation.tap("SizeOnlySource", compilation => {
						compilation.hooks.processAssets.tap("SizeOnlySource", () => {
							const { RawSource } = compiler.rspack.sources;
							compilation.emitAsset("unchanged.txt", new RawSource("same"));
							compilation.emitAsset("immutable.txt", new RawSource("not written"), {
								immutable: true
							});
							compilation.emitAsset("fresh.txt", new RawSource("你好🌍"));
						});
					});
					compiler.hooks.assetEmitted.tap("SizeOnlySource", name => {
						emitted.add(name);
					});
					compiler.hooks.afterEmit.tap("SizeOnlySource", compilation => {
						expect(emitted.has("unchanged.txt")).toBe(false);
						expect(emitted.has("immutable.txt")).toBe(false);
						expect(emitted.has("fresh.txt")).toBe(true);
						for (const { name, source, info } of compilation.getAssets()) {
							const size = compiler.outputFileSystem.statSync(`/out/${name}`).size;
							expect(source).toBeInstanceOf(compiler.rspack.sources.SizeOnlySource);
							expect(source.size()).toBe(size);
							expect(info.size).toBe(size);
						}
					});
				}
			}]
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(Volume.fromJSON({
			"/out/unchanged.txt": "same",
			"/out/immutable.txt": "old"
		}));
	},
	check({ compiler, compilation }) {
		expect(compiler.outputFileSystem.readFileSync("/out/immutable.txt", "utf8")).toBe("old");
		expect(compilation.getAsset("immutable.txt").source.size()).toBe(3);
	}
}, {
	description: "should preserve source content when shouldEmit returns false",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			cache: true,
			plugins: [{
				apply(compiler) {
					compiler.hooks.shouldEmit.tap("SizeOnlySource", () => false);
					compiler.hooks.compilation.tap("SizeOnlySource", compilation => {
						compilation.hooks.processAssets.tap("SizeOnlySource", () => {
							compilation.emitAsset("kept.txt", new compiler.rspack.sources.RawSource("kept"));
						});
					});
				}
			}]
		};
	},
	check({ compiler, compilation, stats }) {
		const { source } = compilation.getAsset("kept.txt");
		expect(source).not.toBeInstanceOf(compiler.rspack.sources.SizeOnlySource);
		expect(source.source()).toBe("kept");
		expect(stats.logs.writeFile).toHaveLength(0);
	}
}, {
	description: "should preserve immutable on-disk sizes across no-op incremental builds",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			mode: "development",
			devtool: false,
			cache: true,
			incremental: { emitAssets: true },
			output: { path: "/out", filename: "[name].[contenthash].js", clean: false },
			plugins: [{
				apply(compiler) {
					let firstBuild = true;
					compiler.hooks.emit.tap("ImmutableDiskSize", compilation => {
						const [asset] = compilation.getAssets();
						expect(asset.info.immutable).toBe(true);
						expect(asset.source.size()).toBeGreaterThan(3);
						if (firstBuild) {
							compiler.outputFileSystem.mkdirSync("/out", { recursive: true });
							compiler.outputFileSystem.writeFileSync(`/out/${asset.name}`, "old");
							firstBuild = false;
						}
					});
				}
			}]
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(context, compiler) {
		let filename;
		for (let build = 0; build < 3; build++) {
			const stats = await context.getCompiler().build();
			expect(stats.hasErrors()).toBe(false);
			const [asset] = stats.compilation.getAssets();
			filename ??= asset.name;
			expect(asset.name).toBe(filename);
			expect(asset.source).toBeInstanceOf(compiler.rspack.sources.SizeOnlySource);
			expect(asset.source.size()).toBe(3);
			expect(asset.info.size).toBe(3);
			expect(stats.toJson({ all: false, assets: true, cachedAssets: true }).assets[0].size).toBe(3);
			expect(compiler.outputFileSystem.readFileSync(`/out/${filename}`, "utf8")).toBe("old");
		}
	}
}, {
	description: "should clean a stale file after its source-less asset disappears",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			mode: "development",
			devtool: false,
			cache: true,
			incremental: { emitAssets: true },
			output: { path: "/out", clean: true },
			plugins: [{
				apply(compiler) {
					let generation = 0;
					compiler.hooks.thisCompilation.tap("SourceLessAsset", compilation => {
						const current = generation++;
						compilation.hooks.processAssets.tap("SourceLessAsset", () => {
							if (current < 2) {
								compilation.emitAsset("stale.txt", new compiler.rspack.sources.RawSource("stale"));
								if (current === 1) delete compilation.assets["stale.txt"];
							}
						});
					});
				}
			}]
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(context, compiler) {
		for (let build = 0; build < 3; build++) {
			const stats = await context.getCompiler().build();
			expect(stats.hasErrors()).toBe(false);
			const asset = stats.compilation.getAsset("stale.txt");
			if (build === 1) {
				expect(asset).toBeDefined();
				expect(asset.source).toBeUndefined();
			} else if (build === 2) {
				expect(asset).toBeUndefined();
			}
			expect(compiler.outputFileSystem.existsSync("/out/stale.txt")).toBe(build < 2);
		}
	}
}, {
	description: "should cache emitted snapshots per info without reading size-only compilation assets",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			mode: "development",
			devtool: false,
			cache: true,
			output: { path: "/out", clean: false, compareBeforeEmit: false },
			plugins: [{
				apply(compiler) {
					const { RawSource, SourceMapSource, SizeOnlySource, util } = compiler.rspack.sources;
					const builds = [];
					context.setValue("emittedSnapshots", builds);
					compiler.hooks.emit.tap("EmittedSnapshots", compilation => {
						builds.push(new Map());
						const content = `build ${builds.length}`;
						compilation.emitAsset("cached.txt", new RawSource(content));
						compilation.emitAsset("empty.txt", new RawSource(""));
						compilation.emitAsset("lazy.txt", new SourceMapSource(content, "original.txt", {
							version: 3,
							sources: ["original.txt"],
							sourcesContent: [content],
							names: [],
							mappings: "AAAA"
						}));
					});
					compiler.hooks.assetEmitted.tap("CacheEmittedSnapshot", (name, info) => {
						if (!["cached.txt", "empty.txt", "lazy.txt"].includes(name)) return;
						expect(info.compilation.getAsset(name).source).toBeInstanceOf(SizeOnlySource);
						builds.at(-1).set(name, info);
						// Leave the mapped source unread until after a rebuild and compiler.close().
						if (name === "lazy.txt") return;
						const caching = util.stringBufferUtils;
						const wasEnabled = caching.isDualStringBufferCachingEnabled();
						try {
							caching.disableDualStringBufferCaching();
							const source = info.source;
							const content = info.content;
							expect(info.source).toBe(source);
							expect(info.content).toBe(content);
							expect(content.toString()).toBe(name === "empty.txt" ? "" : `build ${builds.length}`);
							if (content.length) content[0] = 66; // "build" -> "Build"
							source.clearCache();
							expect(info.content).toBe(content);
						} finally {
							if (wasEnabled) caching.enableDualStringBufferCaching();
						}
					});
					compiler.hooks.assetEmitted.tap("ReadCachedSnapshot", (name, info) => {
						if (name !== "cached.txt" && name !== "empty.txt") return;
						const previous = builds.at(-1).get(name);
						expect(info).toBe(previous);
						expect(info.source).toBe(previous.source);
						expect(info.content).toBe(previous.content);
						expect(info.content.toString()).toBe(name === "empty.txt" ? "" : `Build ${builds.length}`);
					});
				}
			}]
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async build(context, compiler) {
		for (let build = 1; build <= 2; build++) {
			const stats = await context.getCompiler().build();
			expect(stats.hasErrors()).toBe(false);
			// Sharing and mutating the hook's Buffer cannot change the already written file.
			expect(compiler.outputFileSystem.readFileSync("/out/cached.txt", "utf8")).toBe(`build ${build}`);
		}
		await context.closeCompiler();
		const [first, second] = context.getValue("emittedSnapshots");
		for (const name of ["cached.txt", "empty.txt", "lazy.txt"]) {
			expect(first.get(name)).not.toBe(second.get(name));
			expect(first.get(name).source).not.toBe(second.get(name).source);
			expect(first.get(name).content).not.toBe(second.get(name).content);
		}
		for (const [index, infos] of [first, second].entries()) {
			const info = infos.get("lazy.txt");
			const content = `build ${index + 1}`;
			expect(info.source.source()).toBe(content);
			expect(info.content.toString()).toBe(content);
			expect(info.source.map()).toMatchObject({
				sources: ["original.txt"], sourcesContent: [content], mappings: "AAAA"
			});
			expect(info.source.sourceAndMap().source).toBe(content);
			expect(info.source).toBe(info.source);
			expect(info.content).toBe(info.content);
		}
	}
}];
