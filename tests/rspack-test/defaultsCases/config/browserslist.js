const path = require("path");
defineDefaultsCase(Utils.casename(__filename), {
	description: "browserslist",
	options: context => {
		return {
			context: path.resolve(context.getSource(), "./browserslist")
		};
	},
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "context": "<cwd>/fixtures",
		+   "context": "<cwd>/fixtures/browserslist",
		@@ ... @@
		-       "arrowFunction": true,
		-       "asyncFunction": true,
		-       "bigIntLiteral": true,
		-       "const": true,
		-       "destructuring": true,
		+       "arrowFunction": false,
		+       "asyncFunction": false,
		+       "bigIntLiteral": false,
		+       "const": false,
		+       "destructuring": false,
		@@ ... @@
		-       "dynamicImport": undefined,
		-       "dynamicImportInWorker": undefined,
		-       "forOf": true,
		-       "globalThis": undefined,
		-       "module": undefined,
		-       "nodePrefixForCoreModules": true,
		-       "optionalChaining": true,
		-       "templateLiteral": true,
		+       "dynamicImport": false,
		+       "dynamicImportInWorker": false,
		+       "forOf": false,
		+       "globalThis": false,
		+       "module": false,
		+       "nodePrefixForCoreModules": false,
		+       "optionalChaining": false,
		+       "templateLiteral": false,
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk",
		+     "chunkLoadingGlobal": "webpackChunkbrowserslist_test",
		@@ ... @@
		-     "devtoolNamespace": "",
		+     "devtoolNamespace": "browserslist-test",
		@@ ... @@
		-       "arrowFunction": true,
		-       "asyncFunction": true,
		-       "bigIntLiteral": true,
		-       "const": true,
		-       "destructuring": true,
		+       "arrowFunction": false,
		+       "asyncFunction": false,
		+       "bigIntLiteral": false,
		+       "const": false,
		+       "destructuring": false,
		@@ ... @@
		-       "dynamicImport": undefined,
		-       "dynamicImportInWorker": undefined,
		-       "forOf": true,
		-       "globalThis": undefined,
		-       "module": undefined,
		-       "nodePrefixForCoreModules": true,
		-       "optionalChaining": true,
		-       "templateLiteral": true,
		+       "dynamicImport": false,
		+       "dynamicImportInWorker": false,
		+       "forOf": false,
		+       "globalThis": false,
		+       "module": false,
		+       "nodePrefixForCoreModules": false,
		+       "optionalChaining": false,
		+       "templateLiteral": false,
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate",
		+     "hotUpdateGlobal": "webpackHotUpdatebrowserslist_test",
		@@ ... @@
		-     "uniqueName": "",
		+     "uniqueName": "browserslist-test",
		@@ ... @@
		-       "<cwd>/fixtures",
		+       "<cwd>/fixtures/browserslist",
		@@ ... @@
		-   "target": "web",
		+   "target": "browserslist",
	`)
});
