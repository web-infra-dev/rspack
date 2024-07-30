const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,2 +4,2 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		@@ -7,2 +7,0 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		@@ -10,2 +8,0 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		@@ -13,2 +9,0 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		@@ -16,2 +10,1 @@
		- ./c.js?XX XX bytes [built] [code generated]
		- ./b.js?XX XX bytes [built] [code generated]
		+ ./a.js?XX XX bytes [built] [code generated]
		@@ -19,0 +12,5 @@
		+ ./a.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		@@ -20,1 +18,3 @@
		- ./b.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		+ ./c.js?XX XX bytes [built] [code generated]
		@@ -22,1 +22,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
