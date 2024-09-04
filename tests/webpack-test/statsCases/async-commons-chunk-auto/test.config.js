const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -2,8 +2,4 @@
		- chunk (runtime: main) disabled/async-c.js (async-c) XX bytes [rendered]
		- > ./c ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) disabled/async-a.js (async-a) XX bytes [rendered]
		- > ./a ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./a.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) disabled/async-g.js (async-g) XX bytes [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		@@ -14,4 +10,1 @@
		- chunk (runtime: a) disabled/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		- > ./a a
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) disabled/b.js (b) XX bytes [entry] [rendered]
		+ chunk (runtime: b) disabled/b.js (b) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		@@ -19,0 +12,2 @@
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		@@ -20,1 +15,5 @@
		- chunk (runtime: c) disabled/c.js (c) XX bytes [entry] [rendered]
		+ chunk (runtime: main) disabled/async-a.js (async-a) XX bytes [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) disabled/c.js (c) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		@@ -22,1 +21,3 @@
		- ./c.js XX bytes [built] [code generated]
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		+ ./c.js + XX modules XX bytes [built] [code generated]
		@@ -25,0 +26,1 @@
		+ runtime modules XX KiB XX modules
		@@ -26,1 +28,13 @@
		- chunk (runtime: a, main) disabled/async-g.js (async-g) XX bytes [rendered]
		+ chunk (runtime: main) disabled/async-c.js (async-c) XX bytes [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX modules
		+ ./c.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: a) disabled/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ disabled (webpack x.x.x) compiled successfully
		+
		+ default:
		+ chunk (runtime: a, main) default/async-g.js (async-g) XX bytes [rendered]
		@@ -28,1 +42,0 @@
		- dependent modules XX bytes [dependent] XX module
		@@ -30,5 +43,15 @@
		- disabled (Rspack x.x.x) compiled successfully
		-
		- default:
		- chunk (runtime: main) default/async-c.js (async-c) XX bytes [rendered]
		- > ./c ./index.js XX:XX-XX
		+ chunk (runtime: main) default/async-b.js (async-b) XX bytes [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) default/b.js (b) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./b b
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/async-a.js (async-a) XX bytes [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) default/c.js (c) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./c c
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		@@ -36,1 +59,1 @@
		- chunk (runtime: main) default/async-a.js (async-a) XX bytes [rendered]
		+ chunk (runtime: main) default/XX.js XX bytes [rendered] split chunk (cache group: default)
		@@ -38,3 +61,0 @@
		- dependent modules XX bytes [dependent] XX module
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/async-b.js (async-b) XX bytes [rendered]
		@@ -42,1 +62,7 @@
		- ./b.js XX bytes [built] [code generated]
		+ > ./c ./index.js XX:XX-XX
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ ./node_modules/x.js XX bytes [built] [code generated]
		@@ -47,0 +73,4 @@
		+ chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./ main
		+ runtime modules XX KiB XX modules
		+ ./index.js XX bytes [built] [code generated]
		@@ -50,3 +80,1 @@
		- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
		- > ./a ./index.js XX:XX-XX
		- > ./b ./index.js XX:XX-XX
		+ chunk (runtime: main) default/async-c.js (async-c) XX bytes [rendered]
		@@ -54,2 +82,2 @@
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) default/XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) default/XX.js XX bytes [rendered] split chunk (cache group: default)
		@@ -62,2 +90,26 @@
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) default/b.js (b) XX bytes [entry] [rendered]
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ default (webpack x.x.x) compiled successfully
		+
		+ vendors:
		+ Entrypoint main XX KiB = vendors/main.js
		+ Entrypoint a XX KiB = vendors/vendors.js XX KiB vendors/a.js XX KiB
		+ Entrypoint b XX KiB = vendors/vendors.js XX KiB vendors/b.js XX KiB
		+ Entrypoint c XX KiB = vendors/vendors.js XX KiB vendors/c.js XX KiB
		+ chunk (runtime: a, main) vendors/async-g.js (async-g) XX bytes [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) vendors/async-b.js (async-b) XX bytes [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c) vendors/vendors.js (vendors) (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
		+ > ./a a
		+ > ./b b
		+ > ./c c
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ ./node_modules/y.js XX bytes [built] [code generated]
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: b) vendors/b.js (b) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -65,0 +117,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -66,1 +120,1 @@
		- chunk (runtime: main) default/XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
		+ chunk (runtime: main) vendors/async-a.js (async-a) XX bytes [rendered]
		@@ -68,4 +122,3 @@
		- > ./b ./index.js XX:XX-XX
		- > ./c ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: c) default/c.js (c) XX bytes [entry] [rendered]
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) vendors/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -73,0 +126,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -74,1 +129,1 @@
		- chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ chunk (runtime: main) vendors/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -76,0 +131,1 @@
		+ runtime modules XX KiB XX modules
		@@ -77,6 +133,0 @@
		- chunk (runtime: a, main) default/async-g.js (async-g) XX bytes [rendered]
		- > ./g ./a.js XX:XX-XX
		- ./g.js XX bytes [built] [code generated]
		- default (Rspack x.x.x) compiled successfully
		-
		- vendors:
		@@ -87,1 +137,27 @@
		- chunk (runtime: main) vendors/async-a.js (async-a) XX bytes [rendered]
		+ chunk (runtime: a) vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX module
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ vendors (webpack x.x.x) compiled successfully
		+
		+ multiple-vendors:
		+ Entrypoint main XX KiB = multiple-vendors/main.js
		+ Entrypoint a XX KiB = multiple-vendors/libs-x.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/a.js XX KiB
		+ Entrypoint b XX KiB = multiple-vendors/libs-x.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/b.js XX KiB
		+ Entrypoint c XX KiB = multiple-vendors/libs-x.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/XX.js XX bytes multiple-vendors/c.js XX KiB
		+ chunk (runtime: a, main) multiple-vendors/async-g.js (async-g) XX bytes [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) multiple-vendors/async-b.js (async-b) XX bytes [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) multiple-vendors/b.js (b) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./b b
		+ runtime modules XX KiB XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) multiple-vendors/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./a a
		+ ./e.js XX bytes [built] [code generated]
		+ chunk (runtime: main) multiple-vendors/async-a.js (async-a) XX bytes [rendered]
		@@ -89,1 +165,0 @@
		- dependent modules XX bytes [dependent] XX modules
		@@ -91,1 +166,6 @@
		- chunk (runtime: main) vendors/async-b.js (async-b) XX bytes [rendered]
		+ chunk (runtime: c) multiple-vendors/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./c c
		+ runtime modules XX KiB XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) multiple-vendors/libs-x.js (libs-x) (id hint: libs) XX bytes [initial] [rendered] split chunk (cache group: libs) (name: libs-x)
		+ > ./a ./index.js XX:XX-XX
		@@ -93,3 +173,1 @@
		- dependent modules XX bytes [dependent] XX modules
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./c ./index.js XX:XX-XX
		@@ -97,2 +175,0 @@
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) vendors/b.js (b) XX bytes [entry] [rendered]
		@@ -100,2 +176,0 @@
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: c) vendors/c.js (c) XX bytes [entry] [rendered]
		@@ -103,16 +177,2 @@
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) vendors/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		- > ./ main
		- ./index.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) vendors/async-g.js (async-g) XX bytes [rendered]
		- > ./g ./a.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX module
		- ./g.js XX bytes [built] [code generated]
		- vendors (Rspack x.x.x) compiled successfully
		-
		- multiple-vendors:
		- Entrypoint main XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/main.js XX KiB
		- Entrypoint a XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/a.js XX KiB
		- Entrypoint b XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/b.js XX KiB
		- Entrypoint c XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/c.js XX KiB
		- chunk (runtime: a, b, c, main) multiple-vendors/libs.js (libs) (id hint: libs) XX bytes [initial] [rendered] split chunk (cache group: libs)
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) multiple-vendors/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		@@ -122,1 +182,0 @@
		- > ./g ./a.js XX:XX-XX
		@@ -126,0 +185,8 @@
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, main) multiple-vendors/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./a a
		+ > ./b b
		+ ./node_modules/y.js XX bytes [built] [code generated]
		+ chunk (runtime: main) multiple-vendors/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -127,1 +194,1 @@
		- dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX KiB XX modules
		@@ -129,3 +196,11 @@
		- chunk (runtime: a) multiple-vendors/a.js (a) XX KiB [entry] [rendered]
		- > ./a a
		- chunk (runtime: b) multiple-vendors/b.js (b) XX KiB [entry] [rendered]
		+ chunk (runtime: c, main) multiple-vendors/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		+ > ./c ./index.js XX:XX-XX
		+ > ./c c
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) multiple-vendors/async-c.js (async-c) XX bytes [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) multiple-vendors/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./g ./a.js XX:XX-XX
		@@ -133,1 +208,0 @@
		- chunk (runtime: c) multiple-vendors/c.js (c) XX KiB [entry] [rendered]
		@@ -135,3 +209,6 @@
		- chunk (runtime: main) multiple-vendors/main.js (main) XX KiB [entry] [rendered]
		- > ./ main
		- multiple-vendors (Rspack x.x.x) compiled successfully
		+ ./f.js XX bytes [built] [code generated]
		+ chunk (runtime: a) multiple-vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ ./a.js XX bytes [built] [code generated]
		+ multiple-vendors (webpack x.x.x) compiled successfully
		@@ -141,7 +218,14 @@
		- Entrypoint a XX KiB = all/XX.js XX KiB all/a.js XX KiB
		- Entrypoint b XX KiB = all/XX.js XX KiB all/b.js XX KiB
		- Entrypoint c XX KiB = all/XX.js XX KiB all/c.js XX KiB
		- chunk (runtime: main) all/async-c.js (async-c) (id hint: vendors) XX bytes [rendered]
		- > ./c ./index.js XX:XX-XX
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) all/async-a.js (async-a) (id hint: vendors) XX bytes [rendered]
		+ Entrypoint a XX KiB = all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/a.js XX KiB
		+ Entrypoint b XX KiB = all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/b.js XX KiB
		+ Entrypoint c XX KiB = all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/XX.js XX bytes all/c.js XX KiB
		+ chunk (runtime: a, main) all/async-g.js (async-g) XX bytes [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all/async-b.js (async-b) XX bytes [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) all/b.js (b) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./b b
		+ runtime modules XX KiB XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) all/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		@@ -149,0 +233,1 @@
		+ > ./a a
		@@ -150,1 +235,1 @@
		- chunk (runtime: main) all/async-b.js (async-b) (id hint: vendors) XX bytes [rendered]
		+ chunk (runtime: main) all/async-a.js (async-a) XX bytes [rendered]
		@@ -152,5 +237,0 @@
		- > ./b ./index.js XX:XX-XX
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		- > ./a ./index.js XX:XX-XX
		- > ./a a
		@@ -158,2 +238,1 @@
		- chunk (runtime: c, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		- > ./c ./index.js XX:XX-XX
		+ chunk (runtime: c) all/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -161,0 +240,1 @@
		+ runtime modules XX KiB XX modules
		@@ -162,1 +242,1 @@
		- chunk (runtime: main) all/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
		+ chunk (runtime: a, b, c, main) all/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		@@ -166,0 +246,3 @@
		+ > ./a a
		+ > ./b b
		+ > ./c c
		@@ -167,2 +250,2 @@
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) all/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
		+ chunk (runtime: a, b, c, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		+ > ./a ./index.js XX:XX-XX
		@@ -171,4 +254,1 @@
		- > ./g ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: b, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		- > ./b ./index.js XX:XX-XX
		+ > ./a a
		@@ -176,2 +256,5 @@
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) all/a.js (a) XX KiB [entry] [rendered]
		+ > ./c c
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		@@ -179,1 +262,0 @@
		- chunk (runtime: b) all/b.js (b) XX KiB [entry] [rendered]
		@@ -181,3 +263,2 @@
		- chunk (runtime: c) all/c.js (c) XX KiB [entry] [rendered]
		- > ./c c
		- chunk (runtime: main) all/main.js (main) (id hint: vendors) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ ./node_modules/y.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		@@ -185,0 +266,1 @@
		+ runtime modules XX KiB XX modules
		@@ -186,1 +268,10 @@
		- chunk (runtime: a, main) all/async-g.js (async-g) (id hint: vendors) XX bytes [rendered]
		+ chunk (runtime: c, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
		+ > ./c ./index.js XX:XX-XX
		+ > ./c c
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all/async-c.js (async-c) XX bytes [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) all/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -188,2 +279,8 @@
		- ./g.js XX bytes [built] [code generated]
		- all (Rspack x.x.x) compiled successfully
		+ > ./b b
		+ > ./c c
		+ ./f.js XX bytes [built] [code generated]
		+ chunk (runtime: a) all/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ ./a.js XX bytes [built] [code generated]
		+ all (webpack x.x.x) compiled successfully"
	`);
	}
};
