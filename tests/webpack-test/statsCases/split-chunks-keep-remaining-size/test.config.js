const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,6 +2,0 @@
			- chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./a.js XX bytes [built] [code generated]
			@@ -9,1 +3,1 @@
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			@@ -11,0 +5,6 @@
			+ chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ chunk (runtime: main) default/async-d.js (async-d) XX bytes <{XX}> ={XX}= [rendered]
			+ > ./d ./index.js XX:XX-XX
			+ ./d.js XX bytes [built] [code generated]
			@@ -12,1 +12,1 @@
			- > ./d [XX] ./index.js XX:XX-XX
			+ > ./d ./index.js XX:XX-XX
			@@ -16,2 +16,2 @@
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -19,7 +19,1 @@
			- chunk (runtime: main) default/async-d.js (async-d) XX bytes <{XX}> ={XX}= [rendered]
			- > ./d [XX] ./index.js XX:XX-XX
			- ./d.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= [rendered] split chunk (cache group: defaultVendors)
			- > ./a [XX] ./index.js XX:XX-XX
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
