const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {

		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,3 +1,4 @@
			- asset a-runtime~main-XXfXXbXXfeXXeXXcXXa.js XX KiB [emitted] [immutable] (name: runtime~main)
			- asset a-main-XXebXXfXXafXXbXX.js XX bytes [emitted] [immutable] (name: main) (id hint: all)
			- Entrypoint main XX KiB = a-runtime~main-XXfXXbXXfeXXeXXcXXa.js XX KiB a-main-XXebXXfXXafXXbXX.js XX bytes
			+ asset a-runtime~main-XXbaXXcXXfXXaXXaXX.js XX KiB [emitted] [immutable] (name: runtime~main)
			+ asset a-main-XXbXXcXXdXXcXX.js XX bytes [emitted] [immutable] (name: main)
			+ asset a-all-a_js-XXfbXXfXXeXXcXX.js XX bytes [emitted] [immutable] (id hint: all)
			+ Entrypoint main XX KiB = a-runtime~main-XXbaXXcXXfXXaXXaXX.js XX KiB a-all-a_js-XXfbXXfXXeXXcXX.js XX bytes a-main-XXbXXcXXdXXcXX.js XX bytes
			@@ -6,1 +7,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms
			@@ -8,4 +9,5 @@
			- asset b-runtime~main-XXcXXfXXffbXX.js XX KiB [emitted] [immutable] (name: runtime~main)
			- asset b-main-XXaXXcXXeXXdXXbXXccbXX.js XX bytes [emitted] [immutable] (name: main) (id hint: all)
			- asset b-vendors-node_modules_vendor_js-XXdXXfXXcXXccXXeXX.js XX bytes [emitted] [immutable] (id hint: vendors)
			- Entrypoint main XX KiB = b-runtime~main-XXcXXfXXffbXX.js XX KiB b-vendors-node_modules_vendor_js-XXdXXfXXcXXccXXeXX.js XX bytes b-main-XXaXXcXXeXXdXXbXXccbXX.js XX bytes
			+ asset b-runtime~main-bXXacXXcXXaXXceXXe.js XX KiB [emitted] [immutable] (name: runtime~main)
			+ asset b-all-b_js-XXccaeXXaaXXdXXeXX.js XX bytes [emitted] [immutable] (id hint: all)
			+ asset b-main-XXfXXbXXbeXXdXXac.js XX bytes [emitted] [immutable] (name: main)
			+ asset b-vendors-node_modules_vendor_js-XXfXXdbabXXeXXeadXX.js XX bytes [emitted] [immutable] (id hint: vendors)
			+ Entrypoint main XX KiB = b-runtime~main-bXXacXXcXXaXXceXXe.js XX KiB b-vendors-node_modules_vendor_js-XXfXXdbabXXeXXeadXX.js XX bytes b-all-b_js-XXccaeXXaaXXdXXeXX.js XX bytes b-main-XXfXXbXXbeXXdXXac.js XX bytes
			@@ -16,1 +18,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms
			@@ -18,6 +20,7 @@
			- assets by chunk XX KiB (id hint: all)
			- asset c-main-baddXXdXXdfXXffXXefXX.js XX bytes [emitted] [immutable] (name: main) (id hint: all)
			- asset c-all-b_js-aXXaXXccXXabXXc.js XX bytes [emitted] [immutable] (id hint: all)
			- asset c-runtime~main-XXaXXfXXccXXcXXaXXbXX.js XX KiB [emitted] [immutable] (name: runtime~main)
			- asset c-vendors-node_modules_vendor_js-XXdXXfXXcXXccXXeXX.js XX bytes [emitted] [immutable] (id hint: vendors)
			- Entrypoint main XX KiB = c-runtime~main-XXaXXfXXccXXcXXaXXbXX.js XX KiB c-main-baddXXdXXdfXXffXXefXX.js XX bytes
			+ assets by chunk XX bytes (id hint: all)
			+ asset c-all-b_js-dXXdXXfdaadbfXXb.js XX bytes [emitted] [immutable] (id hint: all)
			+ asset c-all-c_js-XXcXXcbbXXcXXaXXbXXbXXc.js XX bytes [emitted] [immutable] (id hint: all)
			+ asset c-runtime~main-XXeXXcaXXaefXXcXX.js XX KiB [emitted] [immutable] (name: runtime~main)
			+ asset c-main-XXcXXfXXfeXXbbXX.js XX bytes [emitted] [immutable] (name: main)
			+ asset c-vendors-node_modules_vendor_js-XXfXXdbabXXeXXeadXX.js XX bytes [emitted] [immutable] (id hint: vendors)
			+ Entrypoint main XX KiB = c-runtime~main-XXeXXcaXXaefXXcXX.js XX KiB c-all-c_js-XXcXXcbbXXcXXaXXbXXbXXc.js XX bytes c-main-XXcXXfXXfeXXbbXX.js XX bytes
			@@ -29,1 +32,1 @@
			- Rspack x.x.x compiled successfully in X.XX
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
