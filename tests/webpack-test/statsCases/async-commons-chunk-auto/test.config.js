const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -2,8 +2,4 @@
			- chunk (runtime: main) disabled/async-c.js (async-c) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX modules
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) disabled/async-a.js (async-a) XX bytes [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX modules
			- ./a.js XX bytes [built] [code generated]
			+ chunk (runtime: a, main) disabled/async-g.js (async-g) XX bytes [rendered]
			+ > ./g ./a.js XX:XX-XX
			+ dependent modules XX bytes [dependent] XX module
			+ ./g.js XX bytes [built] [code generated]
			@@ -11,1 +7,1 @@
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
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
			@@ -26,5 +28,10 @@
			- chunk (runtime: a, main) disabled/async-g.js (async-g) XX bytes [rendered]
			- > ./g [XX] ./a.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX module
			- ./g.js XX bytes [built] [code generated]
			- disabled (Rspack x.x.x) compiled successfully
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
			@@ -33,7 +40,3 @@
			- chunk (runtime: main) default/async-c.js (async-c) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/async-a.js (async-a) XX bytes [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX module
			- ./a.js XX bytes [built] [code generated]
			+ chunk (runtime: a, main) default/async-g.js (async-g) XX bytes [rendered]
			+ > ./g ./a.js XX:XX-XX
			+ ./g.js XX bytes [built] [code generated]
			@@ -41,1 +44,1 @@
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			@@ -43,21 +46,1 @@
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- ./node_modules/y.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			- > ./c [XX] ./index.js XX:XX-XX
			- ./node_modules/z.js XX bytes [built] [code generated]
			- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			- ./node_modules/x.js XX bytes [built] [code generated]
			- chunk (runtime: a, main) default/XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			- > ./g [XX] ./a.js XX:XX-XX
			- ./f.js XX bytes [built] [code generated]
			- chunk (runtime: a) default/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			- > ./a a
			- ./a.js XX bytes [built] [code generated]
			- chunk (runtime: b) default/b.js (b) XX bytes [entry] [rendered]
			+ chunk (runtime: b) default/b.js (b) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
			@@ -65,0 +48,2 @@
			+ dependent modules XX bytes [dependent] XX modules
			+ runtime modules XX bytes XX modules
			@@ -66,6 +51,4 @@
			- chunk (runtime: main) default/XX.js (id hint: ) XX bytes [rendered] split chunk (cache group: default)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			- ./d.js XX bytes [built] [code generated]
			- chunk (runtime: c) default/c.js (c) XX bytes [entry] [rendered]
			+ chunk (runtime: main) default/async-a.js (async-a) XX bytes [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ chunk (runtime: c) default/c.js (c) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
			@@ -73,0 +56,2 @@
			+ dependent modules XX bytes [dependent] XX modules
			+ runtime modules XX bytes XX modules
			@@ -74,0 +59,14 @@
			+ chunk (runtime: main) default/XX.js XX bytes [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			+ ./d.js XX bytes [built] [code generated]
			+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			+ ./node_modules/x.js XX bytes [built] [code generated]
			+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ ./node_modules/y.js XX bytes [built] [code generated]
			@@ -76,0 +75,1 @@
			+ runtime modules XX KiB XX modules
			@@ -77,4 +77,17 @@
			- chunk (runtime: a, main) default/async-g.js (async-g) XX bytes [rendered]
			- > ./g [XX] ./a.js XX:XX-XX
			- ./g.js XX bytes [built] [code generated]
			- default (Rspack x.x.x) compiled successfully
			+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: defaultVendors)
			+ > ./c ./index.js XX:XX-XX
			+ ./node_modules/z.js XX bytes [built] [code generated]
			+ chunk (runtime: main) default/async-c.js (async-c) XX bytes [rendered]
			+ > ./c ./index.js XX:XX-XX
			+ ./c.js XX bytes [built] [code generated]
			+ chunk (runtime: a, main) default/XX.js XX bytes [rendered] split chunk (cache group: default)
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			+ > ./g ./a.js XX:XX-XX
			+ ./f.js XX bytes [built] [code generated]
			+ chunk (runtime: a) default/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ > ./a a
			+ runtime modules XX KiB XX modules
			+ dependent modules XX bytes [dependent] XX modules
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ default (webpack x.x.x) compiled successfully
			@@ -83,8 +96,8 @@
			- chunk (runtime: main) vendors/async-c.js (async-c) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX modules
			- ./c.js XX bytes [built] [code generated]
			- chunk (runtime: main) vendors/async-a.js (async-a) XX bytes [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX modules
			- ./a.js XX bytes [built] [code generated]
			+ Entrypoint main XX KiB = vendors/main.js
			+ Entrypoint a XX KiB = vendors/vendors.js XX KiB vendors/a.js XX KiB
			+ Entrypoint b XX KiB = vendors/vendors.js XX KiB vendors/b.js XX KiB
			+ Entrypoint c XX KiB = vendors/vendors.js XX KiB vendors/c.js XX KiB
			+ chunk (runtime: a, main) vendors/async-g.js (async-g) XX bytes [rendered]
			+ > ./g ./a.js XX:XX-XX
			+ dependent modules XX bytes [dependent] XX module
			+ ./g.js XX bytes [built] [code generated]
			@@ -92,1 +105,1 @@
			- > ./b [XX] ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			@@ -95,1 +108,1 @@
			- chunk (runtime: a) vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ chunk (runtime: a, b, c) vendors/vendors.js (vendors) (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
			@@ -97,2 +110,6 @@
			- ./a.js XX bytes [built] [code generated]
			- chunk (runtime: b) vendors/b.js (b) XX bytes [entry] [rendered]
			+ > ./b b
			+ > ./c c
			+ ./node_modules/x.js XX bytes [built] [code generated]
			+ ./node_modules/y.js XX bytes [built] [code generated]
			+ ./node_modules/z.js XX bytes [built] [code generated]
			+ chunk (runtime: b) vendors/b.js (b) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			@@ -100,0 +117,2 @@
			+ runtime modules XX KiB XX modules
			+ dependent modules XX bytes [dependent] XX modules
			@@ -101,1 +120,5 @@
			- chunk (runtime: c) vendors/c.js (c) XX bytes [entry] [rendered]
			+ chunk (runtime: main) vendors/async-a.js (async-a) XX bytes [rendered]
			+ > ./a ./index.js XX:XX-XX
			+ dependent modules XX bytes [dependent] XX modules
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ chunk (runtime: c) vendors/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			@@ -103,0 +126,2 @@
			+ runtime modules XX KiB XX modules
			+ dependent modules XX bytes [dependent] XX modules
			@@ -106,0 +131,1 @@
			+ runtime modules XX KiB XX modules
			@@ -107,2 +133,7 @@
			- chunk (runtime: a, main) vendors/async-g.js (async-g) XX bytes [rendered]
			- > ./g [XX] ./a.js XX:XX-XX
			+ chunk (runtime: main) vendors/async-c.js (async-c) XX bytes [rendered]
			+ > ./c ./index.js XX:XX-XX
			+ dependent modules XX bytes [dependent] XX modules
			+ ./c.js XX bytes [built] [code generated]
			+ chunk (runtime: a) vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ > ./a a
			+ runtime modules XX KiB XX modules
			@@ -110,2 +141,2 @@
			- ./g.js XX bytes [built] [code generated]
			- vendors (Rspack x.x.x) compiled successfully
			+ ./a.js + XX modules XX bytes [built] [code generated]
			+ vendors (webpack x.x.x) compiled successfully
			@@ -114,6 +145,29 @@
			- Entrypoint main XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/main.js XX KiB
			- Entrypoint a XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/a.js XX KiB
			- Entrypoint b XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/b.js XX KiB
			- Entrypoint c XX KiB = multiple-vendors/libs.js XX KiB multiple-vendors/c.js XX KiB
			- chunk (runtime: a, b, c, main) multiple-vendors/libs.js (libs) (id hint: libs) XX bytes [initial] [rendered] split chunk (cache group: libs)
			- > ./ main
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
			+ > ./a ./index.js XX:XX-XX
			+ ./a.js XX bytes [built] [code generated]
			+ chunk (runtime: c) multiple-vendors/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ > ./c c
			+ runtime modules XX KiB XX modules
			+ ./c.js XX bytes [built] [code generated]
			+ chunk (runtime: a, b, c, main) multiple-vendors/libs-x.js (libs-x) (id hint: libs) XX bytes [initial] [rendered] split chunk (cache group: libs) (name: libs-x)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -123,7 +177,5 @@
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			- > ./g [XX] ./a.js XX:XX-XX
			- dependent modules XX bytes [dependent] XX modules
			- ./index.js XX bytes [built] [code generated]
			- chunk (runtime: a) multiple-vendors/a.js (a) XX KiB [entry] [rendered]
			+ ./node_modules/x.js XX bytes [built] [code generated]
			+ chunk (runtime: a, b, c, main) multiple-vendors/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			@@ -131,1 +183,0 @@
			- chunk (runtime: b) multiple-vendors/b.js (b) XX KiB [entry] [rendered]
			@@ -133,1 +184,0 @@
			- chunk (runtime: c) multiple-vendors/c.js (c) XX KiB [entry] [rendered]
			@@ -135,1 +185,8 @@
			- chunk (runtime: main) multiple-vendors/main.js (main) XX KiB [entry] [rendered]
			+ ./d.js XX bytes [built] [code generated]
			+ chunk (runtime: a, b, main) multiple-vendors/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./a a
			+ > ./b b
			+ ./node_modules/y.js XX bytes [built] [code generated]
			+ chunk (runtime: main) multiple-vendors/main.js (main) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			@@ -137,1 +194,21 @@
			- multiple-vendors (Rspack x.x.x) compiled successfully
			+ runtime modules XX KiB XX modules
			+ ./index.js XX bytes [built] [code generated]
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
			+ > ./b b
			+ > ./c c
			+ ./f.js XX bytes [built] [code generated]
			+ chunk (runtime: a) multiple-vendors/a.js (a) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			+ > ./a a
			+ runtime modules XX KiB XX modules
			+ ./a.js XX bytes [built] [code generated]
			+ multiple-vendors (webpack x.x.x) compiled successfully
			@@ -141,14 +218,15 @@
			- Entrypoint a XX KiB = all/XX.js XX KiB all/a.js XX KiB
			- Entrypoint b XX KiB = all/XX.js XX KiB all/b.js XX KiB
			- Entrypoint c XX KiB = all/XX.js XX KiB all/c.js XX KiB
			- chunk (runtime: main) all/async-c.js (async-c) (id hint: vendors) XX bytes [rendered]
			- > ./c [XX] ./index.js XX:XX-XX
			- ./node_modules/z.js XX bytes [built] [code generated]
			- chunk (runtime: main) all/async-a.js (async-a) (id hint: vendors) XX bytes [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- ./e.js XX bytes [built] [code generated]
			- chunk (runtime: main) all/async-b.js (async-b) (id hint: vendors) XX bytes [rendered]
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- ./node_modules/y.js XX bytes [built] [code generated]
			- chunk (runtime: a, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
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
			+ > ./a ./index.js XX:XX-XX
			@@ -156,1 +234,3 @@
			- > ./a [XX] ./index.js XX:XX-XX
			+ ./e.js XX bytes [built] [code generated]
			+ chunk (runtime: main) all/async-a.js (async-a) XX bytes [rendered]
			+ > ./a ./index.js XX:XX-XX
			@@ -158,1 +238,1 @@
			- chunk (runtime: c, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
			+ chunk (runtime: c) all/c.js (c) XX bytes (javascript) XX KiB (runtime) [entry] [rendered]
			@@ -160,1 +240,1 @@
			- > ./c [XX] ./index.js XX:XX-XX
			+ runtime modules XX KiB XX modules
			@@ -162,4 +242,7 @@
			- chunk (runtime: main) all/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
			- > ./a [XX] ./index.js XX:XX-XX
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			+ chunk (runtime: a, b, c, main) all/XX.js XX bytes [initial] [rendered] split chunk (cache group: default)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			+ > ./a a
			+ > ./b b
			+ > ./c c
			@@ -167,0 +250,7 @@
			+ chunk (runtime: a, b, c, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
			+ > ./a ./index.js XX:XX-XX
			+ > ./b ./index.js XX:XX-XX
			+ > ./c ./index.js XX:XX-XX
			+ > ./a a
			+ > ./b b
			+ > ./c c
			@@ -168,10 +258,3 @@
			- chunk (runtime: a, main) all/XX.js (id hint: vendors) XX bytes [rendered] split chunk (cache group: vendors)
			- > ./b [XX] ./index.js XX:XX-XX
			- > ./c [XX] ./index.js XX:XX-XX
			- > ./g [XX] ./a.js XX:XX-XX
			- ./f.js XX bytes [built] [code generated]
			- chunk (runtime: b, main) all/XX.js (id hint: vendors) XX bytes [initial] [rendered] split chunk (cache group: vendors)
			- > ./b b
			- > ./b [XX] ./index.js XX:XX-XX
			- ./b.js XX bytes [built] [code generated]
			- chunk (runtime: a) all/a.js (a) XX KiB [entry] [rendered]
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
			@@ -186,4 +268,19 @@
			- chunk (runtime: a, main) all/async-g.js (async-g) (id hint: vendors) XX bytes [rendered]
			- > ./g [XX] ./a.js XX:XX-XX
			- ./g.js XX bytes [built] [code generated]
			- all (Rspack x.x.x) compiled successfully
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
			+ > ./g ./a.js XX:XX-XX
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
