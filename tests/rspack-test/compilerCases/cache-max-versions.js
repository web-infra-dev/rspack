const { start } = require("@rspack/test-tools/helper/legacy/deprecationTracking");

let tracker;

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should not warn when cache.maxVersions is not explicitly configured",
		options() {
			tracker = start();
			return {
				cache: {
					type: "persistent"
				}
			};
		},
		async build() {},
		async check({ compiler }) {
			expect(tracker()).toHaveLength(0);
			expect(compiler.options.cache).not.toHaveProperty("maxVersions");
		}
	},
	{
		description:
			"should retain and deprecate an explicitly configured cache.maxVersions",
		options(context) {
			tracker = start();
			return {
				cache: {
					type: "persistent",
					maxVersions: 1,
					storage: {
						type: "filesystem",
						directory: context.getDist()
					}
				}
			};
		},
		async check({ compiler }) {
			const deprecations = tracker();
			expect(deprecations).toHaveLength(1);
			expect(deprecations[0].message).toContain(
				"`cache.maxVersions` is deprecated and has no effect"
			);
			expect(compiler.options.cache).toHaveProperty("maxVersions", 1);
		}
	}
];
