const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,6 +2,0 @@
			- chunk (runtime: main) cXX.js (cXX) XX bytes <{XX}> [rendered]
			- chunk (runtime: main) cXX.js (cXX) XX bytes <{XX}> [rendered]
			- chunk (runtime: main) bXX.js (bXX) XX bytes <{XX}> [rendered]
			- chunk (runtime: main) bXX.js (bXX) XX bytes <{XX}> [rendered]
			- chunk (runtime: main) a.js (a) XX bytes <{XX}> >{XX}< >{XX}< (prefetch: {XX} {XX}) [rendered]
			- chunk (runtime: main) aXX.js (aXX) XX bytes <{XX}> [rendered]
			@@ -9,0 +3,2 @@
			+ chunk (runtime: main) aXX.js (aXX) XX bytes <{XX}> [rendered]
			+ chunk (runtime: main) bXX.js (bXX) XX bytes <{XX}> [rendered]
			@@ -10,0 +6,1 @@
			+ chunk (runtime: main) bXX.js (bXX) XX bytes <{XX}> [rendered]
			@@ -12,0 +9,3 @@
			+ chunk (runtime: main) cXX.js (cXX) XX bytes <{XX}> [rendered]
			+ chunk (runtime: main) cXX.js (cXX) XX bytes <{XX}> [rendered]
			+ chunk (runtime: main) a.js (a) XX bytes <{XX}> >{XX}< >{XX}< (prefetch: {XX} {XX}) [rendered]"
		`);

	}
};
