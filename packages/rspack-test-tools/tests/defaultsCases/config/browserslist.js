const path = require("path");
module.exports = {
	description: "browserslist",
	options: context => ({
		context: path.resolve(context.getSource(), "./browserslist")
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
    - Expected
    + Received

    @@ ... @@
    -   "context": "<cwd>",
    +   "context": "<cwd>/tests/fixtures/browserslist",
    @@ ... @@
    -     "chunkLoadingGlobal": "webpackChunk_rspack_core",
    +     "chunkLoadingGlobal": "webpackChunkbrowserslist_test",
    @@ ... @@
    -     "devtoolNamespace": "@rspack/core",
    +     "devtoolNamespace": "browserslist-test",
    @@ ... @@
    -     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
    +     "hotUpdateGlobal": "webpackHotUpdatebrowserslist_test",
    @@ ... @@
    -     "uniqueName": "@rspack/core",
    +     "uniqueName": "browserslist-test",
  `)
};
