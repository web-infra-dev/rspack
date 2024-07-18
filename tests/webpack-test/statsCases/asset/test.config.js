const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
  validate(stats, error, actual) {

    expect(diffStats(actual, path.basename(__dirname)))
      .toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -19,1 +19,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

  }
};
