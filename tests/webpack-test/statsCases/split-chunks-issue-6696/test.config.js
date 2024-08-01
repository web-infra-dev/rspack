const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,1 +2,5 @@
			- chunk (runtime: main) vendors.js (vendors) (id hint: vendors) XX bytes ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: vendors)
			+ chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> <{XX}> [rendered]
			+ > ./b ./index.js XX:XX-XX
			+ dependent modules XX bytes [dependent] XX module
			+ ./b.js XX bytes [built] [code generated]
			+ chunk (runtime: main) vendors.js (vendors) (id hint: vendors) XX bytes ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
			@@ -9,4 +13,0 @@
			- chunk (runtime: main) async-b.js (async-b) XX bytes <{XX}> <{XX}> [rendered]
			- > ./b ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX module
			- ./b.js XX bytes [built] [code generated]
			@@ -15,0 +15,1 @@
			+ runtime modules XX KiB XX modules
			@@ -16,1 +17,1 @@
			- default (Rspack x.x.x) compiled successfully
			+ default (webpack x.x.x) compiled successfully"
		`);

	}
};
