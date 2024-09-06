const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -12,4 +12,4 @@
		- <CLR=XX,BOLD>WARNING</CLR> in <CLR=XX>⚠<CLR=XX> asset size limit: The following asset(s) exceed the recommended size limit (XX KiB). This can impact web performance.
		- <CLR=XX>│<CLR=XX> Assets:
		- <CLR=XX>│<CLR=XX>   main.js (XX KiB)
		- <CLR=XX>
		+ <CLR=XX,BOLD>WARNING</CLR> in <CLR=BOLD>asset size limit: The following asset(s) exceed the recommended size limit (XX KiB).
		+ This can impact web performance.
		+ Assets:
		+ main.js (XX KiB)</CLR>
		@@ -17,5 +17,5 @@
		- <CLR=XX,BOLD>WARNING</CLR> in <CLR=XX>⚠<CLR=XX> entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		- <CLR=XX>│<CLR=XX> Entrypoints:
		- <CLR=XX>│<CLR=XX>   main (XX KiB)
		- <CLR=XX>│<CLR=XX>       main.js
		- <CLR=XX>
		+ <CLR=XX,BOLD>WARNING</CLR> in <CLR=BOLD>entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		+ Entrypoints:
		+ main (XX KiB)
		+ main.js
		+ </CLR>
		@@ -23,4 +23,3 @@
		- <CLR=XX,BOLD>WARNING</CLR> in <CLR=XX>⚠<CLR=XX> Rspack performance recommendations:
		- <CLR=XX>│<CLR=XX> You can limit the size of your bundles by using import() to lazy load some parts of your application.
		- <CLR=XX>│<CLR=XX> For more info visit https://www.rspack.dev/guide/optimization/code-splitting
		- <CLR=XX>
		+ <CLR=XX,BOLD>WARNING</CLR> in <CLR=BOLD>webpack performance recommendations:
		+ You can limit the size of your bundles by using import() or require.ensure to lazy load some parts of your application.
		+ For more info visit https://webpack.js.org/guides/code-splitting/</CLR>
		@@ -28,1 +27,1 @@
		- Rspack x.x.x compiled with <CLR=XX,BOLD>XX warnings</CLR> in X s
		+ webpack x.x.x compiled with <CLR=XX,BOLD>XX warnings</CLR> in X ms"
	`);
	}
};
