const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -1,4 +1,4 @@
		- asset <CLR=XX,BOLD>main.js</CLR> XX KiB <CLR=XX,BOLD>[emitted]</CLR> (name: main)
		- asset <CLR=XX,BOLD>sec.js</CLR> XX KiB <CLR=XX,BOLD>[emitted]</CLR> (name: sec)
		- Entrypoint <CLR=BOLD>main</CLR> XX KiB = <CLR=XX,BOLD>main.js</CLR>
		- Entrypoint <CLR=BOLD>sec</CLR> XX KiB = <CLR=XX,BOLD>sec.js</CLR>
		+ asset <CLR=XX,BOLD>main.js</CLR> <CLR=XX,BOLD>XX KiB</CLR> <CLR=XX,BOLD>[emitted]</CLR> <CLR=XX,BOLD>[big]</CLR> (name: main)
		+ asset <CLR=XX,BOLD>sec.js</CLR> <CLR=XX,BOLD>XX KiB</CLR> <CLR=XX,BOLD>[emitted]</CLR> <CLR=XX,BOLD>[big]</CLR> (name: sec)
		+ Entrypoint <CLR=BOLD>main</CLR> <CLR=XX,BOLD>[big]</CLR> XX KiB = <CLR=XX,BOLD>main.js</CLR>
		+ Entrypoint <CLR=BOLD>sec</CLR> <CLR=XX,BOLD>[big]</CLR> XX KiB = <CLR=XX,BOLD>sec.js</CLR>
		@@ -6,1 +6,0 @@
		- <CLR=BOLD>./a.js</CLR> XX KiB <CLR=XX,BOLD>[built]</CLR> <CLR=XX,BOLD>[code generated]</CLR>
		@@ -8,0 +7,1 @@
		+ <CLR=BOLD>./a.js</CLR> XX KiB <CLR=XX,BOLD>[built]</CLR> <CLR=XX,BOLD>[code generated]</CLR>
		@@ -9,5 +9,5 @@
		- <CLR=XX,BOLD>ERROR</CLR> in <CLR=XX>×<CLR=XX> asset size limit: The following asset(s) exceed the recommended size limit (XX KiB). This can impact web performance.
		- <CLR=XX>│<CLR=XX> Assets:
		- <CLR=XX>│<CLR=XX>   main.js (XX KiB)
		- <CLR=XX>│<CLR=XX>   sec.js (XX KiB)
		- <CLR=XX>
		+ <CLR=XX,BOLD>ERROR</CLR> in <CLR=BOLD>asset size limit: The following asset(s) exceed the recommended size limit (XX KiB).
		+ This can impact web performance.
		+ Assets:
		+ main.js (XX KiB)
		+ sec.js (XX KiB)</CLR>
		@@ -15,7 +15,7 @@
		- <CLR=XX,BOLD>ERROR</CLR> in <CLR=XX>×<CLR=XX> entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		- <CLR=XX>│<CLR=XX> Entrypoints:
		- <CLR=XX>│<CLR=XX>   main (XX KiB)
		- <CLR=XX>│<CLR=XX>       main.js
		- <CLR=XX>│<CLR=XX>   sec (XX KiB)
		- <CLR=XX>│<CLR=XX>       sec.js
		- <CLR=XX>
		+ <CLR=XX,BOLD>ERROR</CLR> in <CLR=BOLD>entrypoint size limit: The following entrypoint(s) combined asset size exceeds the recommended limit (XX KiB). This can impact web performance.
		+ Entrypoints:
		+ main (XX KiB)
		+ main.js
		+ sec (XX KiB)
		+ sec.js
		+ </CLR>
		@@ -23,4 +23,3 @@
		- <CLR=XX,BOLD>ERROR</CLR> in <CLR=XX>×<CLR=XX> Rspack performance recommendations:
		- <CLR=XX>│<CLR=XX> You can limit the size of your bundles by using import() to lazy load some parts of your application.
		- <CLR=XX>│<CLR=XX> For more info visit https://www.rspack.dev/guide/optimization/code-splitting
		- <CLR=XX>
		+ <CLR=XX,BOLD>ERROR</CLR> in <CLR=BOLD>webpack performance recommendations:
		+ You can limit the size of your bundles by using import() or require.ensure to lazy load some parts of your application.
		+ For more info visit https://webpack.js.org/guides/code-splitting/</CLR>
		@@ -28,1 +27,1 @@
		- Rspack x.x.x compiled with <CLR=XX,BOLD>XX errors</CLR> in X s
		+ webpack x.x.x compiled with <CLR=XX,BOLD>XX errors</CLR> in X ms"
	`);
	}
};
