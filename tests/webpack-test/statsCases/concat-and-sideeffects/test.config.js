const diffStats = require("../../helpers/diffStats");
const path = require("path");

module.exports = {
	validate(stats, error, actual) {
		expect(diffStats(actual, path.basename(__dirname)))
			.toMatchInlineSnapshot(`
		"- Expected
		+ Received

		@@ -1,9 +1,5 @@
		- ./index.js XX bytes [orphan] [built]
		- Statement with side_effects in source code at ./index.js:XX:XX-XX
		- ModuleConcatenation bailout: Module is an entry point
		- ./index.js + XX modules XX bytes [code generated]
		- | ./index.js XX bytes [orphan] [built]
		- |   Statement with side_effects in source code at ./index.js:XX:XX-XX
		- |   ModuleConcatenation bailout: Module is an entry point
		- | ./node_modules/pmodule/a.js XX bytes [orphan] [built]
		- | ./node_modules/pmodule/aa.js XX bytes [orphan] [built]
		+ ./index.js + XX modules XX bytes [built] [code generated]
		+ | ./index.js XX bytes [built]
		+ |   Statement (ExpressionStatement) with side effects in source code at XX:XX-XX
		+ | ./node_modules/pmodule/a.js XX bytes [built]
		+ | ./node_modules/pmodule/aa.js XX bytes [built]"
	`);
	}
};
