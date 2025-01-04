const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -9,1 +9,2 @@
			- prefetch: prefetched.js {XX} (name: prefetched)
			+ prefetch: prefetchedXX.js {XX} (name: prefetchedXX), prefetched.js {XX} (name: prefetched), prefetchedXX.js {XX} (name: prefetchedXX)
			+ chunk {XX} (runtime: main) inner.js (inner) XX bytes <{XX}> [rendered]
			@@ -12,2 +13,0 @@
			- chunk {XX} (runtime: main) inner.js (inner) XX bytes <{XX}> [rendered]
			- chunk {XX} (runtime: main) normal.js (normal) XX bytes <{XX}> [rendered]
			@@ -16,1 +15,2 @@
			- chunk {XX} (runtime: main) main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< (prefetch: {XX}) [entry] [rendered]
			+ chunk {XX} (runtime: main) normal.js (normal) XX bytes <{XX}> [rendered]
			+ chunk {XX} (runtime: main) main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< (prefetch: {XX} {XX} {XX}) [entry] [rendered]"
		`);

	}
};
