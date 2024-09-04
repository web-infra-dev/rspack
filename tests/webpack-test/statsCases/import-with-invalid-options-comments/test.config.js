const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -10,10 +10,3 @@
		- WARNING in ./chunk.js
		- ⚠ Module parse warning:
		- ╰─▶   ⚠ Magic comments parse failed: \`webpackChunkName\` expected a string, but received: notGoingToCompileChunkName .
		- ╭─[XX:XX]
		- XX │ export default function() {
		- XX │     import(/* webpackPrefetch: true, webpackChunkName: notGoingToCompileChunkName */ ./chunk-a);
		- ·                                                        ───────────────────────────
		- XX │     import(/* webpackPrefetch: XX, webpackChunkName: goingToCompileChunkName-b */ ./chunk-b);
		- XX │     import(/* webpack Prefetch: XX, webpackChunkName: notGoingToCompile-c */ ./chunk-c);
		- ╰────
		+ WARNING in ./chunk.js XX:XX-XX
		+ Compilation error while processing magic comment(-s): /* webpackPrefetch: true, webpackChunkName: notGoingToCompileChunkName */: notGoingToCompileChunkName is not defined
		+ @ ./index.js XX:XX-XX
		@@ -21,12 +14,3 @@
		- @ ./index.js
		-
		- WARNING in ./chunk.js
		- ⚠ Module parse warning:
		- ╰─▶   ⚠ Magic comments parse failed: \`webpackPrefetch\` expected true or a number, but received: nope .
		- ╭─[XX:XX]
		- XX │     import(/* webpackPrefetch: XX, webpackChunkName: goingToCompileChunkName-b */ ./chunk-b);
		- XX │     import(/* webpack Prefetch: XX, webpackChunkName: notGoingToCompile-c */ ./chunk-c);
		- XX │     import(/* webpackPrefetch: nope */ /* webpackChunkName: yep */ ./chunk-d);
		- ·                                ─────
		- XX │ }
		- ╰────
		+ WARNING in ./chunk.js XX:XX-XX
		+ Compilation error while processing magic comment(-s): /* webpack Prefetch: XX, webpackChunkName: \\\\notGoingToCompile-c\\\\ */: Unexpected identifier
		+ @ ./index.js XX:XX-XX
		@@ -34,1 +18,3 @@
		- @ ./index.js
		+ WARNING in ./chunk.js XX:XX-XX
		+ Compilation error while processing magic comment(-s): /* webpackPrefetch: nope */: nope is not defined
		+ @ ./index.js XX:XX-XX
		@@ -36,1 +22,1 @@
		- Rspack x.x.x compiled with XX warnings
		+ webpack x.x.x compiled with XX warnings"
	`);
	}
};
