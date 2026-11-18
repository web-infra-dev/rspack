import { runInNewContext } from "node:vm";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description: "should remove unused callbacks passed to no-op wrappers while preserving argument side effects",
	options() {
		return {
			entry: "./index.js",
			mode: "production",
			target: "web",
			devtool: false,
			output: {
				filename: "main.cjs",
				library: { type: "commonjs2" },
				pathinfo: false
			},
			optimization: {
				concatenateModules: true,
				minimize: true,
				minimizer: ["..."]
			}
		};
	},
	check({ compilation }) {
		const { modules } = compilation.getStats().toJson({
			all: false,
			modules: true,
			orphanModules: true,
			nestedModules: true
		});
		const concatenated = modules.find(module =>
			module.modules?.some(nested => nested.name === "./index.js")
		);
		expect(concatenated?.modules.map(module => module.name)).toEqual(
			expect.arrayContaining(["./index.js", "./noop-wrapper.js"])
		);

		const source = compilation.getAsset("main.cjs").source.source().toString();
		expect(source).not.toContain("MINIMAL_CALLBACK_BODY");

		const module = { exports: {} };
		runInNewContext(source, { module });
		expect(module.exports.run(42)).toEqual({ value: 42, predicateCalls: 0 });
		expect(module.exports.runFactory()).toEqual({
			factoryCalls: 1,
			predicateCalls: 0
		});
	}
};
