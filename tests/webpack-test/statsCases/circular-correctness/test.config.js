const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,2 +1,0 @@
			- chunk (runtime: main) XX.bundle.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
			- ./module-a.js XX bytes [built] [code generated]
			@@ -8,0 +6,1 @@
			+ runtime modules XX KiB XX modules
			@@ -9,1 +8,3 @@
			- Rspack x.x.x compiled successfully
			+ chunk (runtime: main) XX.bundle.js (a) XX bytes <{XX}> <{XX}> >{XX}< [rendered]
			+ ./module-a.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully"
		`);

	}
};
