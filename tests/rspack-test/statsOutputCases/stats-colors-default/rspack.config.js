/**
 * Tests that when stats.colors is not set, it defaults to environment support (issue #9353).
 * Build should succeed and stats output should be produced (colors on/off by env).
 */
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	stats: {
		preset: "errors-warnings",
	},
};
