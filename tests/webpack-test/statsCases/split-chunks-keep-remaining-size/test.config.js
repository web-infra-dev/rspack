const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,6 +2,0 @@
		- chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= [rendered]
		- > ./a ./index.js XX:XX-XX
		- ./a.js XX bytes [built] [code generated]
		@@ -11,0 +5,6 @@
		+ chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: main) default/async-d.js (async-d) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./d ./index.js XX:XX-XX
		+ ./d.js XX bytes [built] [code generated]
		@@ -19,7 +19,1 @@
		- chunk (runtime: main) default/async-d.js (async-d) XX bytes <{XX}> ={XX}= [rendered]
		- > ./d ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= [rendered] split chunk (cache group: defaultVendors)
		- > ./a ./index.js XX:XX-XX
		- ./node_modules/shared.js?XX XX bytes [built] [code generated]
		- chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		+ chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		@@ -27,0 +21,1 @@
		+ runtime modules XX KiB XX modules
		@@ -28,1 +23,4 @@
		- Rspack x.x.x compiled successfully
		+ chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ webpack x.x.x compiled successfully"
	`);
	}
};
