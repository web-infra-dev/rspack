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
}];
