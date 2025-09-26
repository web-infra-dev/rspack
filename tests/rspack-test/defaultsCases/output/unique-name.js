defineDefaultsCase(Utils.casename(__filename), {
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
		-     "chunkLoadingGlobal": "webpackChunk",
		+     "chunkLoadingGlobal": "webpackChunk_Hello_World_",
		@@ ... @@
		-     "devtoolNamespace": "",
		+     "devtoolNamespace": "@@@Hello World!",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate",
		+     "hotUpdateGlobal": "webpackHotUpdate_Hello_World_",
		@@ ... @@
		-     "trustedTypes": undefined,
		-     "uniqueName": "",
		+     "trustedTypes": Object {
		+       "onPolicyCreationFailure": "stop",
		+       "policyName": "@@@Hello_World_",
		+     },
		+     "uniqueName": "@@@Hello World!",
	`)
});
