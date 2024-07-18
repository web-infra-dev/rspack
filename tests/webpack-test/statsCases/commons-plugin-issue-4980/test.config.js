const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
			"- Expected
			+ Received

			@@ -1,1 +1,4 @@
			- asset app.aXXebXXeXXedXXccXXbcbdXX-XX.js XX bytes [emitted] [immutable] (name: app)
			+ asset app.XXeXXeXXbdXX-XX.js XX KiB [emitted] [immutable] (name: app)
			+ asset vendor.ffdXXfXXaefXXd-XX.js XX bytes [emitted] [immutable] (name: vendor) (id hint: vendor)
			+ Entrypoint app XX KiB = vendor.ffdXXfXXaefXXd-XX.js XX bytes app.XXeXXeXXbdXX-XX.js XX KiB
			+ runtime modules XX KiB XX modules
			@@ -3,2 +6,4 @@
			- ./entry-XX.js XX bytes [built] [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ cacheable modules XX bytes
			+ ./entry-XX.js + XX modules XX bytes [built] [code generated]
			+ ./constants.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully in X ms
			@@ -6,1 +11,4 @@
			- asset app.XXcXXdbXXeXXfff-XX.js XX bytes [emitted] [immutable] (name: app)
			+ asset app.cXXbXXcXXbXXeXXda-XX.js XX KiB [emitted] [immutable] (name: app)
			+ asset vendor.ffdXXfXXaefXXd-XX.js XX bytes [emitted] [immutable] (name: vendor) (id hint: vendor)
			+ Entrypoint app XX KiB = vendor.ffdXXfXXaefXXd-XX.js XX bytes app.cXXbXXcXXbXXeXXda-XX.js XX KiB
			+ runtime modules XX KiB XX modules
			@@ -8,2 +16,4 @@
			- ./entry-XX.js XX bytes [built] [code generated]
			- Rspack x.x.x compiled successfully in X.XX
			+ cacheable modules XX bytes
			+ ./entry-XX.js + XX modules XX bytes [built] [code generated]
			+ ./constants.js XX bytes [built] [code generated]
			+ webpack x.x.x compiled successfully in X ms"
		`);

	}
};
