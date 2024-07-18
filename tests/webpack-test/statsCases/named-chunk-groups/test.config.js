const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,0 +1,1 @@
			+ Chunk Group main XX KiB = a-main.js
			@@ -4,6 +5,9 @@
			- Chunk Group main XX KiB = a-main.js
			- chunk (runtime: main) a-async-c.js (async-c) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) a-vendors.js (vendors) (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) a-XX.js XX bytes [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ ./shared.js XX bytes [built] [code generated]
			+ chunk (runtime: main) a-async-b.js (async-b) XX bytes [rendered]
			+ > ./b ./index.js XX:XX-XX
			+ ./b.js XX bytes [built] [code generated]
			+ chunk (runtime: main) a-vendors.js (vendors) (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors) (name: vendors)
			+ > ./c ./index.js XX:XX-XX
			@@ -13,1 +17,1 @@
			- > ./a [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			@@ -15,7 +19,0 @@
			- chunk (runtime: main) a-async-b.js (async-b) XX bytes [rendered]
			- > ./b [XX] ./index.js XX:XX-XX
			- ./b.js XX bytes [built] [code generated]
			- chunk (runtime: main) a-XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- ./shared.js XX bytes [built] [code generated]
			@@ -24,0 +21,1 @@
			+ runtime modules XX KiB XX modules
			@@ -25,1 +23,4 @@
			- Rspack x.x.x compiled successfully
			+ chunk (runtime: main) a-async-c.js (async-c) XX bytes [rendered]
			+ > ./c ./index.js XX:XX-XX
			+ ./c.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully
			@@ -31,5 +32,9 @@
			- chunk (runtime: main) b-async-c.js (async-c) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) b-vendors.js (vendors) (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) b-XX.js XX bytes [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ ./shared.js XX bytes [built] [code generated]
			+ chunk (runtime: main) b-async-b.js (async-b) XX bytes [rendered]
			+ > ./b ./index.js XX:XX-XX
			+ ./b.js XX bytes [built] [code generated]
			+ chunk (runtime: main) b-vendors.js (vendors) (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors) (name: vendors)
			+ > ./c ./index.js XX:XX-XX
			@@ -39,1 +44,1 @@
			- > ./a [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			@@ -41,7 +46,0 @@
			- chunk (runtime: main) b-async-b.js (async-b) XX bytes [rendered]
			- > ./b [XX] ./index.js XX:XX-XX
			- ./b.js XX bytes [built] [code generated]
			- chunk (runtime: main) b-XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- ./shared.js XX bytes [built] [code generated]
			@@ -50,0 +48,1 @@
			+ runtime modules XX KiB XX modules
			@@ -51,1 +50,4 @@
			- Rspack x.x.x compiled successfully
			+ chunk (runtime: main) b-async-c.js (async-c) XX bytes [rendered]
			+ > ./c ./index.js XX:XX-XX
			+ ./c.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully"
		`);

	}
};
