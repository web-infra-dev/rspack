// BE CAUTIOUS:
// This file is not intended to include everything that share the same name between webpack and rspack.
// The ultimate goal of this is to remove the exclusion list completely.
// Only the following can be added here:
// - the extended tests of the original webpack test
// - partially passed tests
//
// Also, to note, only ignore a SINGLE test instead of the whole test category.

// Test partially passed.
// This should strictly be used only for partially passed webpack tests.
// NOT intended for additional tests added in webpack, for this kind of test, please use `ADDITIONAL_TESTS`
// Checkout webpack's `test.filter.js` for detail.
const PARTIALLY_PASSED = [
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
	"configCases/trusted-types/custom-policy-name",
	"configCases/trusted-types/default-policy-name",
	"configCases/trusted-types/no-policy-name",
	"configCases/trusted-types/no-trusted-types",

	"hotCases/recover/recover-after-self-error",
	"hotCases/worker/move-between-runtime",

	"statsCases/limit-chunk-count-plugin",
	"statsCases/parse-error"
];

// Webpack test fully passed, with additional test with the same name as webpack in rspack.
const ADDITIONAL_TESTS = [
	"cases/cjs-tree-shaking/non-root-this",
	"configCases/split-chunks/custom-filename",
	"configCases/loader-import-module/css"
];

// Not aligned fixtures:
const UNALIGNED_FIXTURES = [
	// TO BE ALIGNED:
	"fixtures/buildDependencies/index.js",

	// Added filtered test filter
	"ConfigTestCases.template.js",
	"TestCases.template.js",

	// Change marked
	"checkArrayExpectation.js",
	"WatchSuspend.test.js",
	"helpers/FakeDocument.js",
	"helpers/warmup-webpack.js",
	"hotCases/fake-update-loader.js",
	"StatsTestCases.basictest.js",
	"WatchTestCases.template.js",
	"HotTestCases.template.js",
	"HotTestCasesNode.test.js",
	"Compiler.text.js"
];

// Only different in comments. For example, license information difference.
const DIFFERENT_IN_COMMENTS = [
	"helpers/EventSourceForNode.js",
	"helpers/deprecationTracking.js"
];

// Webpack's test case seems not correct, but rspack fixed this.
// And we left webpack test untouched.
const WEBPACK_TEST_FIX = ["cases/context/ignore-hidden-files"];

/**
 * @type {Array<[RegExp | string, string]>}
 */
module.exports = [
	// Intended to have different README.md
	"README.md",
	// Intended to have different package.json
	"package.json",
	// output file
	"js"
]
	.concat(PARTIALLY_PASSED)
	.concat(ADDITIONAL_TESTS)
	.concat(UNALIGNED_FIXTURES)
	.concat(DIFFERENT_IN_COMMENTS)
	.concat(WEBPACK_TEST_FIX);

module.exports = Object.assign(module.exports, {
	PARTIALLY_PASSED
});
