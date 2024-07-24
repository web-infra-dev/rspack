const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -4,1 +4,0 @@
			- asset used-exports-a.js XX KiB [emitted] (name: a)
			@@ -6,1 +5,2 @@
			- Entrypoint a XX KiB = used-exports-XX.js XX bytes used-exports-a.js XX KiB
			+ asset used-exports-a.js XX bytes [emitted] (name: a)
			+ Entrypoint a XX bytes = used-exports-a.js
			@@ -9,4 +9,0 @@
			- chunk (runtime: a, b, c) used-exports-XX.js (id hint: ) XX bytes [initial] [rendered] split chunk (cache group: default)
			- ./objects.js XX bytes [built] [code generated]
			- chunk (runtime: a) used-exports-a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			- ./a.js XX bytes [built] [code generated]
			@@ -14,0 +10,1 @@
			+ runtime modules XX KiB XX modules
			@@ -16,0 +13,1 @@
			+ runtime modules XX KiB XX modules
			@@ -17,1 +15,5 @@
			- used-exports (Rspack x.x.x) compiled successfully in X.XX
			+ chunk (runtime: b, c) used-exports-XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
			+ ./objects.js XX bytes [built] [code generated]
			+ chunk (runtime: a) used-exports-a.js (a) XX bytes [entry] [rendered]
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ used-exports (webpack x.x.x) compiled successfully in X ms
			@@ -21,1 +23,0 @@
			- asset no-used-exports-b.js XX KiB [emitted] (name: b)
			@@ -23,0 +24,1 @@
			+ asset no-used-exports-b.js XX KiB [emitted] (name: b)
			@@ -27,4 +29,0 @@
			- chunk (runtime: a, b, c) no-used-exports-XX.js (id hint: ) XX bytes [initial] [rendered] split chunk (cache group: default)
			- ./objects.js XX bytes [built] [code generated]
			- chunk (runtime: a) no-used-exports-a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			- ./a.js XX bytes [built] [code generated]
			@@ -32,0 +30,1 @@
			+ runtime modules XX KiB XX modules
			@@ -34,0 +33,1 @@
			+ runtime modules XX KiB XX modules
			@@ -35,1 +35,6 @@
			- no-used-exports (Rspack x.x.x) compiled successfully in X.XX
			+ chunk (runtime: a, b, c) no-used-exports-XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
			+ ./objects.js XX bytes [built] [code generated]
			+ chunk (runtime: a) no-used-exports-a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ runtime modules XX KiB XX modules
			+ ./a.js XX bytes [built] [code generated]
			+ no-used-exports (webpack x.x.x) compiled successfully in X ms
			@@ -39,0 +44,1 @@
			+ asset global-a.js XX KiB [emitted] (name: a)
			@@ -40,1 +46,0 @@
			- asset global-a.js XX KiB [emitted] (name: a)
			@@ -45,4 +50,0 @@
			- chunk (runtime: a, b, c) global-XX.js (id hint: ) XX bytes [initial] [rendered] split chunk (cache group: default)
			- ./objects.js XX bytes [built] [code generated]
			- chunk (runtime: a) global-a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			- ./a.js XX bytes [built] [code generated]
			@@ -50,0 +51,1 @@
			+ runtime modules XX KiB XX modules
			@@ -52,0 +54,1 @@
			+ runtime modules XX KiB XX modules
			@@ -53,1 +56,6 @@
			- global (Rspack x.x.x) compiled successfully in X.XX
			+ chunk (runtime: a, b, c) global-XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
			+ ./objects.js XX bytes [built] [code generated]
			+ chunk (runtime: a) global-a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ runtime modules XX KiB XX modules
			+ ./a.js XX bytes [built] [code generated]
			+ global (webpack x.x.x) compiled successfully in X ms"
		`);

	}
};
