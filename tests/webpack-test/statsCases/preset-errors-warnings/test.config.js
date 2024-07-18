const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -8,1 +8,1 @@
			- Rspack compiled successfully
			+ webpack compiled successfully"
		`);

	}
};
