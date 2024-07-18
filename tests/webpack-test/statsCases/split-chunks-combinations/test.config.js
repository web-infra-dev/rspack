const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,2 +2,2 @@
			- chunk (runtime: main) async-c.js (async-c) XX bytes <{XX}> [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) async-g.js (async-g) XX bytes <{XX}> [rendered]
			+ > ./g ./index.js XX:XX-XX
			@@ -5,1 +5,4 @@
			- ./c.js XX bytes [built] [code generated]
			+ ./g.js XX bytes [built] [code generated]
			+ chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
			+ > ./b ./index.js XX:XX-XX
			+ ./b.js XX bytes [built] [code generated]
			@@ -7,1 +10,1 @@
			- > ./f [XX] ./index.js XX:XX-XX
			+ > ./f ./index.js XX:XX-XX
			@@ -10,11 +13,0 @@
			- chunk (runtime: main) async-a.js (async-a) XX bytes <{XX}> ={XX}= [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./a.js XX bytes [built] [code generated]
			- chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
			- > ./b [XX] ./index.js XX:XX-XX
			- ./b.js XX bytes [built] [code generated]
			- chunk (runtime: main) XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= [rendered] split chunk (cache group: default)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- ./x.js XX bytes [built] [code generated]
			- ./y.js XX bytes [built] [code generated]
			@@ -22,1 +14,1 @@
			- > ./e [XX] ./index.js XX:XX-XX
			+ > ./e ./index.js XX:XX-XX
			@@ -25,0 +17,3 @@
			+ chunk (runtime: main) async-a.js (async-a) XX bytes <{XX}> ={XX}= [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js XX bytes [built] [code generated]
			@@ -26,1 +21,1 @@
			- > ./d [XX] ./index.js XX:XX-XX
			+ > ./d ./index.js XX:XX-XX
			@@ -31,0 +26,1 @@
			+ runtime modules XX KiB XX modules
			@@ -32,2 +28,2 @@
			- chunk (runtime: main) async-g.js (async-g) XX bytes <{XX}> [rendered]
			- > ./g [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) async-c.js (async-c) XX bytes <{XX}> [rendered]
			+ > ./c ./index.js XX:XX-XX
			@@ -35,2 +31,7 @@
			- ./g.js XX bytes [built] [code generated]
			- Rspack x.x.x compiled successfully
			+ ./c.js XX bytes [built] [code generated]
			+ chunk (runtime: main) XX.js XX bytes <{XX}> ={XX}= ={XX}= [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ ./x.js XX bytes [built] [code generated]
			+ ./y.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully"
		`);

	}
};
