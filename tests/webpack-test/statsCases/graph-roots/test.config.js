const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -7,3 +7,1 @@
			- dependent modules XX bytes [dependent] XX module
			- ./cycleXX/a.js XX bytes [built] [code generated]
			- ./cycleXX/b.js XX bytes [built] [code generated]
			+ dependent modules XX bytes [dependent] XX modules
			@@ -13,1 +11,0 @@
			- ./cycles/XX/a.js XX bytes [built] [code generated]
			@@ -15,2 +12,0 @@
			- ./cycles/XX/a.js XX bytes [built] [code generated]
			- ./cycles/XX/b.js XX bytes [built] [code generated]
			@@ -27,0 +22,1 @@
			+ runtime modules XX KiB XX modules"
		`);

	}
};
