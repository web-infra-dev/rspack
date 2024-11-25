const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,9 +4,5 @@
		- Entrypoint b XX bytes = default/b.js
		- Entrypoint c XX bytes = default/c.js
		- chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX module
		- ./a.js XX bytes [built] [code generated]
		+ Entrypoint b XX KiB = default/b.js
		+ Entrypoint c XX KiB = default/c.js
		+ chunk (runtime: a, main) default/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		@@ -16,1 +12,6 @@
		- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ chunk (runtime: b) default/b.js (b) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./b b
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		@@ -18,0 +19,8 @@
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) default/c.js (c) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./c c
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		@@ -19,2 +28,0 @@
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		@@ -22,1 +29,1 @@
		- ./node_modules/z.js XX bytes [built] [code generated]
		+ ./d.js XX bytes [built] [code generated]
		@@ -28,12 +35,1 @@
		- chunk (runtime: a, main) default/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b ./index.js XX:XX-XX
		- > ./c ./index.js XX:XX-XX
		- > ./g ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: a) default/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		- > ./a a
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) default/b.js (b) XX bytes [entry] [rendered]
		- > ./b b
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		@@ -42,5 +38,1 @@
		- > ./c ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: c) default/c.js (c) XX bytes [entry] [rendered]
		- > ./c c
		- ./c.js XX bytes [built] [code generated]
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -49,0 +41,1 @@
		+ runtime modules XX KiB XX modules
		@@ -50,1 +43,9 @@
		- chunk (runtime: a, main) default/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		+ > ./c ./index.js XX:XX-XX
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) default/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -52,2 +53,7 @@
		- ./g.js XX bytes [built] [code generated]
		- default (Rspack x.x.x) compiled successfully
		+ ./f.js XX bytes [built] [code generated]
		+ chunk (runtime: a) default/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ default (webpack x.x.x) compiled successfully
		@@ -57,11 +63,7 @@
		- Entrypoint a XX KiB = all-chunks/XX.js XX KiB all-chunks/a.js XX KiB
		- Entrypoint b XX KiB = all-chunks/async-b.js XX KiB all-chunks/b.js XX KiB
		- Entrypoint c XX KiB = all-chunks/async-c.js XX KiB all-chunks/c.js XX KiB
		- chunk (runtime: c, main) all-chunks/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		- > ./c ./index.js XX:XX-XX
		- > ./c c
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a ./index.js XX:XX-XX
		- ./e.js XX bytes [built] [code generated]
		- chunk (runtime: b, main) all-chunks/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ Entrypoint a XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/a.js XX KiB
		+ Entrypoint b XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/b.js XX KiB
		+ Entrypoint c XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/c.js XX KiB
		+ chunk (runtime: a, main) all-chunks/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all-chunks/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		@@ -69,0 +71,2 @@
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) all-chunks/b.js (b) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -70,0 +74,1 @@
		+ runtime modules XX KiB XX modules
		@@ -71,1 +76,1 @@
		- chunk (runtime: a, main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ chunk (runtime: a, main) all-chunks/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		@@ -74,0 +79,3 @@
		+ ./e.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all-chunks/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		@@ -75,1 +83,5 @@
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ chunk (runtime: c) all-chunks/c.js (c) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./c c
		+ runtime modules XX KiB XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) all-chunks/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		@@ -78,2 +90,0 @@
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		@@ -81,2 +91,5 @@
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ > ./a a
		+ > ./b b
		+ > ./c c
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -84,4 +97,0 @@
		- > ./b ./index.js XX:XX-XX
		- > ./c ./index.js XX:XX-XX
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		@@ -90,3 +99,0 @@
		- > ./g ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: a) all-chunks/a.js (a) XX KiB ={XX}= >{XX}< >{XX}< [entry] [rendered]
		@@ -94,1 +100,0 @@
		- chunk (runtime: b) all-chunks/b.js (b) XX KiB ={XX}= [entry] [rendered]
		@@ -96,1 +101,3 @@
		- chunk (runtime: main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		+ > ./c c
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -99,4 +106,3 @@
		- > ./c ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: c) all-chunks/c.js (c) XX KiB ={XX}= [entry] [rendered]
		- > ./c c
		+ > ./a a
		+ > ./b b
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -105,0 +111,1 @@
		+ runtime modules XX KiB XX modules
		@@ -106,1 +113,10 @@
		- chunk (runtime: a, main) all-chunks/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ chunk (runtime: c, main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./c ./index.js XX:XX-XX
		+ > ./c c
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all-chunks/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) all-chunks/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -108,2 +124,8 @@
		- ./g.js XX bytes [built] [code generated]
		- all-chunks (Rspack x.x.x) compiled successfully
		+ > ./b b
		+ > ./c c
		+ ./f.js XX bytes [built] [code generated]
		+ chunk (runtime: a) all-chunks/a.js (a) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ ./a.js XX bytes [built] [code generated]
		+ all-chunks (webpack x.x.x) compiled successfully
		@@ -113,5 +135,9 @@
		- Entrypoint a XX KiB = manual/vendors.js XX bytes manual/a.js XX KiB
		- Entrypoint b XX KiB = manual/vendors.js XX bytes manual/b.js XX KiB
		- Entrypoint c XX KiB = manual/vendors.js XX bytes manual/c.js XX KiB
		- chunk (runtime: main) manual/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		+ Entrypoint a XX KiB = manual/vendors.js XX KiB manual/a.js XX KiB
		+ Entrypoint b XX KiB = manual/vendors.js XX KiB manual/b.js XX KiB
		+ Entrypoint c XX KiB = manual/vendors.js XX KiB manual/c.js XX KiB
		+ chunk (runtime: a, main) manual/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) manual/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		@@ -119,2 +145,2 @@
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: a, b, c, main) manual/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors)
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) manual/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
		@@ -139,14 +165,0 @@
		- chunk (runtime: main) manual/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		- > ./a ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: main) manual/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		- > ./b ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) manual/a.js (a) XX bytes (javascript) XX KiB (runtime) ={XX}= >{XX}< [entry] [rendered]
		- > ./a a
		- > x a
		- > y a
		- > z a
		- ./a.js XX bytes [built] [code generated]
		@@ -158,0 +170,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -159,0 +173,4 @@
		+ chunk (runtime: main) manual/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./a.js + XX modules XX bytes [built] [code generated]
		@@ -164,0 +182,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -167,0 +187,1 @@
		+ runtime modules XX KiB XX modules
		@@ -168,2 +189,10 @@
		- chunk (runtime: a, main) manual/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> [rendered]
		- > ./g ./a.js XX:XX-XX
		+ chunk (runtime: main) manual/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a) manual/a.js (a) XX bytes (javascript) XX KiB (runtime) ={XX}= >{XX}< [entry] [rendered]
		+ > ./a a
		+ > x a
		+ > y a
		+ > z a
		+ runtime modules XX KiB XX modules
		@@ -171,2 +200,2 @@
		- ./g.js XX bytes [built] [code generated]
		- manual (Rspack x.x.x) compiled successfully
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ manual (webpack x.x.x) compiled successfully
		@@ -176,13 +205,7 @@
		- Entrypoint aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa XX KiB = name-too-long/XX.js XX KiB name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js XX KiB
		- Entrypoint bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb XX KiB = name-too-long/async-b.js XX KiB name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js XX KiB
		- Entrypoint cccccccccccccccccccccccccccccc XX KiB = name-too-long/async-c.js XX KiB name-too-long/cccccccccccccccccccccccccccccc.js XX KiB
		- chunk (runtime: cccccccccccccccccccccccccccccc, main) name-too-long/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		- > ./c ./index.js XX:XX-XX
		- > ./c cccccccccccccccccccccccccccccc
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js (bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) XX KiB ={XX}= [entry] [rendered]
		- > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		- chunk (runtime: main) name-too-long/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a ./index.js XX:XX-XX
		- ./e.js XX bytes [built] [code generated]
		- chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, main) name-too-long/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ Entrypoint aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js XX KiB
		+ Entrypoint bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js XX KiB
		+ Entrypoint cccccccccccccccccccccccccccccc XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/cccccccccccccccccccccccccccccc.js XX KiB
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) name-too-long/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		@@ -190,0 +213,2 @@
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js (bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -191,0 +216,1 @@
		+ runtime modules XX KiB XX modules
		@@ -192,1 +218,1 @@
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		@@ -195,0 +221,3 @@
		+ ./e.js XX bytes [built] [code generated]
		+ chunk (runtime: main) name-too-long/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		@@ -196,1 +225,5 @@
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ chunk (runtime: cccccccccccccccccccccccccccccc) name-too-long/cccccccccccccccccccccccccccccc.js (cccccccccccccccccccccccccccccc) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./c cccccccccccccccccccccccccccccc
		+ runtime modules XX KiB XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc, main) name-too-long/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		@@ -199,2 +232,3 @@
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: cccccccccccccccccccccccccccccc) name-too-long/cccccccccccccccccccccccccccccc.js (cccccccccccccccccccccccccccccc) XX KiB ={XX}= [entry] [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		@@ -202,4 +236,6 @@
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		- > ./c ./index.js XX:XX-XX
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [entry] [rendered]
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ runtime modules XX KiB XX modules
		+ ./a.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc, main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -209,0 +245,3 @@
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ > ./c cccccccccccccccccccccccccccccc
		@@ -210,6 +249,1 @@
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b ./index.js XX:XX-XX
		- > ./c ./index.js XX:XX-XX
		- > ./g ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -218,2 +252,3 @@
		- > ./c ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -222,0 +257,1 @@
		+ runtime modules XX KiB XX modules
		@@ -223,1 +259,10 @@
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ chunk (runtime: cccccccccccccccccccccccccccccc, main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./c ./index.js XX:XX-XX
		+ > ./c cccccccccccccccccccccccccccccc
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) name-too-long/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc, main) name-too-long/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -225,4 +270,4 @@
		- ./g.js XX bytes [built] [code generated]
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) XX KiB ={XX}= >{XX}< >{XX}< [entry] [rendered]
		- > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		- name-too-long (Rspack x.x.x) compiled successfully
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ > ./c cccccccccccccccccccccccccccccc
		+ ./f.js XX bytes [built] [code generated]
		+ name-too-long (webpack x.x.x) compiled successfully
		@@ -233,4 +278,16 @@
		- Entrypoint b XX KiB = custom-chunks-filter/async-b.js XX KiB custom-chunks-filter/b.js XX KiB
		- Entrypoint c XX KiB = custom-chunks-filter/async-c.js XX KiB custom-chunks-filter/c.js XX KiB
		- chunk (runtime: c, main) custom-chunks-filter/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		- > ./c ./index.js XX:XX-XX
		+ Entrypoint b XX KiB = custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/b.js XX KiB
		+ Entrypoint c XX KiB = custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/c.js XX KiB
		+ chunk (runtime: a, main) custom-chunks-filter/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) custom-chunks-filter/b.js (b) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./b b
		+ runtime modules XX KiB XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) custom-chunks-filter/c.js (c) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -238,0 +295,1 @@
		+ runtime modules XX KiB XX modules
		@@ -239,1 +297,1 @@
		- chunk (runtime: main) custom-chunks-filter/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ chunk (runtime: b, c, main) custom-chunks-filter/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		@@ -241,3 +299,0 @@
		- dependent modules XX bytes [dependent] XX module
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b, main) custom-chunks-filter/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		@@ -245,0 +300,1 @@
		+ > ./c ./index.js XX:XX-XX
		@@ -246,2 +302,3 @@
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ > ./c c
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: b, c, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -250,2 +307,0 @@
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		@@ -253,2 +308,4 @@
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ > ./b b
		+ > ./c c
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: b, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -257,0 +314,7 @@
		+ > ./b b
		+ ./node_modules/y.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		+ > ./ main
		+ runtime modules XX KiB XX modules
		+ ./index.js XX bytes [built] [code generated]
		+ chunk (runtime: c, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: defaultVendors)
		@@ -258,2 +322,6 @@
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) custom-chunks-filter/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		+ > ./c c
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) custom-chunks-filter/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: default)
		@@ -263,0 +331,2 @@
		+ > ./b b
		+ > ./c c
		@@ -266,17 +336,4 @@
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) custom-chunks-filter/b.js (b) XX KiB ={XX}= [entry] [rendered]
		- > ./b b
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		- > ./a ./index.js XX:XX-XX
		- > ./b ./index.js XX:XX-XX
		- > ./c ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: c) custom-chunks-filter/c.js (c) XX KiB ={XX}= [entry] [rendered]
		- > ./c c
		- chunk (runtime: main) custom-chunks-filter/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		- > ./ main
		- ./index.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) custom-chunks-filter/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		- > ./g ./a.js XX:XX-XX
		- ./g.js XX bytes [built] [code generated]
		- custom-chunks-filter (Rspack x.x.x) compiled successfully
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ custom-chunks-filter (webpack x.x.x) compiled successfully
		@@ -286,5 +343,9 @@
		- Entrypoint a XX KiB = custom-chunks-filter-in-cache-groups/a.js
		- Entrypoint b XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX bytes custom-chunks-filter-in-cache-groups/b.js XX KiB
		- Entrypoint c XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX bytes custom-chunks-filter-in-cache-groups/c.js XX KiB
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c ./index.js XX:XX-XX
		+ Entrypoint a XX KiB = custom-chunks-filter-in-cache-groups/XX.js XX bytes custom-chunks-filter-in-cache-groups/a.js XX KiB
		+ Entrypoint b XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX KiB custom-chunks-filter-in-cache-groups/b.js XX KiB
		+ Entrypoint c XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX KiB custom-chunks-filter-in-cache-groups/c.js XX KiB
		+ chunk (runtime: a, main) custom-chunks-filter-in-cache-groups/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		@@ -292,2 +353,2 @@
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: b, c, main) custom-chunks-filter-in-cache-groups/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors)
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b, c, main) custom-chunks-filter-in-cache-groups/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
		@@ -308,17 +369,0 @@
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		- > ./a ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		- > ./b ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) custom-chunks-filter-in-cache-groups/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< [entry] [rendered]
		- > ./a a
		- > x a
		- > y a
		- > z a
		- dependent modules XX bytes [dependent] XX modules
		- cacheable modules XX bytes
		- ./a.js XX bytes [built] [code generated]
		- ./node_modules/z.js XX bytes [built] [code generated]
		@@ -330,0 +374,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -331,0 +377,4 @@
		+ chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./a.js + XX modules XX bytes [built] [code generated]
		@@ -336,0 +386,2 @@
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -337,0 +389,8 @@
		+ chunk (runtime: a) custom-chunks-filter-in-cache-groups/XX.js (id hint: vendors) XX bytes ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a a
		+ > x a
		+ > y a
		+ > z a
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ ./node_modules/y.js XX bytes [built] [code generated]
		+ ./node_modules/z.js XX bytes [built] [code generated]
		@@ -339,0 +399,1 @@
		+ runtime modules XX KiB XX modules
		@@ -340,2 +401,10 @@
		- chunk (runtime: a, main) custom-chunks-filter-in-cache-groups/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> [rendered]
		- > ./g ./a.js XX:XX-XX
		+ chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a) custom-chunks-filter-in-cache-groups/a.js (a) XX bytes (javascript) XX KiB (runtime) ={XX}= >{XX}< [entry] [rendered]
		+ > ./a a
		+ > x a
		+ > y a
		+ > z a
		+ runtime modules XX KiB XX modules
		@@ -343,2 +412,2 @@
		- ./g.js XX bytes [built] [code generated]
		- custom-chunks-filter-in-cache-groups (Rspack x.x.x) compiled successfully
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ custom-chunks-filter-in-cache-groups (webpack x.x.x) compiled successfully"
	`);
	}
};
