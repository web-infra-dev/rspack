const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,0 +2,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -3,1 +5,1 @@
		- entry ./eXX
		+ entry ./eXX eXX
		@@ -5,7 +7,3 @@
		- esm import ./module-x ./eXX.js
		- esm import ./module-x ./eXX.js
		- import() ./module-x ./module-b.js
		- chunk (runtime: eXX, eXX) a.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		- ./module-a.js XX bytes [built] [code generated]
		- import() ./module-a ./eXX.js
		- import() ./module-a ./module-c.js
		+ ESM side effect evaluation ./module-x ./eXX.js XX:XX-XX
		+ ESM side effect evaluation ./module-x ./eXX.js XX:XX-XX
		+ import() ./module-x ./module-b.js XX:XX-XX
		@@ -14,5 +12,1 @@
		- import() ./module-b ./module-a.js
		- chunk (runtime: eXX, eXX) c.js (c) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		- ./module-c.js XX bytes [built] [code generated]
		- import() ./module-c ./eXX.js
		- import() ./module-c ./module-b.js
		+ import() ./module-b ./module-a.js XX:XX-XX
		@@ -20,0 +14,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -21,1 +17,1 @@
		- entry ./eXX
		+ entry ./eXX eXX
		@@ -23,3 +19,7 @@
		- esm import ./module-x ./eXX.js
		- esm import ./module-x ./eXX.js
		- import() ./module-x ./module-b.js
		+ ESM side effect evaluation ./module-x ./eXX.js XX:XX-XX
		+ ESM side effect evaluation ./module-x ./eXX.js XX:XX-XX
		+ import() ./module-x ./module-b.js XX:XX-XX
		+ chunk (runtime: eXX, eXX) c.js (c) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		+ ./module-c.js XX bytes [built] [code generated]
		+ import() ./module-c ./eXX.js XX:XX-XX
		+ import() ./module-c ./module-b.js XX:XX-XX
		@@ -28,2 +28,6 @@
		- import() ./module-y ./module-x.js
		- Rspack x.x.x compiled successfully
		+ import() ./module-y ./module-x.js XX:XX-XX
		+ chunk (runtime: eXX, eXX) a.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		+ ./module-a.js XX bytes [built] [code generated]
		+ import() ./module-a ./eXX.js XX:XX-XX
		+ import() ./module-a ./module-c.js XX:XX-XX
		+ webpack x.x.x compiled successfully"
	`);
	}
};
