const path = require("path");

// Number large enough to exceed engine's max arguments (e.g. ~65536) when
// spread in push(...data), which would cause "Maximum call stack size exceeded".
const HUGE_DEPS_COUNT = 100_000;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		compiler => {
			compiler.hooks.done.tap("HugeFileDepsPlugin", ({ compilation }) => {
				const largeDeps = Array.from(
					{ length: HUGE_DEPS_COUNT },
					(_, i) => path.resolve(__dirname, `fake-dep-${i}.js`)
				);
				compilation.fileDependencies.addAll(largeDeps);
			});
		}
	]
};
