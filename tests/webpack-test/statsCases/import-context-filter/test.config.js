const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -6,1 +6,1 @@
			- cacheable modules XX bytes
			+ built modules XX bytes [built]
			@@ -8,3 +8,3 @@
			- ./templates/bar.js XX bytes [built] [code generated]
			- ./templates/baz.js XX bytes [built] [code generated]
			- ./templates/foo.js XX bytes [built] [code generated]
			+ ./templates/bar.js XX bytes [optional] [built] [code generated]
			+ ./templates/baz.js XX bytes [optional] [built] [code generated]
			+ ./templates/foo.js XX bytes [optional] [built] [code generated]
			@@ -12,2 +12,2 @@
			- Xdir/import-context-filter/templates|lazy|/^\\\\.\\\\/.*$/|include: /\\\\.js$/|exclude: /\\\\.noimport\\\\.js$/|groupOptions: {}|namespace object XX bytes [built] [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ ./templates/ lazy ^\\\\\\\\.\\\\\\\\/.*$ include: \\\\\\\\.js$ exclude: \\\\\\\\.noimport\\\\\\\\.js$ na...(truncated) XX bytes [optional] [built] [code generated]
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
