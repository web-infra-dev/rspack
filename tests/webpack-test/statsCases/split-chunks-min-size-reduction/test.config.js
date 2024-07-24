const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,8 +2,2 @@
			- chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./a.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
			- > ./b [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) default/async-b.js (async-b) XX bytes <{XX}> [rendered]
			+ > ./b ./index.js XX:XX-XX
			@@ -11,0 +5,1 @@
			+ ./node_modules/shared.js?XX XX bytes [dependent] [built] [code generated]
			@@ -12,1 +7,1 @@
			- > ./e [XX] ./index.js XX:XX-XX
			+ > ./e ./index.js XX:XX-XX
			@@ -14,5 +9,4 @@
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
			- > ./c [XX] ./index.js XX:XX-XX
			- > ./d [XX] ./index.js XX:XX-XX
			- > ./e [XX] ./index.js XX:XX-XX
			- ./node_modules/shared.js?XX XX bytes [built] [code generated]
			+ chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js XX bytes [built] [code generated]
			+ ./node_modules/shared.js?XX XX bytes [dependent] [built] [code generated]
			@@ -20,1 +14,1 @@
			- > ./d [XX] ./index.js XX:XX-XX
			+ > ./d ./index.js XX:XX-XX
			@@ -22,3 +16,4 @@
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
			+ > ./c ./index.js XX:XX-XX
			+ > ./d ./index.js XX:XX-XX
			+ > ./e ./index.js XX:XX-XX
			@@ -26,1 +21,1 @@
			- chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
			+ chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
			@@ -28,0 +23,1 @@
			+ runtime modules XX KiB XX modules
			@@ -29,1 +25,4 @@
			- Rspack x.x.x compiled successfully
			+ chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
			+ > ./c ./index.js XX:XX-XX
			+ ./c.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully"
		`);

	}
};
