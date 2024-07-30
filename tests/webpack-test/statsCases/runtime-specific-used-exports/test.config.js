const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -4,5 +4,5 @@
		- asset production-dx_js.js XX bytes [emitted]
		- asset production-dw_js-_XXcbaXX.js XX bytes [emitted]
		- asset production-dw_js-_XXcbaXX.js XX bytes [emitted]
		- asset production-dy_js.js XX bytes [emitted]
		- asset production-dz_js.js XX bytes [emitted]
		+ asset production-dw_js-_aXX.js XX KiB [emitted]
		+ asset production-dw_js-_aXX.js XX KiB [emitted]
		+ asset production-dx_js.js XX KiB [emitted]
		+ asset production-dy_js.js XX KiB [emitted]
		+ asset production-dz_js.js XX KiB [emitted]
		@@ -11,0 +11,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -16,1 +18,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: x]
		@@ -18,1 +20,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: x]
		@@ -20,1 +22,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: x]
		@@ -22,0 +24,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -27,1 +31,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: y]
		@@ -29,1 +33,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: y]
		@@ -31,1 +35,1 @@
		- [only some exports used: x, y]
		+ [only some exports used: y]
		@@ -35,1 +39,1 @@
		- chunk (runtime: a) production-dw_js-_XXcbaXX.js XX bytes [rendered]
		+ chunk (runtime: a) production-dw_js-_aXX.js XX bytes [rendered]
		@@ -38,2 +42,2 @@
		- [only some exports used: identity, w, x, y, z]
		- chunk (runtime: b) production-dw_js-_XXcbaXX.js XX bytes [rendered]
		+ [only some exports used: identity, w, x, y]
		+ chunk (runtime: b) production-dw_js-_aXX.js XX bytes [rendered]
		@@ -42,1 +46,1 @@
		- [only some exports used: identity, w, x, y, z]
		+ [only some exports used: identity, w, x, z]
		@@ -50,1 +54,1 @@
		- [only some exports used: identity, w, x, y, z]
		+ [only some exports used: identity, w, x, y]
		@@ -54,1 +58,1 @@
		- [only some exports used: identity, w, x, y, z]
		+ [only some exports used: identity, w, x, z]
		@@ -58,0 +62,4 @@
		+ [no exports used]
		+ ./b.js XX bytes [built] [code generated]
		+ [no exports used]
		+ ./c.js XX bytes [built] [code generated]
		@@ -63,2 +71,0 @@
		- ./module.js?reexported XX bytes [built] [code generated]
		- [only some exports used: x, y]
		@@ -67,4 +73,0 @@
		- ./b.js XX bytes [built] [code generated]
		- [no exports used]
		- ./c.js XX bytes [built] [code generated]
		- [no exports used]
		@@ -72,0 +74,4 @@
		+ ./dw.js XX bytes [built] [code generated]
		+ ./dz.js XX bytes [built] [code generated]
		+ ./module.js?reexported XX bytes [built] [code generated]
		+ [only some exports used: x, y]
		@@ -74,1 +80,0 @@
		- ./dw.js XX bytes [built] [code generated]
		@@ -76,2 +81,1 @@
		- ./dz.js XX bytes [built] [code generated]
		- production (Rspack x.x.x) compiled successfully in X.XX
		+ production (webpack x.x.x) compiled successfully in X ms
		@@ -86,1 +90,1 @@
		- asset development-c.js XX bytes [emitted] (name: c)
		+ asset development-c.js XX KiB [emitted] (name: c)
		@@ -88,0 +92,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -99,0 +105,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -136,0 +144,4 @@
		+ ./b.js XX bytes [built] [code generated]
		+ [used exports unknown]
		+ ./c.js XX bytes [built] [code generated]
		+ [used exports unknown]
		@@ -139,2 +151,0 @@
		- [used exports unknown]
		- ./module.js?reexported XX bytes [built] [code generated]
		@@ -143,0 +153,2 @@
		+ [used exports unknown]
		+ ./dy.js XX bytes [built] [code generated]
		@@ -144,1 +156,1 @@
		- ./b.js XX bytes [built] [code generated]
		+ ./dw.js XX bytes [built] [code generated]
		@@ -146,1 +158,1 @@
		- ./c.js XX bytes [built] [code generated]
		+ ./dz.js XX bytes [built] [code generated]
		@@ -148,1 +160,1 @@
		- ./dy.js XX bytes [built] [code generated]
		+ ./module.js?reexported XX bytes [built] [code generated]
		@@ -152,2 +164,0 @@
		- ./dw.js XX bytes [built] [code generated]
		- [used exports unknown]
		@@ -156,3 +166,1 @@
		- ./dz.js XX bytes [built] [code generated]
		- [used exports unknown]
		- development (Rspack x.x.x) compiled successfully in X.XX
		+ development (webpack x.x.x) compiled successfully in X ms
		@@ -163,4 +171,4 @@
		- asset global-dw_js.js XX bytes [emitted]
		- asset global-dx_js.js XX bytes [emitted]
		- asset global-dy_js.js XX bytes [emitted]
		- asset global-dz_js.js XX bytes [emitted]
		+ asset global-dw_js.js XX KiB [emitted]
		+ asset global-dx_js.js XX KiB [emitted]
		+ asset global-dy_js.js XX KiB [emitted]
		+ asset global-dz_js.js XX KiB [emitted]
		@@ -169,0 +177,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -180,0 +190,2 @@
		+ runtime modules XX KiB XX modules
		+ cacheable modules XX bytes
		@@ -212,0 +224,4 @@
		+ [no exports used]
		+ ./b.js XX bytes [built] [code generated]
		+ [no exports used]
		+ ./c.js XX bytes [built] [code generated]
		@@ -217,2 +233,0 @@
		- ./module.js?reexported XX bytes [built] [code generated]
		- [only some exports used: x, y]
		@@ -221,4 +235,0 @@
		- ./b.js XX bytes [built] [code generated]
		- [no exports used]
		- ./c.js XX bytes [built] [code generated]
		- [no exports used]
		@@ -226,0 +236,4 @@
		+ ./dw.js XX bytes [built] [code generated]
		+ ./dz.js XX bytes [built] [code generated]
		+ ./module.js?reexported XX bytes [built] [code generated]
		+ [only some exports used: x, y]
		@@ -228,1 +242,0 @@
		- ./dw.js XX bytes [built] [code generated]
		@@ -230,2 +243,1 @@
		- ./dz.js XX bytes [built] [code generated]
		- global (Rspack x.x.x) compiled successfully in X.XX
		+ global (webpack x.x.x) compiled successfully in X ms"
	`);
	}
};
