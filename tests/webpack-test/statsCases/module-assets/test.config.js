const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -3,1 +3,0 @@
		- asset b.js XX bytes [emitted] (name: b)
		@@ -5,0 +4,1 @@
		+ asset b.js XX bytes [emitted] (name: b)
		@@ -6,2 +6,2 @@
		- asset XX.png XX KiB [emitted] [from: node_modules/a/XX.png]
		- asset XX.png XX KiB [emitted] [from: node_modules/a/XX.png]
		+ asset XX.png XX KiB [emitted] [from: node_modules/a/XX.png] (auxiliary name: a)
		+ asset XX.png XX KiB [emitted] [from: node_modules/a/XX.png] (auxiliary name: a, b)
		@@ -9,4 +9,2 @@
		- Chunk Group a XX bytes = a.js
		- Chunk Group b XX bytes = b.js
		- chunk (runtime: main) a.js (a) XX bytes [rendered]
		- ./node_modules/a/index.js XX bytes [built] [code generated]
		+ Chunk Group a XX bytes (XX KiB) = a.js XX bytes (XX.png XX KiB XX.png XX KiB)
		+ Chunk Group b XX bytes (XX KiB) = b.js XX bytes (XX.png XX KiB)
		@@ -14,0 +12,1 @@
		+ ./node_modules/a/XX.png XX bytes [dependent] [built] [code generated] [XX asset]
		@@ -16,0 +15,1 @@
		+ runtime modules XX KiB XX modules
		@@ -17,0 +17,3 @@
		+ chunk (runtime: main) a.js (a) XX bytes [rendered]
		+ ./node_modules/a/XX.png XX bytes [dependent] [built] [code generated] [XX asset]
		+ ./node_modules/a/index.js + XX modules XX bytes [built] [code generated] [XX asset]
		@@ -18,1 +21,1 @@
		- orphan modules XX bytes [orphan] XX modules
		+ orphan modules XX bytes [orphan] XX module
		@@ -20,0 +23,3 @@
		+ modules by path ./node_modules/a/ XX bytes
		+ ./node_modules/a/index.js + XX modules XX bytes [built] [code generated] [XX asset]
		+ ./node_modules/a/XX.png XX bytes [built] [code generated] [XX asset]
		@@ -21,1 +27,0 @@
		- ./node_modules/a/index.js XX bytes [built] [code generated]
		@@ -23,1 +28,1 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
