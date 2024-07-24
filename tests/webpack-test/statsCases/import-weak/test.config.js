const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -3,1 +3,0 @@
			- asset XX.js XX bytes [emitted]
			@@ -5,0 +4,1 @@
			+ orphan modules XX bytes [orphan] XX module
			@@ -7,1 +7,0 @@
			- ./modules/a.js XX bytes [built] [code generated]
			@@ -9,1 +8,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
