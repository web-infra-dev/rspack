/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "uniqueName",
	options: () => ({
		output: {
			uniqueName: "@@@Hello World!",
			trustedTypes: true
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_test_tools",
		+     "chunkLoadingGlobal": "webpackChunk_Hello_World_",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/test-tools",
		+     "devtoolNamespace": "@@@Hello World!",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_test_tools",
		+     "hotUpdateGlobal": "webpackHotUpdate_Hello_World_",
		@@ ... @@
		-     "trustedTypes": undefined,
		-     "uniqueName": "@rspack/test-tools",
		+     "trustedTypes": Object {
		+       "policyName": "@@@Hello_World_",
		+     },
		+     "uniqueName": "@@@Hello World!",
	`)
};
