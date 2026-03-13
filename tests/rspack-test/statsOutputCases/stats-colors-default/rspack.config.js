/**
 * Tests that when stats.colors is not set, it defaults to environment support (issue #9353).
 * The rspack test runner sets NO_COLOR='1' globally (rstest.config.ts), so the default is false
 * and we get uncolored output. This case asserts that the env-based default is applied.
 */
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		preset: "errors-warnings",
	},
};
