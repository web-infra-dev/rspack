const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -4,1 +4,1 @@
			- Rspack x.x.x compiled successfully
			+ webpack x.x.x compiled successfully"
		`);
	}

};
