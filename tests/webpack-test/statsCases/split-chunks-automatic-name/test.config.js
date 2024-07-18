const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,5 +2,5 @@
			- chunk (runtime: main) async-a.js (async-a) (id hint: common) XX bytes <{main}> ={common-d_js}= ={common-node_modules_x_js}= ={common-node_modules_y_js}= [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./a.js + XX modules XX bytes [code generated]
			- chunk (runtime: main) async-b.js (async-b) (id hint: common) XX bytes <{main}> ={common-d_js}= ={common-f_js}= ={common-node_modules_x_js}= ={common-node_modules_y_js}= [rendered]
			- > ./b [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) async-a.js (async-a) XX bytes <{main}> ={common-d_js}= ={common-node_modules_x_js}= ={common-node_modules_y_js}= [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ chunk (runtime: main) async-b.js (async-b) XX bytes <{main}> ={common-d_js}= ={common-f_js}= ={common-node_modules_x_js}= ={common-node_modules_y_js}= [rendered]
			+ > ./b ./index.js XX:XX-XX
			@@ -8,2 +8,2 @@
			- chunk (runtime: main) async-c.js (async-c) (id hint: common) XX bytes <{main}> ={common-d_js}= ={common-f_js}= ={common-node_modules_x_js}= ={common-node_modules_z_js}= [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: main) async-c.js (async-c) XX bytes <{main}> ={common-d_js}= ={common-f_js}= ={common-node_modules_x_js}= ={common-node_modules_z_js}= [rendered]
			+ > ./c ./index.js XX:XX-XX
			@@ -12,3 +12,3 @@
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -17,2 +17,2 @@
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -21,3 +21,3 @@
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -26,2 +26,2 @@
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			@@ -30,1 +30,1 @@
			- > ./c [XX] ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -32,1 +32,1 @@
			- chunk (runtime: main) main.js (main) (id hint: common) XX bytes (javascript) XX KiB (runtime) >{async-a}< >{async-b}< >{async-c}< >{common-d_js}< >{common-f_js}< >{common-node_modules_x_js}< >{common-node_modules_y_js}< >{common-node_modules_z_js}< [entry] [rendered]
			+ chunk (runtime: main) main.js (main) XX bytes (javascript) XX KiB (runtime) >{async-a}< >{async-b}< >{async-c}< >{common-d_js}< >{common-f_js}< >{common-node_modules_x_js}< >{common-node_modules_y_js}< >{common-node_modules_z_js}< [entry] [rendered]
			@@ -34,0 +34,1 @@
			+ runtime modules XX KiB XX modules
			@@ -35,1 +36,1 @@
			- production (Rspack x.x.x) compiled successfully
			+ production (webpack x.x.x) compiled successfully"
		`);

	}
};
