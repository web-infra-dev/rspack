const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
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
			+ |   [inactive] harmony side effect evaluation pmodule ./index.js XX:XX-XX
			+ |   harmony import specifier pmodule ./index.js XX:XX-XX
			+ |   [inactive] harmony import specifier pmodule ./index.js XX:XX-XX
			+ | ./node_modules/pmodule/c.js XX bytes [built]
			+ |   [only some exports used: z]
			+ |   [inactive] from origin ./node_modules/pmodule/b.js
			+ |     [inactive] harmony side effect evaluation ./c ./node_modules/pmodule/b.js XX:XX-XX
			+ |     [inactive] harmony export imported specifier ./c ./node_modules/pmodule/b.js XX:XX-XX
			+ |   harmony import specifier pmodule ./index.js XX:XX-XX (skipped side-effect-free modules)
			+ |   [inactive] harmony export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX (skipped side-effect-free modules)
			@@ -5,3 +21,3 @@
			- esm import pmodule
			- esm import specifier pmodule
			- esm import specifier pmodule
			+ [inactive] harmony side effect evaluation pmodule ./index.js XX:XX-XX
			+ harmony import specifier pmodule ./index.js XX:XX-XX
			+ [inactive] harmony import specifier pmodule ./index.js XX:XX-XX
			@@ -10,3 +26,5 @@
			- esm import specifier pmodule
			- esm export ./c
			- esm export import specifier ./c
			+ [inactive] from origin ./node_modules/pmodule/b.js
			+ [inactive] harmony side effect evaluation ./c ./node_modules/pmodule/b.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./c ./node_modules/pmodule/b.js XX:XX-XX
			+ harmony import specifier pmodule ./index.js XX:XX-XX (skipped side-effect-free modules)
			+ [inactive] harmony export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX (skipped side-effect-free modules)
			@@ -15,4 +33,6 @@
			- esm export ./a
			- esm export import specifier ./a
			- esm export ./a
			- esm export import specifier ./a
			+ [inactive] from origin ./index.js + XX modules
			+ [inactive] harmony side effect evaluation ./a ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./a ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] from origin ./node_modules/pmodule/index.js
			+ [inactive] harmony side effect evaluation ./a ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./a ./node_modules/pmodule/index.js XX:XX-XX
			@@ -21,27 +41,11 @@
			- esm export ./b
			- esm export import specifier ./b
			- esm export import specifier ./b
			- esm export import specifier ./b
			- esm export ./b
			- esm export import specifier ./b
			- esm export import specifier ./b
			- esm export import specifier ./b
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
			- |   esm import pmodule
			- |   esm import specifier pmodule
			- |   esm import specifier pmodule
			- | ./node_modules/pmodule/c.js XX bytes [orphan] [built]
			- |   [only some exports used: z]
			- |   esm import specifier pmodule
			- |   esm export ./c
			- |   esm export import specifier ./c
			- Rspack x.x.x compiled successfully in X.XX
			+ [inactive] from origin ./index.js + XX modules
			+ [inactive] harmony side effect evaluation ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./index.js + XX modules ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] from origin ./node_modules/pmodule/index.js
			+ [inactive] harmony side effect evaluation ./b ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./b ./node_modules/pmodule/index.js XX:XX-XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
