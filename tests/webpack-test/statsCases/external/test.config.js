const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,1 +1,1 @@
			- asset main.js XX bytes [emitted] (name: main)
			+ asset main.js XX KiB [emitted] (name: main)
			@@ -3,2 +3,2 @@
			- external test XX bytes [built] [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ external \\\\test\\\\ XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
