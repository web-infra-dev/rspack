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
    -     "chunkLoadingGlobal": "webpackChunk_rspack_core",
    +     "chunkLoadingGlobal": "webpackChunk_Hello_World_",
    @@ ... @@
    -     "devtoolNamespace": "@rspack/core",
    +     "devtoolNamespace": "@@@Hello World!",
    @@ ... @@
    -     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
    +     "hotUpdateGlobal": "webpackHotUpdate_Hello_World_",
    @@ ... @@
    -     "trustedTypes": undefined,
    -     "uniqueName": "@rspack/core",
    +     "trustedTypes": Object {
    +       "policyName": "@@@Hello_World_",
    +     },
    +     "uniqueName": "@@@Hello World!",
  `)
};
