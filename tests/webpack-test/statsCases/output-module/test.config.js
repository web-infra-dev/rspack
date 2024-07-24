const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -4,1 +4,1 @@
			- orphan modules XX bytes [orphan] XX modules
			+ orphan modules XX bytes [orphan] XX module
			@@ -6,1 +6,1 @@
			- ./index.js + XX modules XX bytes [code generated]
			+ ./index.js + XX modules XX bytes [built] [code generated]
			@@ -8,1 +8,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
