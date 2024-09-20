const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,0 +2,1 @@
		+ runtime modules XX KiB XX modules
		@@ -3,5 +4,1 @@
		- entry ./eXX
		- chunk (runtime: eXX, eXX) a.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		- ./module-a.js XX bytes [built] [code generated]
		- import() ./module-a ./eXX.js
		- import() ./module-a ./module-c.js
		+ entry ./eXX eXX
		@@ -10,5 +7,1 @@
		- import() ./module-b ./module-a.js
		- chunk (runtime: eXX, eXX) c.js (c) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		- ./module-c.js XX bytes [built] [code generated]
		- import() ./module-c ./eXX.js
		- import() ./module-c ./module-b.js
		+ import() ./module-b ./module-a.js XX:XX-XX
		@@ -16,0 +9,1 @@
		+ runtime modules XX KiB XX modules
		@@ -17,2 +11,10 @@
		- entry ./eXX
		- Rspack x.x.x compiled successfully
		+ entry ./eXX eXX
		+ chunk (runtime: eXX, eXX) c.js (c) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		+ ./module-c.js XX bytes [built] [code generated]
		+ import() ./module-c ./eXX.js XX:XX-XX
		+ import() ./module-c ./module-b.js XX:XX-XX
		+ chunk (runtime: eXX, eXX) a.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
		+ ./module-a.js XX bytes [built] [code generated]
		+ import() ./module-a ./eXX.js XX:XX-XX
		+ import() ./module-a ./module-c.js XX:XX-XX
		+ webpack x.x.x compiled successfully"
	`);
	}
};
