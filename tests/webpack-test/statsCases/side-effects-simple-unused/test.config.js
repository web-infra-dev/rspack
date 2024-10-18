const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,1 +2,17 @@
		- modules by path ./node_modules/pmodule/*.js XX bytes
		+ ./index.js + XX modules XX bytes [built] [code generated]
		+ [no exports used]
		+ entry ./index main
		+ | ./index.js XX bytes [built]
		+ |   [no exports used]
		+ | ./node_modules/pmodule/index.js XX bytes [built]
		+ |   [only some exports used: default]
		+ |   [inactive] ESM side effect evaluation pmodule ./index.js XX:XX-XX
		+ |   ESM import specifier pmodule ./index.js XX:XX-XX
		+ |   [inactive] ESM import specifier pmodule ./index.js XX:XX-XX
		+ | ./node_modules/pmodule/c.js XX bytes [built]
		+ |   [only some exports used: z]
		+ |   [inactive] from origin ./node_modules/pmodule/b.js
		+ |     [inactive] ESM side effect evaluation ./c ./node_modules/pmodule/b.js XX:XX-XX
		+ |     [inactive] ESM export imported specifier ./c ./node_modules/pmodule/b.js XX:XX-XX
		+ |   ESM import specifier pmodule ./index.js XX:XX-XX (skipped side-effect-free modules)
		+ |   [inactive] ESM export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX (skipped side-effect-free modules)
		@@ -5,3 +21,3 @@
		- esm import pmodule ./index.js
		- esm import specifier pmodule ./index.js
		- esm import specifier pmodule ./index.js
		+ [inactive] ESM side effect evaluation pmodule ./index.js XX:XX-XX
		+ ESM import specifier pmodule ./index.js XX:XX-XX
		+ [inactive] ESM import specifier pmodule ./index.js XX:XX-XX
		@@ -10,3 +26,5 @@
		- esm import specifier pmodule ./index.js
		- esm export ./c ./node_modules/pmodule/b.js
		- esm export import specifier ./c ./node_modules/pmodule/b.js
		+ [inactive] from origin ./node_modules/pmodule/b.js
		+ [inactive] ESM side effect evaluation ./c ./node_modules/pmodule/b.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./c ./node_modules/pmodule/b.js XX:XX-XX
		+ ESM import specifier pmodule ./index.js XX:XX-XX (skipped side-effect-free modules)
		+ [inactive] ESM export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX (skipped side-effect-free modules)
		@@ -15,4 +33,6 @@
		- esm export ./a ./node_modules/pmodule/index.js
		- esm export import specifier ./a ./node_modules/pmodule/index.js
		- esm export ./a ./node_modules/pmodule/index.js
		- esm export import specifier ./a ./node_modules/pmodule/index.js
		+ [inactive] from origin ./index.js + XX modules
		+ [inactive] ESM side effect evaluation ./a ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./a ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] from origin ./node_modules/pmodule/index.js
		+ [inactive] ESM side effect evaluation ./a ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./a ./node_modules/pmodule/index.js XX:XX-XX
		@@ -21,27 +41,11 @@
		- esm export ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- esm export ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- esm export import specifier ./b ./node_modules/pmodule/index.js
		- modules by path ./*.js XX bytes
		- ./index.js XX bytes [orphan] [built]
		- [no exports used]
		- ./index.js + XX modules XX bytes [code generated]
		- [no exports used]
		- entry ./index
		- | ./index.js XX bytes [orphan] [built]
		- |   [no exports used]
		- | ./node_modules/pmodule/index.js XX bytes [orphan] [built]
		- |   [only some exports used: default]
		- |   esm import pmodule ./index.js
		- |   esm import specifier pmodule ./index.js
		- |   esm import specifier pmodule ./index.js
		- | ./node_modules/pmodule/c.js XX bytes [orphan] [built]
		- |   [only some exports used: z]
		- |   esm import specifier pmodule ./index.js
		- |   esm export ./c ./node_modules/pmodule/b.js
		- |   esm export import specifier ./c ./node_modules/pmodule/b.js
		- Rspack x.x.x compiled successfully in X.XX
		+ [inactive] from origin ./index.js + XX modules
		+ [inactive] ESM side effect evaluation ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] from origin ./node_modules/pmodule/index.js
		+ [inactive] ESM side effect evaluation ./b ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
