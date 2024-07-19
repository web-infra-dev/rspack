const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -10,3 +10,0 @@
		- chunk (runtime: eXX, eXX) XX.js XX bytes [rendered]
		- ./asyncXX.js XX bytes [built] [code generated]
		- ./h.js XX bytes [dependent] [built] [code generated]
		@@ -15,4 +12,6 @@
		- chunk (runtime: eXX) XX.js XX bytes [rendered]
		- ./asyncXX.js XX bytes [built] [code generated]
		- chunk (runtime: eXX) XX.js XX bytes [rendered]
		- ./asyncXX.js XX bytes [built] [code generated]
		+ chunk (runtime: eXX) eXX.js (eXX) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		+ ./b.js XX bytes [dependent] [built] [code generated]
		+ ./eXX.js + XX modules XX bytes [built] [code generated]
		+ ./f.js XX bytes [dependent] [built] [code generated]
		@@ -20,0 +19,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -22,1 +23,6 @@
		- ./eXX.js + XX modules XX bytes [code generated]
		+ ./eXX.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: eXX) XX.js XX bytes [rendered]
		+ ./asyncXX.js XX bytes [built] [code generated]
		+ chunk (runtime: eXX, eXX) XX.js XX bytes [rendered]
		+ ./asyncXX.js XX bytes [built] [code generated]
		+ ./f.js XX bytes [dependent] [built] [code generated]
		@@ -26,4 +32,0 @@
		- chunk (runtime: eXX) eXX.js (eXX) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		- ./b.js XX bytes [dependent] [built] [code generated]
		- ./eXX.js + XX modules XX bytes [code generated]
		- ./h.js XX bytes [dependent] [built] [code generated]
		@@ -32,1 +34,3 @@
		- ./f.js XX bytes [dependent] [built] [code generated]
		+ ./h.js XX bytes [dependent] [built] [code generated]
		+ chunk (runtime: eXX) XX.js XX bytes [rendered]
		+ ./asyncXX.js XX bytes [built] [code generated]
		@@ -34,0 +38,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -35,3 +41,3 @@
		- ./eXX.js + XX modules XX bytes [code generated]
		- ./f.js XX bytes [dependent] [built] [code generated]
		- Rspack x.x.x compiled successfully
		+ ./eXX.js + XX modules XX bytes [built] [code generated]
		+ ./h.js XX bytes [dependent] [built] [code generated]
		+ webpack x.x.x compiled successfully"
	`);
	}
};
