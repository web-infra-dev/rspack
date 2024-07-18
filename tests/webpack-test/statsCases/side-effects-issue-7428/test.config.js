const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -11,5 +11,7 @@
			- esm import ./components
			- esm import specifier ./components
			- esm import specifier ./components
			- esm import specifier ./components
			- esm import ./components
			+ [inactive] from origin ./main.js + XX modules
			+ [inactive] harmony side effect evaluation ./components ./main.js + XX modules ./main.js XX:XX-XX
			+ [inactive] harmony import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX
			+ [inactive] harmony import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX
			+ [inactive] from origin ./foo.js
			+ [inactive] harmony side effect evaluation ./components ./foo.js XX:XX-XX
			+ [inactive] harmony import specifier ./components ./foo.js XX:XX-XX
			@@ -19,5 +21,6 @@
			- esm export ./CompA
			- esm export import specifier ./CompA
			- esm export import specifier ./CompAB
			- esm import specifier ./components
			- esm import specifier ./components
			+ [inactive] from origin ./components/src/CompAB/index.js
			+ [inactive] harmony side effect evaluation ./CompA ./components/src/CompAB/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./CompA ./components/src/CompAB/index.js XX:XX-XX
			+ [inactive] harmony export imported specifier ./CompAB ./components/src/index.js XX:XX-XX (skipped side-effect-free modules)
			+ harmony import specifier ./components ./foo.js XX:XX-XX (skipped side-effect-free modules)
			+ harmony import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX (skipped side-effect-free modules)
			@@ -25,6 +28,9 @@
			- esm import ./utils
			- esm import specifier ./utils
			- esm import ./utils
			- esm import specifier ./utils
			- esm import ./utils
			- esm import specifier ./utils
			+ from origin ./components/src/CompAB/CompA.js
			+ [inactive] harmony side effect evaluation ./utils ./components/src/CompAB/CompA.js XX:XX-XX
			+ harmony import specifier ./utils ./components/src/CompAB/CompA.js XX:XX-XX
			+ from origin ./components/src/CompAB/CompB.js
			+ [inactive] harmony side effect evaluation ./utils ./components/src/CompAB/CompB.js XX:XX-XX
			+ harmony import specifier ./utils ./components/src/CompAB/CompB.js XX:XX-XX
			+ from origin ./main.js + XX modules
			+ [inactive] harmony side effect evaluation ./utils ./main.js + XX modules ./components/src/CompAB/CompB.js XX:XX-XX
			+ harmony import specifier ./utils ./main.js + XX modules ./components/src/CompAB/CompB.js XX:XX-XX
			@@ -32,3 +38,1 @@
			- ./main.js XX bytes [orphan] [built]
			- [no exports used]
			- ./main.js + XX modules XX bytes [code generated]
			+ ./main.js + XX modules XX bytes [built] [code generated]
			@@ -36,2 +40,2 @@
			- entry ./main.js
			- | ./main.js XX bytes [orphan] [built]
			+ entry ./main.js main
			+ | ./main.js XX bytes [built]
			@@ -39,1 +43,1 @@
			- | ./components/src/CompAB/CompB.js XX bytes [orphan] [built]
			+ | ./components/src/CompAB/CompB.js XX bytes [built]
			@@ -41,4 +45,5 @@
			- |   esm export ./CompB
			- |   esm export import specifier ./CompB
			- |   esm export import specifier ./CompAB
			- |   esm import specifier ./components
			+ |   [inactive] from origin ./components/src/CompAB/index.js
			+ |     [inactive] harmony side effect evaluation ./CompB ./components/src/CompAB/index.js XX:XX-XX
			+ |     [inactive] harmony export imported specifier ./CompB ./components/src/CompAB/index.js XX:XX-XX
			+ |   [inactive] harmony export imported specifier ./CompAB ./components/src/index.js XX:XX-XX (skipped side-effect-free modules)
			+ |   harmony import specifier ./components ./main.js XX:XX-XX (skipped side-effect-free modules)
			@@ -46,2 +51,2 @@
			- import() ./foo
			- Rspack x.x.x compiled successfully in X.XX
			+ import() ./foo ./main.js + XX modules ./main.js XX:XX-XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
