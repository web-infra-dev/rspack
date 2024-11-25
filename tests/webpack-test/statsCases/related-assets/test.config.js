const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,9 +2,0 @@
		- assets by path *.br XX KiB
		- asset default-main.js.br XX KiB [emitted]
		- asset default-main.js.map.br XX bytes [emitted]
		- + XX assets
		- assets by path *.gz XX KiB
		- asset default-main.js.gz XX KiB [emitted]
		- asset default-main.js.map.gz XX bytes [emitted]
		- asset default-chunk_js.js.gz XX bytes [emitted]
		- + XX assets
		@@ -12,2 +3,2 @@
		- asset default-main.js XX KiB [emitted] (name: main)
		- asset default-chunk_js.js XX bytes [emitted]
		+ asset default-main.js XX KiB [emitted] (name: main) XX related assets
		+ asset default-chunk_js.js XX bytes [emitted] XX related assets
		@@ -15,2 +6,2 @@
		- asset default-chunk_js.css XX bytes [emitted]
		- asset default-main.css XX bytes [emitted] (name: main)
		+ asset default-chunk_js.css XX bytes [emitted] XX related assets
		+ asset default-main.css XX bytes [emitted] (name: main) XX related assets
		@@ -19,9 +10,0 @@
		- assets by path *.br XX KiB
		- asset relatedAssets-main.js.br XX KiB [emitted]
		- asset relatedAssets-main.js.map.br XX bytes [emitted]
		- + XX assets
		- assets by path *.gz XX KiB
		- asset relatedAssets-main.js.gz XX KiB [emitted]
		- asset relatedAssets-main.js.map.gz XX bytes [emitted]
		- asset relatedAssets-chunk_js.js.gz XX bytes [emitted]
		- + XX assets
		@@ -30,0 +12,5 @@
		+ compressed relatedAssets-main.js.br XX KiB [emitted]
		+ compressed relatedAssets-main.js.gz XX KiB [emitted]
		+ sourceMap relatedAssets-main.js.map XX KiB [emitted] [dev] (auxiliary name: main)
		+ compressed relatedAssets-main.js.map.br XX KiB [emitted]
		+ compressed relatedAssets-main.js.map.gz XX KiB [emitted]
		@@ -31,0 +18,5 @@
		+ compressed relatedAssets-chunk_js.js.br XX bytes [emitted]
		+ compressed relatedAssets-chunk_js.js.gz XX bytes [emitted]
		+ sourceMap relatedAssets-chunk_js.js.map XX bytes [emitted] [dev]
		+ compressed relatedAssets-chunk_js.js.map.br XX bytes [emitted]
		+ compressed relatedAssets-chunk_js.js.map.gz XX bytes [emitted]
		@@ -33,0 +25,5 @@
		+ sourceMap relatedAssets-chunk_js.css.map XX bytes [emitted] [dev]
		+ compressed relatedAssets-chunk_js.css.map.br XX bytes [emitted]
		+ compressed relatedAssets-chunk_js.css.map.gz XX bytes [emitted]
		+ compressed relatedAssets-chunk_js.css.br XX bytes [emitted]
		+ compressed relatedAssets-chunk_js.css.gz XX bytes [emitted]
		@@ -34,0 +31,5 @@
		+ sourceMap relatedAssets-main.css.map XX bytes [emitted] [dev] (auxiliary name: main)
		+ compressed relatedAssets-main.css.map.br XX bytes [emitted]
		+ compressed relatedAssets-main.css.map.gz XX bytes [emitted]
		+ compressed relatedAssets-main.css.br XX bytes [emitted]
		+ compressed relatedAssets-main.css.gz XX bytes [emitted]
		@@ -36,2 +38,0 @@
		- hidden assets XX KiB XX assets
		- assets by status XX KiB [emitted]
		@@ -40,0 +40,5 @@
		+ hidden assets XX KiB XX assets
		+ sourceMap excludeXX-main.js.map XX KiB [emitted] [dev] (auxiliary name: main)
		+ hidden assets XX KiB XX assets
		+ + XX related asset
		+ + XX related asset
		@@ -41,0 +46,5 @@
		+ hidden assets XX KiB XX assets
		+ sourceMap excludeXX-chunk_js.js.map XX bytes [emitted] [dev]
		+ hidden assets XX bytes XX assets
		+ + XX related asset
		+ + XX related asset
		@@ -43,0 +53,5 @@
		+ hidden assets XX bytes XX assets
		+ sourceMap excludeXX-chunk_js.css.map XX bytes [emitted] [dev]
		+ hidden assets XX bytes XX assets
		+ + XX related asset
		+ + XX related asset
		@@ -44,0 +59,5 @@
		+ hidden assets XX bytes XX assets
		+ sourceMap excludeXX-main.css.map XX bytes [emitted] [dev] (auxiliary name: main)
		+ hidden assets XX bytes XX assets
		+ + XX related asset
		+ + XX related asset
		@@ -46,9 +66,0 @@
		- assets by path *.br XX KiB
		- asset excludeXX-main.js.br XX KiB [emitted]
		- asset excludeXX-main.js.map.br XX bytes [emitted]
		- + XX assets
		- assets by path *.gz XX KiB
		- asset excludeXX-main.js.gz XX KiB [emitted]
		- asset excludeXX-main.js.map.gz XX bytes [emitted]
		- asset excludeXX-chunk_js.js.gz XX bytes [emitted]
		- + XX assets
		@@ -57,0 +68,3 @@
		+ hidden assets XX KiB XX asset
		+ compressed excludeXX-main.js.br XX KiB [emitted]
		+ compressed excludeXX-main.js.gz XX KiB [emitted]
		@@ -58,0 +72,3 @@
		+ hidden assets XX bytes XX asset
		+ compressed excludeXX-chunk_js.js.br XX bytes [emitted]
		+ compressed excludeXX-chunk_js.js.gz XX bytes [emitted]
		@@ -60,0 +77,3 @@
		+ hidden assets XX bytes XX asset
		+ compressed excludeXX-chunk_js.css.br XX bytes [emitted]
		+ compressed excludeXX-chunk_js.css.gz XX bytes [emitted]
		@@ -61,0 +81,3 @@
		+ hidden assets XX bytes XX asset
		+ compressed excludeXX-main.css.br XX bytes [emitted]
		+ compressed excludeXX-main.css.gz XX bytes [emitted]
		@@ -63,1 +86,1 @@
		- hidden assets XX KiB XX assets
		+ hidden assets XX bytes XX assets
		@@ -65,11 +88,0 @@
		- assets by path *.br XX KiB
		- asset excludeXX-main.js.br XX KiB [emitted]
		- asset excludeXX-main.js.map.br XX bytes [emitted]
		- asset excludeXX-main.css.map.br XX bytes [emitted]
		- asset excludeXX-main.css.br XX bytes [emitted]
		- assets by path *.gz XX KiB
		- asset excludeXX-main.js.gz XX KiB [emitted]
		- asset excludeXX-main.js.map.gz XX bytes [emitted]
		- asset excludeXX-main.css.map.gz XX bytes [emitted]
		- asset excludeXX-main.css.gz XX bytes [emitted]
		- assets by chunk XX KiB (name: main)
		@@ -77,0 +89,5 @@
		+ compressed excludeXX-main.js.br XX KiB [emitted]
		+ compressed excludeXX-main.js.gz XX KiB [emitted]
		+ sourceMap excludeXX-main.js.map XX KiB [emitted] [dev] (auxiliary name: main)
		+ compressed excludeXX-main.js.map.br XX KiB [emitted]
		+ compressed excludeXX-main.js.map.gz XX KiB [emitted]
		@@ -78,0 +95,5 @@
		+ sourceMap excludeXX-main.css.map XX bytes [emitted] [dev] (auxiliary name: main)
		+ compressed excludeXX-main.css.map.br XX bytes [emitted]
		+ compressed excludeXX-main.css.map.gz XX bytes [emitted]
		+ compressed excludeXX-main.css.br XX bytes [emitted]
		+ compressed excludeXX-main.css.gz XX bytes [emitted]"
	`);
	}
};
