const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,6 +2,0 @@
			- chunk (runtime: main) async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./a.js XX bytes [built] [code generated]
			@@ -9,1 +3,1 @@
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			@@ -11,4 +5,7 @@
			- chunk (runtime: main) XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js XX bytes [built] [code generated]
			+ chunk (runtime: main) XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -17,3 +14,3 @@
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
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
