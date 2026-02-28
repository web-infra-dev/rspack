/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
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
			-     "chunkLoadingGlobal": "rspackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "rspackChunk_Hello_World_",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "@@@Hello World!",
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdate_Hello_World_",
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
