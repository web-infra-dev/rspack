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
}];
