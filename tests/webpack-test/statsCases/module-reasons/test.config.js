const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,2 +4,2 @@
		- ./index.js + XX modules XX bytes [code generated]
		- entry ./index
		+ ./index.js + XX modules XX bytes [built] [code generated]
		+ entry ./index main
		@@ -7,3 +7,3 @@
		- cjs require ./c ./a.js
		- cjs require ./c ./b.js
		- Rspack x.x.x compiled successfully in X.XX
		+ cjs require ./c ./index.js + XX modules ./a.js XX:XX-XX
		+ cjs require ./c ./index.js + XX modules ./b.js XX:XX-XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
