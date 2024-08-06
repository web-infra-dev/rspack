const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,3 +2,3 @@
		- chunk (runtime: main) async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		@@ -8,4 +8,1 @@
		- chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
		- > ./b ./index.js XX:XX-XX
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: main) XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		+ chunk (runtime: main) XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		@@ -23,0 +20,1 @@
		+ runtime modules XX KiB XX modules
		@@ -24,1 +22,4 @@
		- default (Rspack x.x.x) compiled successfully
		+ chunk (runtime: main) async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ default (webpack x.x.x) compiled successfully"
	`);
	}
};
