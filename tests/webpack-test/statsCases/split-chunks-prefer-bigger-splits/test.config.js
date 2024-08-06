const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,3 +2,4 @@
		- chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./b.js XX bytes [built] [code generated]
		@@ -9,1 +10,1 @@
		- chunk (runtime: main) default/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ chunk (runtime: main) default/XX.js XX bytes <{XX}> ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		@@ -11,4 +12,0 @@
		- dependent modules XX bytes [dependent] XX module
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b ./index.js XX:XX-XX
		@@ -20,0 +17,1 @@
		+ runtime modules XX KiB XX modules
		@@ -21,1 +19,4 @@
		- Rspack x.x.x compiled successfully
		+ chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ webpack x.x.x compiled successfully"
	`);
	}
};
