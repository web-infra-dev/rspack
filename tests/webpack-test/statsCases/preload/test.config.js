const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -9,1 +9,1 @@
			- preload: preloaded.js {XX} (name: preloaded)
			+ preload: preloadedXX.js (name: preloadedXX), preloaded.js (name: preloaded), preloadedXX.js (name: preloadedXX)
			@@ -12,1 +12,0 @@
			- chunk (runtime: main) preloadedXX.js (preloadedXX) XX bytes [rendered]
			@@ -14,1 +13,0 @@
			- chunk (runtime: main) normal.js (normal) XX bytes [rendered]
			@@ -16,1 +14,3 @@
			- chunk (runtime: main) main.js (main) XX bytes (javascript) XX KiB (runtime) (preload: {XX}) [entry] [rendered]
			+ chunk (runtime: main) normal.js (normal) XX bytes [rendered]
			+ chunk (runtime: main) preloadedXX.js (preloadedXX) XX bytes [rendered]
			+ chunk (runtime: main) main.js (main) XX bytes (javascript) XX KiB (runtime) (preload: {XX} {XX} {XX}) [entry] [rendered]"
		`);

	}
};
