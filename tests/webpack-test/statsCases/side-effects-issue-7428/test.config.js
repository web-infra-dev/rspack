const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -11,5 +11,7 @@
		- esm import ./components ./foo.js
		- esm import specifier ./components ./foo.js
		- esm import specifier ./components ./main.js
		- esm import specifier ./components ./main.js
		- esm import ./components ./main.js
		+ [inactive] from origin ./main.js + XX modules
		+ [inactive] ESM side effect evaluation ./components ./main.js + XX modules ./main.js XX:XX-XX
		+ [inactive] ESM import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX
		+ [inactive] ESM import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX
		+ [inactive] from origin ./foo.js
		+ [inactive] ESM side effect evaluation ./components ./foo.js XX:XX-XX
		+ [inactive] ESM import specifier ./components ./foo.js XX:XX-XX
		@@ -19,5 +21,6 @@
		- esm export ./CompA ./components/src/CompAB/index.js
		- esm export import specifier ./CompA ./components/src/CompAB/index.js
		- esm export import specifier ./CompAB ./components/src/index.js
		- esm import specifier ./components ./foo.js
		- esm import specifier ./components ./main.js
		+ [inactive] from origin ./components/src/CompAB/index.js
		+ [inactive] ESM side effect evaluation ./CompA ./components/src/CompAB/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./CompA ./components/src/CompAB/index.js XX:XX-XX
		+ [inactive] ESM export imported specifier ./CompAB ./components/src/index.js XX:XX-XX (skipped side-effect-free modules)
		+ ESM import specifier ./components ./foo.js XX:XX-XX (skipped side-effect-free modules)
		+ ESM import specifier ./components ./main.js + XX modules ./main.js XX:XX-XX (skipped side-effect-free modules)
		@@ -25,6 +28,9 @@
		- esm import ./utils ./components/src/CompAB/CompA.js
		- esm import specifier ./utils ./components/src/CompAB/CompA.js
		- esm import ./utils ./components/src/CompAB/CompB.js
		- esm import specifier ./utils ./components/src/CompAB/CompB.js
		- esm import ./utils ./components/src/CompAB/CompB.js
		- esm import specifier ./utils ./components/src/CompAB/CompB.js
		+ from origin ./components/src/CompAB/CompA.js
		+ [inactive] ESM side effect evaluation ./utils ./components/src/CompAB/CompA.js XX:XX-XX
		+ ESM import specifier ./utils ./components/src/CompAB/CompA.js XX:XX-XX
		+ from origin ./components/src/CompAB/CompB.js
		+ [inactive] ESM side effect evaluation ./utils ./components/src/CompAB/CompB.js XX:XX-XX
		+ ESM import specifier ./utils ./components/src/CompAB/CompB.js XX:XX-XX
		+ from origin ./main.js + XX modules
		+ [inactive] ESM side effect evaluation ./utils ./main.js + XX modules ./components/src/CompAB/CompB.js XX:XX-XX
		+ ESM import specifier ./utils ./main.js + XX modules ./components/src/CompAB/CompB.js XX:XX-XX
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
		- |   esm export ./CompB ./components/src/CompAB/index.js
		- |   esm export import specifier ./CompB ./components/src/CompAB/index.js
		- |   esm export import specifier ./CompAB ./components/src/index.js
		- |   esm import specifier ./components ./main.js
		+ |   [inactive] from origin ./components/src/CompAB/index.js
		+ |     [inactive] ESM side effect evaluation ./CompB ./components/src/CompAB/index.js XX:XX-XX
		+ |     [inactive] ESM export imported specifier ./CompB ./components/src/CompAB/index.js XX:XX-XX
		+ |   [inactive] ESM export imported specifier ./CompAB ./components/src/index.js XX:XX-XX (skipped side-effect-free modules)
		+ |   ESM import specifier ./components ./main.js XX:XX-XX (skipped side-effect-free modules)
		@@ -46,2 +51,2 @@
		- import() ./foo ./main.js
		- Rspack x.x.x compiled successfully in X.XX
		+ import() ./foo ./main.js + XX modules ./main.js XX:XX-XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
