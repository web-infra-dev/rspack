const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,0 +4,7 @@
		+ chunk (runtime: b) b.js (b) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		+ > ./b b
		+ runtime modules XX KiB XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) c.js (c) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./c ./b.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		@@ -10,0 +17,1 @@
		+ runtime modules XX KiB XX modules
		@@ -11,7 +19,1 @@
		- chunk (runtime: b) b.js (b) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		- > ./b b
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: b) c.js (c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c ./b.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- default (Rspack x.x.x) compiled successfully
		+ default (webpack x.x.x) compiled successfully"
	`);
	}
};
