const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -4,1 +4,1 @@
			- minimal (Rspack x.x.x) compiled successfully in X.XX
			+ minimal (webpack x.x.x) compiled successfully in X ms
			@@ -9,1 +9,1 @@
			- verbose (Rspack x.x.x) compiled successfully
			+ verbose (webpack x.x.x) compiled successfully"
		`);

	}
};
