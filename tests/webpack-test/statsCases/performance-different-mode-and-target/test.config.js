const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,2 +4,4 @@
		- WARNING in ⚠ asset size limit: The following asset(s) exceed the recommended size limit (XX KiB). This can impact web performance.Assets:
		- │   warning.pro-web.js (XX KiB)
		+ WARNING in asset size limit: The following asset(s) exceed the recommended size limit (XX KiB).
		+ This can impact web performance.
		+ Assets:
		+ warning.pro-web.js (XX KiB)
		@@ -7,3 +9,4 @@
		- WARNING in ⚠ entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.Entrypoints:
		- │   main (XX KiB)
		- │       warning.pro-web.js
		+ WARNING in entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		+ Entrypoints:
		+ main (XX KiB)
		+ warning.pro-web.js
		@@ -11,2 +14,3 @@
		- WARNING in ⚠ Rspack performance recommendations:You can limit the size of your bundles by using import() to lazy load some parts of your application.
		- │ For more info visit https://www.rspack.dev/guide/optimization/code-splitting
		+ WARNING in webpack performance recommendations:
		+ You can limit the size of your bundles by using import() or require.ensure to lazy load some parts of your application.
		+ For more info visit https://webpack.js.org/guides/code-splitting/
		@@ -14,1 +18,1 @@
		- Rspack x.x.x compiled with XX warnings in X.XX
		+ webpack x.x.x compiled with XX warnings in X ms
		@@ -19,2 +23,4 @@
		- WARNING in ⚠ asset size limit: The following asset(s) exceed the recommended size limit (XX KiB). This can impact web performance.Assets:
		- │   warning.pro-webworker.js (XX KiB)
		+ WARNING in asset size limit: The following asset(s) exceed the recommended size limit (XX KiB).
		+ This can impact web performance.
		+ Assets:
		+ warning.pro-webworker.js (XX KiB)
		@@ -22,3 +28,4 @@
		- WARNING in ⚠ entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.Entrypoints:
		- │   main (XX KiB)
		- │       warning.pro-webworker.js
		+ WARNING in entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		+ Entrypoints:
		+ main (XX KiB)
		+ warning.pro-webworker.js
		@@ -26,2 +33,3 @@
		- WARNING in ⚠ Rspack performance recommendations:You can limit the size of your bundles by using import() to lazy load some parts of your application.
		- │ For more info visit https://www.rspack.dev/guide/optimization/code-splitting
		+ WARNING in webpack performance recommendations:
		+ You can limit the size of your bundles by using import() or require.ensure to lazy load some parts of your application.
		+ For more info visit https://webpack.js.org/guides/code-splitting/
		@@ -29,1 +37,1 @@
		- Rspack x.x.x compiled with XX warnings in X.XX
		+ webpack x.x.x compiled with XX warnings in X ms
		@@ -33,1 +41,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms
		@@ -37,1 +45,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms
		@@ -41,1 +49,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms
		@@ -45,1 +53,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms
		@@ -50,2 +58,4 @@
		- WARNING in ⚠ asset size limit: The following asset(s) exceed the recommended size limit (XX KiB). This can impact web performance.Assets:
		- │   warning.pro-node-with-hints-set.js (XX KiB)
		+ WARNING in asset size limit: The following asset(s) exceed the recommended size limit (XX KiB).
		+ This can impact web performance.
		+ Assets:
		+ warning.pro-node-with-hints-set.js (XX KiB)
		@@ -53,3 +63,4 @@
		- WARNING in ⚠ entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.Entrypoints:
		- │   main (XX KiB)
		- │       warning.pro-node-with-hints-set.js
		+ WARNING in entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		+ Entrypoints:
		+ main (XX KiB)
		+ warning.pro-node-with-hints-set.js
		@@ -57,2 +68,3 @@
		- WARNING in ⚠ Rspack performance recommendations:You can limit the size of your bundles by using import() to lazy load some parts of your application.
		- │ For more info visit https://www.rspack.dev/guide/optimization/code-splitting
		+ WARNING in webpack performance recommendations:
		+ You can limit the size of your bundles by using import() or require.ensure to lazy load some parts of your application.
		+ For more info visit https://webpack.js.org/guides/code-splitting/
		@@ -60,1 +72,1 @@
		- Rspack x.x.x compiled with XX warnings in X.XX
		+ webpack x.x.x compiled with XX warnings in X ms"
	`);
	}
};
