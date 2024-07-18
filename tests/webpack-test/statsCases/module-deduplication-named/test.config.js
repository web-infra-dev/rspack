const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -11,0 +11,2 @@
			+ runtime modules XX KiB XX modules
			+ cacheable modules XX bytes
			@@ -12,4 +14,1 @@
			- ./d.js XX bytes [dependent] [built] [code generated]
			- ./eXX.js + XX modules XX bytes [code generated]
			- chunk (runtime: eXX, eXX, eXX) asyncXX.js (asyncXX) XX bytes [rendered]
			- ./asyncXX.js XX bytes [built] [code generated]
			+ ./eXX.js + XX modules XX bytes [built] [code generated]
			@@ -21,0 +20,2 @@
			+ runtime modules XX KiB XX modules
			+ cacheable modules XX bytes
			@@ -22,2 +23,5 @@
			- ./eXX.js + XX modules XX bytes [code generated]
			- ./h.js XX bytes [dependent] [built] [code generated]
			+ ./d.js XX bytes [dependent] [built] [code generated]
			+ ./eXX.js + XX modules XX bytes [built] [code generated]
			+ chunk (runtime: eXX, eXX, eXX) asyncXX.js (asyncXX) XX bytes [rendered]
			+ ./asyncXX.js XX bytes [built] [code generated]
			+ ./f.js XX bytes [dependent] [built] [code generated]
			@@ -25,0 +29,2 @@
			+ runtime modules XX KiB XX modules
			+ cacheable modules XX bytes
			@@ -26,3 +32,3 @@
			- ./eXX.js + XX modules XX bytes [code generated]
			- ./f.js XX bytes [dependent] [built] [code generated]
			- Rspack x.x.x compiled successfully
			+ ./eXX.js + XX modules XX bytes [built] [code generated]
			+ ./h.js XX bytes [dependent] [built] [code generated]
			+ webpack x.x.x compiled successfully"
		`);

	}
};
