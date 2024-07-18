const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,0 +2,3 @@
			+ chunk (runtime: main) default/async-b.js (async-b) (id hint: vendors) XX bytes <{XX}> [rendered] reused as split chunk (cache group: defaultVendors)
			+ > b ./index.js XX:XX-XX
			+ ./node_modules/b.js XX bytes [built] [code generated]
			@@ -3,1 +6,1 @@
			- > a [XX] ./index.js XX:XX-XX
			+ > a ./index.js XX:XX-XX
			@@ -5,6 +8,3 @@
			- chunk (runtime: main) default/async-b.js (async-b) (id hint: vendors) XX bytes <{XX}> [rendered]
			- > b [XX] ./index.js XX:XX-XX
			- ./node_modules/b.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/async-c-XX.js (async-c-XX) (id hint: vendors) XX bytes <{XX}> [rendered]
			- > c [XX] ./index.js XX:XX-XX
			- > c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) default/async-c-XX.js (async-c-XX) (id hint: vendors) XX bytes <{XX}> [rendered] reused as split chunk (cache group: defaultVendors)
			+ > c ./index.js XX:XX-XX
			+ > c ./index.js XX:XX-XX
			@@ -14,0 +14,1 @@
			+ runtime modules XX KiB XX modules
			@@ -15,1 +16,1 @@
			- Rspack x.x.x compiled successfully
			+ webpack x.x.x compiled successfully"
		`);

	}
};
