const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname))).toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -1,0 +1,2 @@
		+ Entrypoint first XX KiB = a-vendor.js XX bytes a-first.js XX KiB
		+ Entrypoint second XX KiB = a-vendor.js XX bytes a-second.js XX KiB
		@@ -2,1 +4,1 @@
		- orphan modules XX bytes [orphan] XX modules
		+ orphan modules XX bytes [orphan] XX module
		@@ -6,0 +8,3 @@
		+ ./vendor.js XX bytes [built] [code generated]
		+ ./module_first.js XX bytes [built] [code generated]
		+ ./commonXX.js XX bytes [built] [code generated]
		@@ -7,2 +12,0 @@
		- ./common_lazy.js XX bytes [built] [code generated]
		- ./common_lazy_shared.js XX bytes [built] [code generated]
		@@ -11,1 +14,3 @@
		- Rspack x.x.x compiled successfully in X.XX
		+ ./common_lazy.js XX bytes [built] [code generated]
		+ ./common_lazy_shared.js XX bytes [built] [code generated]
		+ webpack x.x.x compiled successfully in X ms
		@@ -13,0 +18,2 @@
		+ Entrypoint first XX KiB = b-vendor.js XX bytes b-first.js XX KiB
		+ Entrypoint second XX KiB = b-vendor.js XX bytes b-second.js XX KiB
		@@ -14,7 +21,1 @@
		- cacheable modules XX KiB
		- built modules XX bytes [built]
		- orphan modules XX bytes [orphan]
		- ./lazy_first.js XX bytes [orphan] [built]
		- ./common_lazy.js XX bytes [orphan] [built]
		- ./lazy_second.js XX bytes [orphan] [built]
		- + XX modules
		+ cacheable modules XX bytes
		@@ -22,6 +23,11 @@
		- ./first.js XX bytes [built] [code generated]
		- Statement with side_effects in source code at ./first.js:XX:XX-XX
		- ModuleConcatenation bailout: Module is an entry point
		- ./second.js XX bytes [built] [code generated]
		- Statement with side_effects in source code at ./second.js:XX:XX-XX
		- ModuleConcatenation bailout: Module is an entry point
		+ ./first.js + XX modules XX bytes [built] [code generated]
		+ ModuleConcatenation bailout: Cannot concat with ./vendor.js: Module ./vendor.js is not in the same chunk(s) (expected in chunk(s) first, module is in chunk(s) vendor)
		+ ./second.js + XX modules XX bytes [built] [code generated]
		+ ModuleConcatenation bailout: Cannot concat with ./vendor.js: Module ./vendor.js is not in the same chunk(s) (expected in chunk(s) second, module is in chunk(s) vendor)
		+ ./vendor.js XX bytes [built] [code generated]
		+ ./lazy_first.js + XX modules XX bytes [built] [code generated]
		+ ModuleConcatenation bailout: Cannot concat with ./common_lazy_shared.js: Module ./common_lazy_shared.js is referenced from different chunks by these modules: ./lazy_shared.js
		+ ./lazy_shared.js XX bytes [built] [code generated]
		+ ModuleConcatenation bailout: Cannot concat with ./common_lazy_shared.js: Module ./common_lazy_shared.js is referenced from different chunks by these modules: ./lazy_first.js, ./lazy_second.js
		+ ./lazy_second.js + XX modules XX bytes [built] [code generated]
		+ ModuleConcatenation bailout: Cannot concat with ./common_lazy_shared.js: Module ./common_lazy_shared.js is referenced from different chunks by these modules: ./lazy_shared.js
		@@ -29,5 +35,7 @@
		- ./lazy_shared.js XX bytes [built] [code generated]
		- ModuleConcatenation bailout: Cannot concat with Xdir/scope-hoisting-multi/common_lazy_shared.js: Module ./common_lazy_shared.js is referenced from different chunks by these modules: ./lazy_first.js, ./lazy_second.js
		- ./lazy_first.js + XX modules XX bytes [code generated]
		- ./lazy_second.js + XX modules XX bytes [code generated]
		- Rspack x.x.x compiled successfully in X.XX
		+ orphan modules XX bytes [orphan]
		+ ./module_first.js XX bytes [orphan] [built]
		+ ./commonXX.js XX bytes [orphan] [built]
		+ ./common.js XX bytes [orphan] [built]
		+ ModuleConcatenation bailout: Module is not in any chunk
		+ ./common_lazy.js XX bytes [orphan] [built]
		+ webpack x.x.x compiled successfully in X ms"
	`);
	}
};
