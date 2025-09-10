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
		-     "chunkLoadingGlobal": "webpackChunk_rspack_tests",
		+     "chunkLoadingGlobal": "webpackChunk_Hello_World_",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/tests",
		+     "devtoolNamespace": "@@@Hello World!",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_tests",
		+     "hotUpdateGlobal": "webpackHotUpdate_Hello_World_",
		@@ ... @@
		-     "trustedTypes": undefined,
		-     "uniqueName": "@rspack/tests",
		+     "trustedTypes": Object {
		+       "onPolicyCreationFailure": "stop",
		+       "policyName": "@@@Hello_World_",
		+     },
		+     "uniqueName": "@@@Hello World!",
	`)
};
