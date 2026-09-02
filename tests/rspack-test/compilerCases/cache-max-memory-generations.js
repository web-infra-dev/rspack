const getMemoryCachePlugins = compiler =>
	compiler.cache.hooks.store.taps
		.map(tap => tap.name)
		.filter(name =>
			["MemoryCachePlugin", "MemoryWithGcCachePlugin"].includes(name)
		);

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should use an unbounded memory cache by default outside development mode",
		options() {
			return {
				cache: {
					type: "persistent"
				}
			};
		},
		async check({ compiler }) {
			expect(compiler.options.cache).toHaveProperty(
				"maxMemoryGenerations",
				Infinity
			);
			expect(getMemoryCachePlugins(compiler)).toEqual(["MemoryCachePlugin"]);
		}
	},
	{
		description:
			"should use a five-generation memory cache by default in development mode",
		options() {
			return {
				mode: "development",
				cache: {
					type: "persistent"
				}
			};
		},
		async check({ compiler }) {
			expect(compiler.options.cache).toHaveProperty("maxMemoryGenerations", 5);
			expect(getMemoryCachePlugins(compiler)).toEqual([
				"MemoryWithGcCachePlugin"
			]);
		}
	},
	{
		description:
			"should disable the additional memory cache when maxMemoryGenerations is zero",
		options() {
			return {
				cache: {
					type: "persistent",
					maxMemoryGenerations: 0
				}
			};
		},
		async check({ compiler }) {
			expect(compiler.options.cache).toHaveProperty("maxMemoryGenerations", 0);
			expect(getMemoryCachePlugins(compiler)).toEqual([]);
		}
	}
];
