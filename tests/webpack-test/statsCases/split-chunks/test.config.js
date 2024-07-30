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
		- > ./c [XX] ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX module
		- ./a.js XX bytes [built] [code generated]
		+ Entrypoint b XX KiB = default/b.js
		+ Entrypoint c XX KiB = default/c.js
		+ chunk (runtime: a, main) default/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		@@ -14,1 +10,6 @@
		- > ./b [XX] ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) default/b.js (b) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./b b
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		@@ -16,0 +17,18 @@
		+ chunk (runtime: main) default/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) default/c.js (c) XX bytes (javascript) XX bytes (runtime) [entry] [rendered]
		+ > ./c c
		+ dependent modules XX bytes [dependent] XX modules
		+ runtime modules XX bytes XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ ./node_modules/x.js XX bytes [built] [code generated]
		@@ -17,2 +36,2 @@
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		@@ -20,0 +39,4 @@
		+ chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		+ > ./ main
		+ runtime modules XX KiB XX modules
		+ ./index.js XX bytes [built] [code generated]
		@@ -21,1 +44,1 @@
		- > ./c [XX] ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -23,8 +46,6 @@
		- chunk (runtime: main) default/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) default/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		+ chunk (runtime: main) default/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, main) default/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -35,19 +56,4 @@
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) default/b.js (b) XX bytes [entry] [rendered]
		- > ./b b
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		- chunk (runtime: c) default/c.js (c) XX bytes [entry] [rendered]
		- > ./c c
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) default/main.js (main) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< >{XX}< [entry] [rendered]
		- > ./ main
		- ./index.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) default/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		- > ./g ./a.js XX:XX-XX
		- ./g.js XX bytes [built] [code generated]
		- default (Rspack x.x.x) compiled successfully
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ default (webpack x.x.x) compiled successfully
		@@ -57,11 +63,10 @@
		- Entrypoint a XX KiB = all-chunks/XX.js XX KiB all-chunks/a.js XX KiB
		- Entrypoint b XX KiB = all-chunks/async-b.js XX KiB all-chunks/b.js XX KiB
		- Entrypoint c XX KiB = all-chunks/async-c.js XX KiB all-chunks/c.js XX KiB
		- chunk (runtime: c, main) all-chunks/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		- > ./c c
		- > ./c [XX] ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- ./e.js XX bytes [built] [code generated]
		- chunk (runtime: b, main) all-chunks/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ Entrypoint a XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/a.js XX KiB
		+ Entrypoint b XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/b.js XX KiB
		+ Entrypoint c XX KiB = all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/XX.js XX bytes all-chunks/c.js XX KiB
		+ chunk (runtime: a, main) all-chunks/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all-chunks/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) all-chunks/b.js (b) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -69,1 +74,1 @@
		- > ./b [XX] ./index.js XX:XX-XX
		+ runtime modules XX KiB XX modules
		@@ -71,1 +76,2 @@
		- chunk (runtime: a, main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ chunk (runtime: a, main) all-chunks/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		@@ -73,1 +79,3 @@
		- > ./a [XX] ./index.js XX:XX-XX
		+ ./e.js XX bytes [built] [code generated]
		+ chunk (runtime: main) all-chunks/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		@@ -75,18 +83,8 @@
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- > ./g [XX] ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: a) all-chunks/a.js (a) XX KiB ={XX}= >{XX}< >{XX}< [entry] [rendered]
		+ chunk (runtime: c) all-chunks/c.js (c) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./c c
		+ runtime modules XX KiB XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) all-chunks/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -94,1 +92,0 @@
		- chunk (runtime: b) all-chunks/b.js (b) XX KiB ={XX}= [entry] [rendered]
		@@ -96,4 +93,1 @@
		- chunk (runtime: main) all-chunks/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		+ > ./c c
		@@ -101,1 +95,6 @@
		- chunk (runtime: c) all-chunks/c.js (c) XX KiB ={XX}= [entry] [rendered]
		+ chunk (runtime: a, b, c, main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./a a
		+ > ./b b
		@@ -103,0 +102,7 @@
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, main) all-chunks/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./a a
		+ > ./b b
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -105,0 +111,1 @@
		+ runtime modules XX KiB XX modules
		@@ -106,4 +113,19 @@
		- chunk (runtime: a, main) all-chunks/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		- > ./g [XX] ./a.js XX:XX-XX
		- ./g.js XX bytes [built] [code generated]
		- all-chunks (Rspack x.x.x) compiled successfully
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
		+ > ./g ./a.js XX:XX-XX
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
		- > ./c [XX] ./index.js XX:XX-XX
		+ Entrypoint a XX KiB = manual/vendors.js XX KiB manual/a.js XX KiB
		+ Entrypoint b XX KiB = manual/vendors.js XX KiB manual/b.js XX KiB
		+ Entrypoint c XX KiB = manual/vendors.js XX KiB manual/c.js XX KiB
		+ chunk (runtime: a, main) manual/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) manual/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		@@ -119,2 +145,6 @@
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: a, b, c, main) manual/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors)
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) manual/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./a a
		@@ -124,1 +154,1 @@
		- > ./a a
		+ > ./b b
		@@ -128,1 +158,1 @@
		- > ./b b
		+ > ./c c
		@@ -132,4 +162,0 @@
		- > ./c c
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		@@ -139,14 +165,0 @@
		- chunk (runtime: main) manual/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: main) manual/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		- > ./b [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) manual/a.js (a) XX bytes (javascript) XX KiB (runtime) ={XX}= >{XX}< [entry] [rendered]
		- > x a
		- > y a
		- > z a
		- > ./a a
		- ./a.js XX bytes [built] [code generated]
		@@ -154,0 +166,1 @@
		+ > ./b b
		@@ -157,1 +170,2 @@
		- > ./b b
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -159,0 +173,4 @@
		+ chunk (runtime: main) manual/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./a.js + XX modules XX bytes [built] [code generated]
		@@ -160,0 +178,1 @@
		+ > ./c c
		@@ -163,1 +182,2 @@
		- > ./c c
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -167,0 +187,1 @@
		+ runtime modules XX KiB XX modules
		@@ -168,2 +189,10 @@
		- chunk (runtime: a, main) manual/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> [rendered]
		- > ./g [XX] ./a.js XX:XX-XX
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
		@@ -176,4 +205,21 @@
		- Entrypoint aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa XX KiB = name-too-long/XX.js XX KiB name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js XX KiB
		- Entrypoint bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb XX KiB = name-too-long/async-b.js XX KiB name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js XX KiB
		- Entrypoint cccccccccccccccccccccccccccccc XX KiB = name-too-long/async-c.js XX KiB name-too-long/cccccccccccccccccccccccccccccc.js XX KiB
		- chunk (runtime: cccccccccccccccccccccccccccccc, main) name-too-long/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ Entrypoint aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js XX KiB
		+ Entrypoint bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js XX KiB
		+ Entrypoint cccccccccccccccccccccccccccccc XX KiB = name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/XX.js XX bytes name-too-long/cccccccccccccccccccccccccccccc.js XX KiB
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) name-too-long/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js (bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ runtime modules XX KiB XX modules
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ ./e.js XX bytes [built] [code generated]
		+ chunk (runtime: main) name-too-long/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js XX bytes [built] [code generated]
		+ chunk (runtime: cccccccccccccccccccccccccccccc) name-too-long/cccccccccccccccccccccccccccccc.js (cccccccccccccccccccccccccccccc) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -181,1 +227,1 @@
		- > ./c [XX] ./index.js XX:XX-XX
		+ runtime modules XX KiB XX modules
		@@ -183,6 +229,5 @@
		- chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) name-too-long/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.js (bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb) XX KiB ={XX}= [entry] [rendered]
		- > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		- chunk (runtime: main) name-too-long/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- ./e.js XX bytes [built] [code generated]
		- chunk (runtime: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, main) name-too-long/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc, main) name-too-long/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		@@ -190,3 +235,3 @@
		- > ./b [XX] ./index.js XX:XX-XX
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./c cccccccccccccccccccccccccccccc
		+ ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [entry] [rendered]
		@@ -194,1 +239,1 @@
		- > ./a [XX] ./index.js XX:XX-XX
		+ runtime modules XX KiB XX modules
		@@ -196,5 +241,6 @@
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: cccccccccccccccccccccccccccccc) name-too-long/cccccccccccccccccccccccccccccc.js (cccccccccccccccccccccccccccccc) XX KiB ={XX}= [entry] [rendered]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, cccccccccccccccccccccccccccccc, main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		@@ -202,7 +248,0 @@
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		@@ -210,10 +249,6 @@
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- > ./g [XX] ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: main) name-too-long/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- ./d.js XX bytes [built] [code generated]
		+ chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, main) name-too-long/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -222,0 +257,1 @@
		+ runtime modules XX KiB XX modules
		@@ -223,6 +259,15 @@
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, main) name-too-long/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		- > ./g [XX] ./a.js XX:XX-XX
		- ./g.js XX bytes [built] [code generated]
		- chunk (runtime: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) name-too-long/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.js (aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa) XX KiB ={XX}= >{XX}< >{XX}< [entry] [rendered]
		- > ./a aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		- name-too-long (Rspack x.x.x) compiled successfully
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
		+ > ./g ./a.js XX:XX-XX
		+ > ./b bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
		+ > ./c cccccccccccccccccccccccccccccc
		+ ./f.js XX bytes [built] [code generated]
		+ name-too-long (webpack x.x.x) compiled successfully
		@@ -233,11 +278,9 @@
		- Entrypoint b XX KiB = custom-chunks-filter/async-b.js XX KiB custom-chunks-filter/b.js XX KiB
		- Entrypoint c XX KiB = custom-chunks-filter/async-c.js XX KiB custom-chunks-filter/c.js XX KiB
		- chunk (runtime: c, main) custom-chunks-filter/async-c.js (async-c) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		- > ./c c
		- > ./c [XX] ./index.js XX:XX-XX
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX module
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b, main) custom-chunks-filter/async-b.js (async-b) (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered]
		+ Entrypoint b XX KiB = custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/b.js XX KiB
		+ Entrypoint c XX KiB = custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/XX.js XX bytes custom-chunks-filter/c.js XX KiB
		+ chunk (runtime: a, main) custom-chunks-filter/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/async-b.js (async-b) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b) custom-chunks-filter/b.js (b) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		@@ -245,1 +288,1 @@
		- > ./b [XX] ./index.js XX:XX-XX
		+ runtime modules XX KiB XX modules
		@@ -247,21 +290,11 @@
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- ./node_modules/y.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: defaultVendors)
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/z.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: defaultVendors)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- ./node_modules/x.js XX bytes [built] [code generated]
		- chunk (runtime: a, main) custom-chunks-filter/XX.js (id hint: ) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [rendered] split chunk (cache group: default)
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		- > ./g ./a.js XX:XX-XX
		- ./f.js XX bytes [built] [code generated]
		- chunk (runtime: a) custom-chunks-filter/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		- > ./a a
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: b) custom-chunks-filter/b.js (b) XX KiB ={XX}= [entry] [rendered]
		+ chunk (runtime: main) custom-chunks-filter/async-a.js (async-a) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ chunk (runtime: c) custom-chunks-filter/c.js (c) XX bytes (javascript) XX KiB (runtime) ={XX}= ={XX}= ={XX}= ={XX}= [entry] [rendered]
		+ > ./c c
		+ runtime modules XX KiB XX modules
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: b, c, main) custom-chunks-filter/XX.js XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: default)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -269,4 +302,1 @@
		- chunk (runtime: main) custom-chunks-filter/XX.js (id hint: ) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [rendered] split chunk (cache group: default)
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		+ > ./c c
		@@ -274,1 +304,5 @@
		- chunk (runtime: c) custom-chunks-filter/c.js (c) XX KiB ={XX}= [entry] [rendered]
		+ chunk (runtime: b, c, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./b b
		@@ -276,0 +310,6 @@
		+ ./node_modules/x.js XX bytes [built] [code generated]
		+ chunk (runtime: b, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< >{XX}< [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./b b
		+ ./node_modules/y.js XX bytes [built] [code generated]
		@@ -278,0 +318,1 @@
		+ runtime modules XX KiB XX modules
		@@ -279,1 +320,10 @@
		- chunk (runtime: a, main) custom-chunks-filter/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= [rendered]
		+ chunk (runtime: c, main) custom-chunks-filter/XX.js (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: defaultVendors)
		+ > ./c ./index.js XX:XX-XX
		+ > ./c c
		+ ./node_modules/z.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter/async-c.js (async-c) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= [rendered]
		+ > ./c ./index.js XX:XX-XX
		+ ./c.js XX bytes [built] [code generated]
		+ chunk (runtime: a, b, c, main) custom-chunks-filter/XX.js XX bytes <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= [initial] [rendered] split chunk (cache group: default)
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		@@ -281,2 +331,9 @@
		- ./g.js XX bytes [built] [code generated]
		- custom-chunks-filter (Rspack x.x.x) compiled successfully
		+ > ./b b
		+ > ./c c
		+ ./f.js XX bytes [built] [code generated]
		+ chunk (runtime: a) custom-chunks-filter/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< >{XX}< [entry] [rendered]
		+ > ./a a
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		+ ./a.js + XX modules XX bytes [built] [code generated]
		+ custom-chunks-filter (webpack x.x.x) compiled successfully
		@@ -286,5 +343,9 @@
		- Entrypoint a XX KiB = custom-chunks-filter-in-cache-groups/a.js
		- Entrypoint b XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX bytes custom-chunks-filter-in-cache-groups/b.js XX KiB
		- Entrypoint c XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX bytes custom-chunks-filter-in-cache-groups/c.js XX KiB
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-c.js (async-c) XX bytes <{XX}> ={XX}= [rendered]
		- > ./c [XX] ./index.js XX:XX-XX
		+ Entrypoint a XX KiB = custom-chunks-filter-in-cache-groups/XX.js XX bytes custom-chunks-filter-in-cache-groups/a.js XX KiB
		+ Entrypoint b XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX KiB custom-chunks-filter-in-cache-groups/b.js XX KiB
		+ Entrypoint c XX KiB = custom-chunks-filter-in-cache-groups/vendors.js XX KiB custom-chunks-filter-in-cache-groups/c.js XX KiB
		+ chunk (runtime: a, main) custom-chunks-filter-in-cache-groups/async-g.js (async-g) XX bytes <{XX}> <{XX}> <{XX}> <{XX}> [rendered]
		+ > ./g ./a.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./g.js XX bytes [built] [code generated]
		+ chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		+ > ./b ./index.js XX:XX-XX
		@@ -292,2 +353,6 @@
		- ./c.js XX bytes [built] [code generated]
		- chunk (runtime: b, c, main) custom-chunks-filter-in-cache-groups/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors)
		+ ./b.js XX bytes [built] [code generated]
		+ chunk (runtime: b, c, main) custom-chunks-filter-in-cache-groups/vendors.js (vendors) (id hint: vendors) XX bytes <{XX}> ={XX}= ={XX}= ={XX}= ={XX}= ={XX}= >{XX}< [initial] [rendered] split chunk (cache group: vendors) (name: vendors)
		+ > ./a ./index.js XX:XX-XX
		+ > ./b ./index.js XX:XX-XX
		+ > ./c ./index.js XX:XX-XX
		+ > ./b b
		@@ -297,1 +362,1 @@
		- > ./b b
		+ > ./c c
		@@ -301,4 +366,0 @@
		- > ./c c
		- > ./a [XX] ./index.js XX:XX-XX
		- > ./b [XX] ./index.js XX:XX-XX
		- > ./c [XX] ./index.js XX:XX-XX
		@@ -308,17 +369,0 @@
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		- > ./a [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./a.js XX bytes [built] [code generated]
		- chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-b.js (async-b) XX bytes <{XX}> ={XX}= [rendered]
		- > ./b [XX] ./index.js XX:XX-XX
		- dependent modules XX bytes [dependent] XX modules
		- ./b.js XX bytes [built] [code generated]
		- chunk (runtime: a) custom-chunks-filter-in-cache-groups/a.js (a) XX bytes (javascript) XX KiB (runtime) >{XX}< [entry] [rendered]
		- > x a
		- > y a
		- > z a
		- > ./a a
		- dependent modules XX bytes [dependent] XX modules
		- cacheable modules XX bytes
		- ./a.js XX bytes [built] [code generated]
		- ./node_modules/z.js XX bytes [built] [code generated]
		@@ -326,0 +370,1 @@
		+ > ./b b
		@@ -329,1 +374,2 @@
		- > ./b b
		+ runtime modules XX KiB XX modules
		+ dependent modules XX bytes [dependent] XX modules
		@@ -331,0 +377,4 @@
		+ chunk (runtime: main) custom-chunks-filter-in-cache-groups/async-a.js (async-a) XX bytes <{XX}> ={XX}= >{XX}< [rendered]
		+ > ./a ./index.js XX:XX-XX
		+ dependent modules XX bytes [dependent] XX module
		+ ./a.js + XX modules XX bytes [built] [code generated]
		@@ -332,0 +382,1 @@
		+ > ./c c
		@@ -335,1 +386,2 @@
		- > ./c c
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
