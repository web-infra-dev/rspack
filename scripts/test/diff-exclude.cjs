// BE CAUTIOUS:
// This file is not intended to include everything that share the same name between webpack and rspack.
// The ultimate goal of this is to remove the exclusion list completely.
// Only the following can be added here:
// - the extended tests of the original webpack test
// - partially passed tests
//
// Also, to note, only ignore a SINGLE test instead of the whole test category.

/**
 * @type {Array<RegExp | string>}
 */
module.exports = [
	// Intended to have different README.md
	"README.md",

	// Change marked
	"WatchSuspend.test.js",

	// Test partially passed.
	// Checkout webpack's `test.filter.js` for detail.
	"cases/chunks/context",
	"cases/chunks/weak-dependencies",
	"cases/parsing/harmony-deep-exports",
	"cases/esm/import-meta",
	"cases/mjs/type-module",
	"cases/parsing/harmony-export-import-specifier",
	"cases/parsing/harmony-reexport",
	"cases/parsing/renaming",
	"cases/parsing/typeof",

	"configCases/css/urls",
	"configCases/source-map/relative-source-map-path",
	"configCases/source-map/relative-source-maps-by-loader",
	"configCases/source-map/resource-path",
	"configCases/source-map/source-map-filename-contenthash",
	"configCases/trusted-types/custom-policy-name",
	"configCases/trusted-types/default-policy-name",
	"configCases/trusted-types/no-policy-name",
	"configCases/trusted-types/no-trusted-types",

	// Webpack test fully passed, with additional test in rspack.
	"cases/cjs-tree-shaking/non-root-this",
	"configCases/split-chunks/custom-filename",

	// Webpack's test case seems not correct, but rspack fixed this.
	// And we left webpack test untouched.
	"cases/context/ignore-hidden-files"
];
