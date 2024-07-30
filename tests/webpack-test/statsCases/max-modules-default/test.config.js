const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,2 +4,0 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		@@ -7,2 +5,0 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		@@ -10,2 +6,1 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		@@ -13,2 +8,1 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		@@ -16,0 +10,6 @@
		+ ./a.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		@@ -17,1 +17,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
